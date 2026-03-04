//! Machine learning pack: `ml.experiment_tracking`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, safe_pattern};

// ---------------------------------------------------------------------------
// ml.experiment_tracking
// ---------------------------------------------------------------------------

pub static EXPERIMENT_TRACKING_PACK: Pack = Pack {
    id: "ml.experiment_tracking",
    name: "ML Experiment Tracking",
    description: "Guards against destructive ML experiment and model tracking operations",
    safe_patterns: &[
        safe_pattern!("nvidia_smi", r"^nvidia-smi\b[^|><]*$"),
        safe_pattern!("gpustat", r"^gpustat\b[^|><]*$"),
        safe_pattern!("wandb_status", r"^wandb\s+status\b[^|><]*$"),
        safe_pattern!("mlflow_models_list", r"^mlflow\s+models\s+list\b[^|><]*$"),
    ],
    destructive_patterns: &[
        destructive_pattern!(
            "mlflow_gc",
            r"\bmlflow\s+gc\b",
            "Permanently deletes expired MLflow runs and artifacts",
            "Use `mlflow experiments search` to review before garbage collecting",
            Confidence::High
        ),
        destructive_pattern!(
            "mlflow_delete_experiment",
            r"\bmlflow\s+experiments\s+delete\b",
            "Deletes an MLflow experiment and all its runs",
            "Use `mlflow experiments search` to review the experiment first",
            Confidence::High
        ),
        destructive_pattern!(
            "wandb_artifact_delete",
            r"\bwandb\s+artifact\s+delete\b",
            "Permanently deletes a Weights & Biases artifact version",
            "Use `wandb artifact get` to inspect the artifact first",
            Confidence::High
        ),
        destructive_pattern!(
            "wandb_sweep_cancel",
            r"\bwandb\s+sweep\s+cancel\b",
            "Cancels a running hyperparameter sweep and all its agents",
            "Review sweep progress in the W&B dashboard first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "clearml_task_delete",
            r"\bclearml-task\b.*--delete\b",
            "Deletes ClearML tasks and associated artifacts",
            "Review tasks with `clearml-task list` before deleting",
            Confidence::High
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- MLflow --

    #[test]
    fn mlflow_gc_matches() {
        let re = Regex::new(rule_by_id(&EXPERIMENT_TRACKING_PACK, "mlflow_gc").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("mlflow gc"));
        assert!(re.is_match("mlflow gc --backend-store-uri sqlite:///mlflow.db"));
        assert!(!re.is_match("mlflow experiments search"));
        assert!(!re.is_match("mlflow run ."));
        assert!(!re.is_match("mlflow ui"));
    }

    #[test]
    fn mlflow_delete_experiment_matches() {
        let re =
            Regex::new(rule_by_id(&EXPERIMENT_TRACKING_PACK, "mlflow_delete_experiment").pattern)
                .expect("pattern should compile");
        assert!(re.is_match("mlflow experiments delete --experiment-id 1"));
        assert!(re.is_match("mlflow experiments delete --experiment-id 42"));
        assert!(!re.is_match("mlflow experiments search"));
        assert!(!re.is_match("mlflow experiments list"));
        assert!(!re.is_match("mlflow experiments create --experiment-name test"));
    }

    // -- Weights & Biases --

    #[test]
    fn wandb_artifact_delete_matches() {
        let re = Regex::new(rule_by_id(&EXPERIMENT_TRACKING_PACK, "wandb_artifact_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("wandb artifact delete my-project/model:v0"));
        assert!(re.is_match("wandb artifact delete user/project/artifact:latest"));
        assert!(!re.is_match("wandb artifact get my-project/model:v0"));
        assert!(!re.is_match("wandb artifact ls my-project"));
        assert!(!re.is_match("wandb artifact put model.onnx"));
    }

    #[test]
    fn wandb_sweep_cancel_matches() {
        let re = Regex::new(rule_by_id(&EXPERIMENT_TRACKING_PACK, "wandb_sweep_cancel").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("wandb sweep cancel user/project/sweep_id"));
        assert!(re.is_match("wandb sweep cancel abc123"));
        assert!(!re.is_match("wandb sweep create config.yaml"));
        assert!(!re.is_match("wandb sweep"));
        assert!(!re.is_match("wandb sweep agent abc123"));
    }

    // -- ClearML --

    #[test]
    fn clearml_task_delete_matches() {
        let re = Regex::new(rule_by_id(&EXPERIMENT_TRACKING_PACK, "clearml_task_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("clearml-task --delete --id abc123"));
        assert!(re.is_match("clearml-task --project myproject --delete"));
        assert!(!re.is_match("clearml-task list"));
        assert!(!re.is_match("clearml-task --project myproject"));
        assert!(!re.is_match("clearml-task --id abc123"));
    }
}
