---
title: "ML"
description: "Guards against destructive ML experiment and model tracking operations"
---

This page lists the safe and destructive patterns in the **ML Experiment Tracking** shell guard pack. See [Shell Tool](/docs/agents/tools/shell#guard-pipeline) for how these patterns are evaluated.

## ML Experiment Tracking

**Pack ID:** `ml.experiment_tracking`

Guards against destructive ML experiment and model tracking operations

### Safe patterns

| Rule ID | Pattern |
|---------|--------|
| `ml.experiment_tracking.nvidia_smi` | `^nvidia-smi\b[^\|><]*$` |
| `ml.experiment_tracking.gpustat` | `^gpustat\b[^\|><]*$` |
| `ml.experiment_tracking.wandb_status` | `^wandb\s+status\b[^\|><]*$` |
| `ml.experiment_tracking.mlflow_models_list` | `^mlflow\s+models\s+list\b[^\|><]*$` |

### Destructive patterns

| Rule ID | Reason | Suggestion | Confidence |
|---------|--------|------------|:----------:|
| `ml.experiment_tracking.mlflow_gc` | Permanently deletes expired MLflow runs and artifacts | Use `mlflow experiments search` to review before garbage collecting | High |
| `ml.experiment_tracking.mlflow_delete_experiment` | Deletes an MLflow experiment and all its runs | Use `mlflow experiments search` to review the experiment first | High |
| `ml.experiment_tracking.wandb_artifact_delete` | Permanently deletes a Weights & Biases artifact version | Use `wandb artifact get` to inspect the artifact first | High |
| `ml.experiment_tracking.wandb_sweep_cancel` | Cancels a running hyperparameter sweep and all its agents | Review sweep progress in the W&B dashboard first | Medium |
| `ml.experiment_tracking.clearml_task_delete` | Deletes ClearML tasks and associated artifacts | Review tasks with `clearml-task list` before deleting | High |

---

This documentation was generated from [`rust/agents/src/tool_guard/shell/packs/ml.rs`](https://github.com/stencila/stencila/blob/main/rust/agents/src/tool_guard/shell/packs/ml.rs).
