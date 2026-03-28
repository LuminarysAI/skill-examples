/// @skill:id      ai.luminarys.rust.intent-classifier
/// @skill:name    "Intent Classifier"
/// @skill:version 1.0.0
/// @skill:desc    "Classifies user text into intents for skill routing. Uses an embedded ONNX model (char trigram features + linear classifier) via Tract. Multilingual (EN/RU)."

mod classifier;

use luminarys_sdk::prelude::*;

/// Classify user text and return the top intent with confidence score.
///
/// @skill:method classify "Classify user text into an intent category for routing."
/// @skill:param  text required "User input text to classify"
/// @skill:result "JSON with intent, confidence, and all scores"
pub fn classify(_ctx: &mut Context, text: String) -> Result<String, SkillError> {
    let result = classifier::classify(&text)?;
    serde_json::to_string(&result).map_err(|e| SkillError(format!("json: {e}")))
}

/// Return the list of known intent labels.
///
/// @skill:method intents "Return all supported intent labels."
/// @skill:result "JSON array of intent label strings"
pub fn intents(_ctx: &mut Context) -> Result<String, SkillError> {
    let labels = classifier::intent_labels();
    serde_json::to_string(&labels).map_err(|e| SkillError(format!("json: {e}")))
}

/// Classify text and return only the top intent label (no scores).
///
/// @skill:method route "Return only the best matching intent label. Optimized for routing."
/// @skill:param  text required "User input text"
/// @skill:result "Intent label string"
pub fn route(_ctx: &mut Context, text: String) -> Result<String, SkillError> {
    let result = classifier::classify(&text)?;
    Ok(result.intent)
}

/// Health check.
///
/// @skill:method ping "Health check. Returns pong if the model is loaded."
/// @skill:result "pong"
pub fn ping(_ctx: &mut Context) -> Result<String, SkillError> {
    // Force model init on first call.
    classifier::ensure_loaded()?;
    Ok("pong".into())
}
