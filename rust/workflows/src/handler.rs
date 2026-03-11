use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;
use stencila_attractor::{
    context::Context,
    events::{EventEmitter, PipelineEvent},
    graph::{Graph, Node},
    handler::Handler,
    interpolation::expand_runtime_variables,
    interviewer::Interviewer,
    types::Outcome,
};

use crate::{
    get_by_name,
    run::{ParentRun, RunOptions, run_workflow_with_options_and_parent},
};

#[derive(Clone)]
pub struct WorkflowHandler {
    workflow_home: PathBuf,
    emitter: Arc<dyn EventEmitter>,
    interviewer: Option<Arc<dyn Interviewer>>,
}

impl WorkflowHandler {
    pub fn new(
        workflow_home: PathBuf,
        emitter: Arc<dyn EventEmitter>,
        interviewer: Option<Arc<dyn Interviewer>>,
    ) -> Self {
        Self {
            workflow_home,
            emitter,
            interviewer,
        }
    }
}

#[async_trait]
impl Handler for WorkflowHandler {
    async fn execute(
        &self,
        node: &Node,
        context: &Context,
        _graph: &Graph,
    ) -> stencila_attractor::error::AttractorResult<Outcome> {
        let Some(workflow_name) = node.get_str_attr("workflow") else {
            return Ok(Outcome::fail(format!(
                "node '{}' has type 'workflow' but no 'workflow' attribute",
                node.id
            )));
        };

        let workflow_name = workflow_name.trim();
        if workflow_name.is_empty() {
            return Ok(Outcome::fail(format!(
                "node '{}' has type 'workflow' but an empty 'workflow' attribute",
                node.id
            )));
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let stage_index = context.get_i64("internal.stage_index").unwrap_or(0) as usize;

        let raw_input = node
            .get_str_attr("prompt")
            .or_else(|| node.get_str_attr("label"))
            .unwrap_or_default();
        let input = expand_runtime_variables(raw_input, context);
        let goal = node
            .get_str_attr("goal")
            .map(|goal| expand_runtime_variables(goal, context))
            .filter(|goal| !goal.is_empty());

        self.emitter.emit(PipelineEvent::StageInput {
            node_id: node.id.clone(),
            stage_index,
            input: workflow_name.to_string(),
            agent_name: String::new(),
        });

        let child = get_by_name(&self.workflow_home, workflow_name)
            .await
            .map_err(
                |error| stencila_attractor::error::AttractorError::HandlerFailed {
                    node_id: node.id.clone(),
                    reason: format!("Unable to resolve workflow `{workflow_name}`: {error}"),
                },
            )?;

        let mut child_input = IndexMap::new();
        child_input.insert("parent.node_id".to_string(), serde_json::json!(node.id));
        child_input.insert(
            "parent.workflow".to_string(),
            serde_json::json!(context.get_string("internal.workflow_name")),
        );
        if let Some(goal) = goal {
            child_input.insert("goal".to_string(), serde_json::json!(goal));
        } else if !input.is_empty() {
            child_input.insert("goal".to_string(), serde_json::json!(input.clone()));
        }
        if !input.is_empty() {
            child_input.insert("input".to_string(), serde_json::json!(input));
        }

        let outcome = run_workflow_with_options_and_parent(
            &child,
            RunOptions {
                emitter: self.emitter.clone(),
                interviewer: self.interviewer.clone(),
            },
            Some(ParentRun {
                run_id: context.get_string("internal.run_id"),
                node_id: node.id.clone(),
            }),
            Some(child_input),
        )
        .await
        .map_err(
            |error| stencila_attractor::error::AttractorError::HandlerFailed {
                node_id: node.id.clone(),
                reason: format!("Subworkflow `{workflow_name}` failed to run: {error}"),
            },
        )?;

        Ok(map_child_outcome(node, outcome))
    }
}

fn map_child_outcome(node: &Node, child: Outcome) -> Outcome {
    let mut outcome = child.clone();

    let child_output = child
        .context_updates
        .get("last_output_full")
        .cloned()
        .unwrap_or_else(|| serde_json::json!(child.notes.clone()));

    outcome
        .context_updates
        .insert("last_stage".to_string(), serde_json::json!(node.id.clone()));
    outcome
        .context_updates
        .insert("last_output".to_string(), child_output.clone());
    outcome
        .context_updates
        .insert("last_output_full".to_string(), child_output.clone());
    outcome
        .context_updates
        .insert("shell.output".to_string(), child_output.clone());
    outcome
        .context_updates
        .insert("shell".to_string(), child_output.clone());
    outcome
        .context_updates
        .insert(format!("workflow.output.{}", node.id), child_output);
    outcome.context_updates.insert(
        format!("workflow.outcome.{}", node.id),
        serde_json::to_value(&child).unwrap_or_else(|_| serde_json::json!({})),
    );

    if outcome.notes.is_empty() {
        outcome.notes = format!("Workflow completed for node '{}'", node.id);
    }

    outcome
}

#[cfg(test)]
mod tests {
    use stencila_attractor::types::StageStatus;

    use super::*;

    #[test]
    fn map_child_outcome_sets_parent_output_keys() {
        let node = Node::new("compose");
        let mut child = Outcome::success();
        child.notes = "done".into();
        child
            .context_updates
            .insert("last_output_full".into(), serde_json::json!("hello"));

        let mapped = map_child_outcome(&node, child);

        assert_eq!(mapped.status, StageStatus::Success);
        assert_eq!(
            mapped.context_updates.get("last_stage"),
            Some(&serde_json::json!("compose"))
        );
        assert_eq!(
            mapped.context_updates.get("last_output_full"),
            Some(&serde_json::json!("hello"))
        );
        assert_eq!(
            mapped.context_updates.get("workflow.output.compose"),
            Some(&serde_json::json!("hello"))
        );
    }

    #[test]
    fn workflow_handler_uses_shared_runtime_variable_expansion() {
        let context = Context::new();
        context.set("last_output_full", serde_json::json!("child goal"));

        assert_eq!(
            expand_runtime_variables("Use this: $last_output", &context),
            "Use this: child goal"
        );
    }

    #[test]
    fn workflow_handler_preserves_unknown_variables() {
        let context = Context::new();

        assert_eq!(
            expand_runtime_variables("COUNT=$COUNT; test $COUNT -ge 2", &context),
            "COUNT=$COUNT; test $COUNT -ge 2"
        );
        assert_eq!(
            expand_runtime_variables("Previous stage: $last_stage", &context),
            "Previous stage: $last_stage"
        );
    }
}
