#!/usr/bin/env python3
"""
Train a tiny intent classifier and export to ONNX.

Model: character trigram hashing (5000 buckets) + logistic regression.
Output: model/intent.onnx (~160 KB)

Usage:
    pip install numpy scikit-learn onnx
    python train.py
"""

import json
import pathlib

import numpy as np
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import cross_val_score

# ── Constants (must match classifier.rs) ─────────────────────────────────────

N_FEATURES = 5000

SCRIPT_DIR = pathlib.Path(__file__).parent
DATA_PATH = SCRIPT_DIR / "intents.json"
MODEL_PATH = SCRIPT_DIR.parent / "model" / "intent.onnx"


# ── Tokenizer (must match classifier.rs exactly) ────────────────────────────

def extract_features(text: str) -> np.ndarray:
    """Character trigram hashing — same algorithm as Rust classifier."""
    features = np.zeros(N_FEATURES, dtype=np.float32)
    lower = text.lower()
    chars = list(lower)

    if len(chars) < 3:
        for ch in chars:
            h = (ord(ch) * 7919) % N_FEATURES
            features[h] += 1.0
        if len(chars) == 2:
            h = (ord(chars[0]) * 7919 + ord(chars[1]) * 31) % N_FEATURES
            features[h] += 1.0
        norm = np.linalg.norm(features)
        if norm > 0:
            features /= norm
        return features

    for i in range(len(chars) - 2):
        c0, c1, c2 = ord(chars[i]), ord(chars[i + 1]), ord(chars[i + 2])
        h = (c0 * 7919 + c1 * 31 + c2) % N_FEATURES
        features[h] += 1.0

    norm = np.linalg.norm(features)
    if norm > 0:
        features /= norm
    return features


# ── Load data ────────────────────────────────────────────────────────────────

def load_data():
    with open(DATA_PATH, encoding="utf-8") as f:
        data = json.load(f)

    intents = data["intents"]
    examples = data["examples"]

    X, y = [], []
    for idx, intent in enumerate(intents):
        for text in examples[intent]:
            X.append(extract_features(text))
            y.append(idx)

    return np.array(X), np.array(y), intents


# ── Train ────────────────────────────────────────────────────────────────────

def train():
    X, y, intents = load_data()
    print(f"Loaded {len(X)} examples, {len(intents)} intents: {intents}")

    clf = LogisticRegression(
        max_iter=1000,
        C=1.0,
        solver="lbfgs",
    )

    # Cross-validation score.
    scores = cross_val_score(clf, X, y, cv=min(5, len(X)), scoring="accuracy")
    print(f"Cross-validation accuracy: {scores.mean():.3f} (+/- {scores.std():.3f})")

    # Train on full data.
    clf.fit(X, y)
    train_acc = clf.score(X, y)
    print(f"Training accuracy: {train_acc:.3f}")

    return clf, intents


# ── Export to ONNX ───────────────────────────────────────────────────────────

def export_onnx(clf, intents):
    """Build ONNX graph manually: Gemm(input, W^T, bias) -> Softmax -> output."""
    import onnx
    from onnx import helper, TensorProto

    n_intents = len(intents)
    W = clf.coef_.astype(np.float32)         # [n_intents, N_FEATURES]
    b = clf.intercept_.astype(np.float32)     # [n_intents]

    # ONNX initializers.
    W_init = helper.make_tensor("W", TensorProto.FLOAT, W.shape, W.flatten().tolist())
    b_init = helper.make_tensor("b", TensorProto.FLOAT, b.shape, b.flatten().tolist())

    # Graph nodes.
    gemm_node = helper.make_node(
        "Gemm",
        inputs=["features", "W", "b"],
        outputs=["logits"],
        transB=1,  # W is [n_intents, N_FEATURES], needs transpose
    )

    softmax_node = helper.make_node(
        "Softmax",
        inputs=["logits"],
        outputs=["probabilities"],
        axis=1,
    )

    # Input / output.
    input_def = helper.make_tensor_value_info("features", TensorProto.FLOAT, [1, N_FEATURES])
    output_def = helper.make_tensor_value_info("probabilities", TensorProto.FLOAT, [1, n_intents])

    graph = helper.make_graph(
        [gemm_node, softmax_node],
        "intent_classifier",
        [input_def],
        [output_def],
        initializer=[W_init, b_init],
    )

    model = helper.make_model(graph, opset_imports=[helper.make_opsetid("", 13)])
    model.ir_version = 7

    # Add intent labels as metadata.
    for i, name in enumerate(intents):
        entry = model.metadata_props.add()
        entry.key = f"intent_{i}"
        entry.value = name

    onnx.checker.check_model(model)

    MODEL_PATH.parent.mkdir(parents=True, exist_ok=True)
    onnx.save(model, str(MODEL_PATH))

    size_kb = MODEL_PATH.stat().st_size / 1024
    print(f"Saved {MODEL_PATH} ({size_kb:.1f} KB)")
    print(f"  Weights: {W.shape} ({W.nbytes / 1024:.1f} KB)")
    print(f"  Bias:    {b.shape} ({b.nbytes:.0f} bytes)")
    print(f"  Intents: {intents}")


# ── Quick test ───────────────────────────────────────────────────────────────

def test_model(clf, intents):
    test_cases = [
        ("read the file config.yaml", "files"),
        ("прочитай файл main.go", "files"),
        ("search the web for rust tutorials", "web"),
        ("поищи в интернете", "web"),
        ("run the test suite", "code"),
        ("запусти тесты", "code"),
        ("commit and push", "git"),
        ("покажи git log", "git"),
        ("how much disk space is left", "system"),
        ("покажи информацию о системе", "system"),
        ("hello, how are you", "general"),
        ("привет, что умеешь", "general"),
    ]

    print("\n-- Test predictions --")
    correct = 0
    for text, expected in test_cases:
        features = extract_features(text).reshape(1, -1)
        probs = clf.predict_proba(features)[0]
        pred_idx = np.argmax(probs)
        pred = intents[pred_idx]
        ok = "OK" if pred == expected else "FAIL"
        if pred == expected:
            correct += 1
        print(f"  [{ok}] '{text}' -> {pred} ({probs[pred_idx]:.2f})  expected={expected}")

    print(f"\nTest accuracy: {correct}/{len(test_cases)}")


# ── Main ─────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    clf, intents = train()
    test_model(clf, intents)
    export_onnx(clf, intents)
