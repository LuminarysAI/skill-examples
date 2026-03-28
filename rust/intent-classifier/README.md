# Intent Classifier

Classifies user text into intents for skill routing. Uses an embedded ONNX model (character trigram features + linear classifier) via [Tract](https://github.com/sonos/tract). Multilingual (EN/RU).

## Methods

| Method | Parameters | Description |
|--------|-----------|-------------|
| `classify` | `text` (string, required) | Classify text, return intent + confidence + all scores (JSON) |
| `route` | `text` (string, required) | Return only the top intent label |
| `intents` | — | List supported intent labels |
| `ping` | — | Health check (verifies model is loaded) |

## Intent Labels

`files`, `web`, `code`, `git`, `system`, `general`

## Model Training

The model is pre-trained and embedded in the binary. To retrain:

```bash
cd train
pip install numpy scikit-learn onnx
python train.py
```

Training data: `train/intents.json` (bilingual EN/RU examples).
Output: `model/intent.onnx` (~120 KB).

## Build

```bash
lmsk generate -lang rust ./src
cargo build --target wasm32-wasip1 --release
lmsk sign target/wasm32-wasip1/release/intent_classifier.wasm
```

## Manifest

```yaml
id: intent-classifier
path: skills/ai.luminarys.rust.intent-classifier.skill
permissions:
  fs: { enabled: false }
invoke_policy:
  can_invoke: []
  can_be_invoked_by: ["*"]
mcp:
  mapping: per_method
```
