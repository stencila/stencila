use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use inflector::Inflector;
use stencila_attractor::events::PipelineEvent;

use crate::{
    agent::AgentProgress, autocomplete::workflows::WorkflowDefinitionInfo,
    config::WorkflowVerbosity,
};

use super::{
    ActiveWorkflow, ActiveWorkflowState, App, AppMessage, ExchangeKind, ExchangeStatus,
    WorkflowProgressKind, WorkflowStatusState,
};

impl App {
    /// Enter workflow mode for the given workflow definition.
    ///
    /// If a previous workflow is still running it is cancelled first so we
    /// don't leak a detached background task.
    pub(super) fn activate_workflow(&mut self, info: WorkflowDefinitionInfo) {
        self.cancel_active_workflow();
        self.mode = super::AppMode::Workflow;
        let default_goal = info.goal.clone();
        self.active_workflow = Some(ActiveWorkflow {
            info,
            state: ActiveWorkflowState::Pending,
            run_handle: None,
            current_exchange_msg_index: None,
            current_stage_progress: None,
            workflow_status_msg_index: None,
            stage_status_msg_index: None,
            in_parallel: false,
            parallel_stages: HashMap::new(),
            parallel_had_failure: false,
        });
        if let Some(goal) = default_goal {
            self.input.set_text(&goal);
        } else {
            self.input.clear();
        }
        self.input_scroll = 0;
        self.scroll_pinned = true;
    }

    /// Submit a goal for the active workflow and spawn the workflow run.
    pub(super) fn submit_workflow_goal(&mut self, goal: String) {
        let Some(workflow) = &self.active_workflow else {
            return;
        };

        let (handle, ..) = crate::workflow::spawn_workflow(&workflow.info, goal);

        if let Some(workflow) = &mut self.active_workflow {
            workflow.run_handle = Some(handle);
            workflow.state = ActiveWorkflowState::Running;
            workflow.workflow_status_msg_index = None;
            workflow.stage_status_msg_index = None;
        }

        self.scroll_pinned = true;
        self.scroll_offset = 0;
    }

    pub(super) fn handle_pipeline_event(&mut self, event: &PipelineEvent) {
        match self.config.workflow_verbosity {
            WorkflowVerbosity::Minimal => self.handle_pipeline_event_minimal(event),
            WorkflowVerbosity::Simple => self.handle_pipeline_event_simple(event),
            WorkflowVerbosity::Detailed => self.handle_pipeline_event_detailed(event),
        }
    }

