use luminarys_sdk::prelude::SkillError;
use serde::Serialize;
use std::cell::RefCell;
use std::io::Cursor;
use tract_onnx::prelude::*;

// ── Embedded model ──────────────────────────────────────────────────────────

const MODEL_BYTES: &[u8] = include_bytes!("../../model/intent.onnx");

/// Number of hash buckets for character trigram features.
/// Must match N_FEATURES in train.py.
const N_FEATURES: usize = 5000;

/// Intent labels. Order must match train.py INTENTS list.
const INTENTS: &[&str] = &[
    "files",
    "web",
    "code",
    "git",
    "system",
    "general",
];

// ── Types ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ClassifyResult {
    pub intent: String,
    pub confidence: f64,
    pub scores: Vec<IntentScore>,
}

#[derive(Serialize)]
pub struct IntentScore {
    pub label: String,
    pub score: f64,
}

// ── Model singleton ─────────────────────────────────────────────────────────

type Model = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

thread_local! {
    static MODEL: RefCell<Option<Model>> = const { RefCell::new(None) };
}

fn load_model() -> Result<Model, SkillError> {
    let mut reader = Cursor::new(MODEL_BYTES);
    tract_onnx::onnx()
        .model_for_read(&mut reader)
        .map_err(|e| SkillError(format!("onnx load: {e}")))?
        .into_optimized()
        .map_err(|e| SkillError(format!("onnx optimize: {e}")))?
        .into_runnable()
        .map_err(|e| SkillError(format!("onnx runnable: {e}")))
}

pub fn ensure_loaded() -> Result<(), SkillError> {
    MODEL.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            *opt = Some(load_model()?);
        }
        Ok(())
    })
}

// ── Tokenizer ───────────────────────────────────────────────────────────────

/// Extract character trigram features using hashing trick.
/// Algorithm must match train.py exactly.
fn extract_features(text: &str) -> Vec<f32> {
    let mut features = vec![0.0f32; N_FEATURES];
    let lower: Vec<char> = text.to_lowercase().chars().collect();

    if lower.len() < 3 {
        // For very short texts, use unigrams and bigrams too.
        for ch in &lower {
            let h = (*ch as u64 * 7919) % N_FEATURES as u64;
            features[h as usize] += 1.0;
        }
        if lower.len() == 2 {
            let h = (lower[0] as u64 * 7919 + lower[1] as u64 * 31) % N_FEATURES as u64;
            features[h as usize] += 1.0;
        }
        return normalize(features);
    }

    for win in lower.windows(3) {
        let h = (win[0] as u64 * 7919 + win[1] as u64 * 31 + win[2] as u64) % N_FEATURES as u64;
        features[h as usize] += 1.0;
    }

    normalize(features)
}

/// L2-normalize the feature vector.
fn normalize(mut v: Vec<f32>) -> Vec<f32> {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
    v
}

// ── Inference ───────────────────────────────────────────────────────────────

pub fn classify(text: &str) -> Result<ClassifyResult, SkillError> {
    ensure_loaded()?;

    let features = extract_features(text);
    let input = tract_ndarray::Array2::from_shape_vec((1, N_FEATURES), features)
        .map_err(|e| SkillError(format!("input shape: {e}")))?;
    let input_tensor: TValue = input.into_tvalue();

    MODEL.with(|cell| {
        let opt = cell.borrow();
        let model = opt.as_ref().unwrap();
        let outputs = model
            .run(tvec![input_tensor])
            .map_err(|e| SkillError(format!("inference: {e}")))?;

        let probs = outputs[0]
            .to_array_view::<f32>()
            .map_err(|e| SkillError(format!("output: {e}")))?;

        let mut best_idx = 0;
        let mut best_score: f32 = 0.0;
        let mut scores = Vec::with_capacity(INTENTS.len());

        for (i, &score) in probs.iter().enumerate() {
            if i < INTENTS.len() {
                scores.push(IntentScore {
                    label: INTENTS[i].to_string(),
                    score: score as f64,
                });
                if score > best_score {
                    best_score = score;
                    best_idx = i;
                }
            }
        }

        // Sort scores descending.
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ClassifyResult {
            intent: INTENTS[best_idx].to_string(),
            confidence: best_score as f64,
            scores,
        })
    })
}

pub fn intent_labels() -> Vec<String> {
    INTENTS.iter().map(|s| s.to_string()).collect()
}
