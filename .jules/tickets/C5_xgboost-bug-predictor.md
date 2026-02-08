# TICKET C5 — XGBoost Bug Predictor

## Context
- Source: Session 11
- Train on Defects4J and BugsInPy datasets
- Model: 10K trees, 10MB, <0.1ms inference

## Requirements
1. Download and preprocess Defects4J + BugsInPy datasets
2. Extract features per line/function:
   - Cyclomatic complexity
   - Lines of code
   - Nesting depth
   - Number of contributors
   - Time since last change
   - Number of past bugs in this file
   - Function length
   - Parameter count
3. Train XGBoost model (binary classification: buggy/not-buggy)
4. Export to ONNX format for Rust inference
5. Evaluate: AUROC, precision, recall, F1

## Files to Create
- `forge/ml/bug_predictor/train.py`
- `forge/ml/bug_predictor/features.py`
- `forge/ml/bug_predictor/evaluate.py`
- `forge/ml/bug_predictor/export_onnx.py`
- `forge/ml/bug_predictor/requirements.txt` (xgboost, scikit-learn, onnx, onnxmltools)

## Acceptance Criteria
- [ ] Model trained on real data
- [ ] AUROC > 0.75
- [ ] ONNX export successful
- [ ] Model size < 15MB
- [ ] Inference time < 0.5ms
- [ ] Evaluation report generated

## Effort: 3 days → WITH JULES: ~1 session