    /// Minimal verbosity: a single in-place-updated status message for the entire pipeline.
    ///
    /// NOTE: parallel execution events are not handled here; parallel
    /// stages are effectively invisible in minimal mode. Use detailed
    /// verbosity for full parallel stage visibility.
    fn handle_pipeline_event_minimal(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                let msg_index = self.messages.len();

                self.messages.push(AppMessage::WorkflowStatus {
                    state: WorkflowStatusState::Running,
                    label: format!("Workflow `{pipeline_name}`"),
                    detail: None,
                });

                if let Some(workflow) = &mut self.active_workflow {
                    workflow.workflow_status_msg_index = Some(msg_index);
                }
            }
            PipelineEvent::StageStarted { stage_index, .. } if *stage_index > 0 => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.workflow_status_msg_index)
                    && let Some(AppMessage::WorkflowStatus {
                        state: phase,
                        detail,
                        ..
                    }) = self.messages.get_mut(idx)
                {
                    *phase = WorkflowStatusState::Running;
                    *detail = Some(format!("Stage: {stage_index}"));
                }
            }
            PipelineEvent::PipelineCompleted { outcome, .. } => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.workflow_status_msg_index)
                    && let Some(AppMessage::WorkflowStatus {
                        state: phase,
                        detail,
                        ..
                    }) = self.messages.get_mut(idx)
                {
                    *phase = WorkflowStatusState::Completed;
                    *detail = Some(format!("Completed: {}", outcome.status.as_str()));
                }
            }
            PipelineEvent::PipelineFailed { reason, .. } => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.workflow_status_msg_index)
                    && let Some(AppMessage::WorkflowStatus {
                        state: phase,
                        detail,
                        ..
                    }) = self.messages.get_mut(idx)
                {
                    *phase = WorkflowStatusState::Failed;
                    *detail = Some(format!("Failed: {reason}"));
                }
            }
            // All other events are suppressed in minimal mode
            _ => {}
        }
    }

    /// Simple mode: progress messages including one in-place-updated status message per stage.
    ///
    /// NOTE: parallel execution events are not handled here; concurrent
    /// stage updates may overwrite each other in the single status
    /// message slot. Use detailed verbosity for full parallel stage
    /// visibility.
    #[allow(clippy::too_many_lines)]
    fn handle_pipeline_event_simple(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Started,
                    label: format!("Workflow `{pipeline_name}` started"),
                    detail: None,
                });
            }
            PipelineEvent::StageStarted {
                node_id,
                stage_index,
                ..
            } if *stage_index > 0 => {
                let msg_index = self.messages.len();
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Running,
                    label: format!("Workflow stage {stage_index}: {node_id}"),
                    detail: None,
                });
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.stage_status_msg_index = Some(msg_index);
                }
            }
            PipelineEvent::StageInput {
                agent_name, input, ..
            } => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.stage_status_msg_index)
                    && let Some(AppMessage::WorkflowProgress { detail, .. }) =
                        self.messages.get_mut(idx)
                {
                    *detail = Some(format!("{agent_name}: {input}"));
                }
            }
            PipelineEvent::StageCompleted { outcome, .. } => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.stage_status_msg_index)
                    && let Some(AppMessage::WorkflowProgress { kind, detail, .. }) =
                        self.messages.get_mut(idx)
                {
                    if outcome.status.is_success() {
                        *kind = WorkflowProgressKind::Completed;
                    } else {
                        *kind = WorkflowProgressKind::Failed;
                    }
                    *detail = Some(outcome.status.as_str().to_title_case());
                }
            }
            PipelineEvent::StageFailed { reason, .. } => {
                if let Some(idx) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.stage_status_msg_index)
                    && let Some(AppMessage::WorkflowProgress { kind, detail, .. }) =
                        self.messages.get_mut(idx)
                {
                    *kind = WorkflowProgressKind::Failed;
                    *detail = Some(reason.clone());
                }
            }
            PipelineEvent::StageRetrying {
                node_id,
                stage_index,
                attempt,
                max_attempts,
            } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Retrying,
                    label: format!(
                        "Stage {stage_index}: {node_id} retrying ({attempt}/{max_attempts})",
                    ),
                    detail: None,
                });
            }
            PipelineEvent::PipelineCompleted {
                pipeline_name,
                outcome,
                ..
            } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Completed,
                    label: format!("Workflow `{pipeline_name}` completed"),
                    detail: Some(outcome.status.as_str().to_title_case()),
                });
            }
            PipelineEvent::PipelineFailed {
                pipeline_name,
                reason,
                ..
            } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Failed,
                    label: format!("Workflow `{pipeline_name}` failed"),
                    detail: Some(reason.to_string()),
                });
            }
            // Suppress parallel, interview, checkpoint, text delta, response end
            _ => {}
        }
    }

    /// Detailed mode: full exchange per stage with prompt + streaming response.
    ///
    /// During parallel (fan-out) execution, each branch gets its own
    /// `AppMessage::Exchange` so prompts and responses are grouped per
    /// branch instead of interleaved into a single block.
    #[allow(clippy::too_many_lines)]
    fn handle_pipeline_event_detailed(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Started,
                    label: format!("Workflow `{pipeline_name}` started"),
                    detail: None,
                });
            }
            PipelineEvent::StageStarted {
                node_id,
                stage_index,
                handler_type,
            } if *stage_index > 0 => {
                let msg_index = self.messages.len();
                self.messages.push(AppMessage::Exchange {
                    kind: ExchangeKind::Workflow,
                    status: ExchangeStatus::Running,
                    request: format!("**Stage {stage_index}: {node_id}**"),
                    response: None,
                    response_segments: None,
                    exit_code: None,
                    agent_index: None,
                    agent_name: None,
                    handler_type: Some(handler_type.clone()),
                });
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.current_exchange_msg_index = Some(msg_index);
                    workflow.current_stage_progress =
                        Some(Arc::new(Mutex::new(AgentProgress::default())));
                }
            }
            PipelineEvent::StageInput {
                node_id,
                agent_name,
                input,
                ..
            } => {
                self.color_registry.color_for(agent_name);

                // During parallel execution, route to the branch's own exchange.
                let is_parallel = self.active_workflow.as_ref().is_some_and(|w| w.in_parallel);

                if is_parallel {
                    let msg_index = self.messages.len();
                    self.messages.push(AppMessage::Exchange {
                        kind: ExchangeKind::Workflow,
                        status: ExchangeStatus::Running,
                        request: format!("**{node_id}**\n\n{input}"),
                        response: None,
                        response_segments: None,
                        exit_code: None,
                        agent_index: None,
                        agent_name: Some(agent_name.clone()),
                        // `StageInput` is currently emitted only for
                        // agent-backed stages, so the agent color will be
                        // used and no handler-type fallback is needed here.
                        handler_type: None,
                    });
                    let progress = Arc::new(Mutex::new(AgentProgress::default()));
                    if let Some(workflow) = &mut self.active_workflow {
                        workflow
                            .parallel_stages
                            .insert(node_id.clone(), (msg_index, progress));
                    }
                } else if let Some(msg_index) = self
                    .active_workflow
                    .as_ref()
                    .and_then(|w| w.current_exchange_msg_index)
                    && let Some(AppMessage::Exchange {
                        request,
                        agent_name: workflow_agent_name,
                        ..
                    }) = self.messages.get_mut(msg_index)
                {
                    request.push_str("\n\n");
                    request.push_str(input);
                    *workflow_agent_name = Some(agent_name.clone());
                }
            }
            PipelineEvent::StageSessionEvent {
                node_id, event: se, ..
            } => {
                let stage_progress = self.active_workflow.as_ref().and_then(|w| {
                    if w.in_parallel {
                        w.parallel_stages.get(node_id).cloned()
                    } else {
                        None
                    }
                });

                if let Some((msg_index, progress)) = stage_progress {
                    // Parallel stage: route to per-stage progress
                    crate::agent::process_event(se, &progress);
                    if let Ok(g) = progress.lock() {
                        let text = crate::agent::plain_text_from_segments(&g.segments);
                        Self::update_exchange_streaming(
                            &mut self.messages,
                            msg_index,
                            text,
                            g.segments.clone(),
                        );
                    }
                } else if let Some(workflow) = &self.active_workflow
                    && let Some(ref progress) = workflow.current_stage_progress
                {
                    // Normal (non-parallel) stage
                    crate::agent::process_event(se, progress);
                    if let Some(msg_index) = workflow.current_exchange_msg_index
                        && let Ok(g) = progress.lock()
                    {
                        let text = crate::agent::plain_text_from_segments(&g.segments);
                        Self::update_exchange_streaming(
                            &mut self.messages,
                            msg_index,
                            text,
                            g.segments.clone(),
                        );
                    }
                }
            }
            PipelineEvent::StageOutput {
                node_id, output, ..
            } => {
                // During parallel execution, finalise the per-stage
                // exchange: set the response text (if the stage didn't
                // stream session events) and mark as succeeded.
                if let Some(workflow) = &self.active_workflow
                    && workflow.in_parallel
                    && let Some((msg_index, _)) = workflow.parallel_stages.get(node_id)
                {
                    let msg_index = *msg_index;
                    if let Some(AppMessage::Exchange {
                        status, response, ..
                    }) = self.messages.get_mut(msg_index)
                    {
                        if response.is_none() {
                            *response = Some(output.clone());
                        }
                        *status = ExchangeStatus::Succeeded;
                    }
                }
            }
            PipelineEvent::ParallelStarted { .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.in_parallel = true;
                    workflow.parallel_stages.clear();
                    workflow.parallel_had_failure = false;
                }
            }
            PipelineEvent::ParallelBranchFailed { reason, .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.parallel_had_failure = true;
                }
                // We cannot identify which specific stage exchanges
                // belong to this branch (the event carries the parallel
                // node id and branch index, not individual stage node
                // ids). Emit a standalone failure message so the user
                // sees the error.
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Failed,
                    label: "Parallel branch failed".to_string(),
                    detail: Some(reason.clone()),
                });
            }
            PipelineEvent::ParallelCompleted { .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    // Mark remaining running stage exchanges based on
                    // whether any branch failed. If a failure occurred we
                    // cannot tell which specific stages belonged to the
                    // failed branch (the event model doesn't expose that),
                    // so remaining running exchanges are marked Failed to
                    // avoid displaying a misleading success state.
                    let fallback_status = if workflow.parallel_had_failure {
                        ExchangeStatus::Failed
                    } else {
                        ExchangeStatus::Succeeded
                    };
                    let indices: Vec<usize> = workflow
                        .parallel_stages
                        .values()
                        .map(|(idx, _)| *idx)
                        .collect();
                    for idx in indices {
                        if let Some(AppMessage::Exchange { status, .. }) =
                            self.messages.get_mut(idx)
                            && *status == ExchangeStatus::Running
                        {
                            *status = fallback_status;
                        }
                    }
                    workflow.in_parallel = false;
                    workflow.parallel_stages.clear();
                    workflow.parallel_had_failure = false;
                }
            }
            PipelineEvent::StageCompleted {
                node_id, outcome, ..
            } => {
                if let Some(workflow) = &mut self.active_workflow {
                    let completed_status = if outcome.status.is_success() {
                        ExchangeStatus::Succeeded
                    } else {
                        ExchangeStatus::Failed
                    };

                    // Parallel stage path: look up the per-stage exchange.
                    // (Currently branch subgraphs use NoOpEmitter for
                    // lifecycle events so this won't fire, but it is here
                    // for defensive correctness if that changes.)
                    if workflow.in_parallel {
                        if let Some((msg_index, _)) = workflow.parallel_stages.get(node_id) {
                            let msg_index = *msg_index;
                            if let Some(AppMessage::Exchange { status, .. }) =
                                self.messages.get_mut(msg_index)
                            {
                                *status = completed_status;
                            }
                        }
                    } else if let Some(msg_index) = workflow.current_exchange_msg_index.take()
                        && let Some(AppMessage::Exchange { status, .. }) =
                            self.messages.get_mut(msg_index)
                    {
                        *status = completed_status;
                    }
                    workflow.current_stage_progress = None;
                }
            }
            PipelineEvent::StageFailed { reason, .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    if let Some(msg_index) = workflow.current_exchange_msg_index.take() {
                        Self::update_exchange_at(
                            &mut self.messages,
                            msg_index,
                            ExchangeStatus::Failed,
                            Some(reason.clone()),
                            None,
                        );
                    }
                    workflow.current_stage_progress = None;
                }
            }
            PipelineEvent::StageRetrying {
                node_id,
                stage_index,
                attempt,
                max_attempts,
            } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Retrying,
                    label: format!(
                        "Stage {stage_index}: {node_id} retrying ({attempt}/{max_attempts})",
                    ),
                    detail: None,
                });
            }
            PipelineEvent::PipelineCompleted {
                pipeline_name,
                outcome,
                ..
            } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Completed,
                    label: format!("Workflow `{pipeline_name}` completed"),
                    detail: Some(outcome.status.as_str().to_title_case()),
                });
            }
            PipelineEvent::PipelineFailed {
                pipeline_name,
                reason,
                ..
            } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Failed,
                    label: format!("Workflow `{pipeline_name}` failed"),
                    detail: Some(reason.to_string()),
                });
            }
            // Suppress remaining events: ParallelBranchStarted, ParallelBranchCompleted, interview, checkpoint
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    use super::super::{App, AppMode};
    use crate::autocomplete::workflows::WorkflowDefinitionInfo;

    fn key_event(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[tokio::test]
    async fn ctrl_d_exits_workflow_mode() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
        });
        assert_eq!(app.mode, AppMode::Workflow);
        assert!(app.active_workflow.is_some());

        app.handle_event(&key_event(KeyCode::Char('d'), KeyModifiers::CONTROL))
            .await;
        assert_eq!(app.mode, AppMode::Agent);
        assert!(app.active_workflow.is_some());
    }

    #[tokio::test]
    async fn workflow_ctrl_c_clears_input() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
        });

        for c in "some text".chars() {
            app.handle_event(&key_event(KeyCode::Char(c), KeyModifiers::NONE))
                .await;
        }
        assert_eq!(app.input.text(), "some text");

        let quit = app
            .handle_event(&key_event(KeyCode::Char('c'), KeyModifiers::CONTROL))
            .await;
        assert!(!quit);
        assert!(app.input.is_empty());
    }

    #[tokio::test]
    async fn workflow_with_default_goal_prefills_input() {
        let mut app = App::new_for_test().await;
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: Some("Review the latest pull request".to_string()),
        });

        assert_eq!(app.mode, AppMode::Workflow);
        assert_eq!(app.input.text(), "Review the latest pull request");
    }
}
