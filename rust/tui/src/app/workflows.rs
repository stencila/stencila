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
            pending_interview: None,
            current_exchange_msg_index: None,
            current_stage_progress: None,
            workflow_status_msg_index: None,
            stage_status_msg_index: None,
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
    fn handle_pipeline_event_minimal(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                let msg_index = self.messages.len();

                self.messages.push(AppMessage::WorkflowStatus {
                    state: WorkflowStatusState::Running,
                    label: format!("Workflow {pipeline_name}"),
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
    #[allow(clippy::too_many_lines)]
    fn handle_pipeline_event_simple(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Started,
                    label: format!("Workflow {pipeline_name} started"),
                    detail: None,
                });
            }
            PipelineEvent::StageStarted {
                node_id,
                stage_index,
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
                    label: format!("Workflow {pipeline_name} completed"),
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
                    label: format!("Workflow {pipeline_name} failed"),
                    detail: Some(reason.to_string()),
                });
            }
            // Suppress parallel, interview, checkpoint, text delta, response end
            _ => {}
        }
    }

    /// Detailed mode: full exchange per stage with prompt + streaming response.
    #[allow(clippy::too_many_lines)]
    fn handle_pipeline_event_detailed(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                self.messages.push(AppMessage::WorkflowProgress {
                    kind: WorkflowProgressKind::Started,
                    label: format!("Workflow {pipeline_name} started"),
                    detail: None,
                });
            }
            PipelineEvent::StageStarted {
                node_id,
                stage_index,
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
                });
                if let Some(workflow) = &mut self.active_workflow {
                    workflow.current_exchange_msg_index = Some(msg_index);
                    workflow.current_stage_progress =
                        Some(Arc::new(Mutex::new(AgentProgress::default())));
                }
            }
            PipelineEvent::StageInput {
                agent_name, input, ..
            } => {
                self.color_registry.color_for(agent_name);
                if let Some(msg_index) = self
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
            PipelineEvent::StageSessionEvent { event: se, .. } => {
                if let Some(workflow) = &self.active_workflow
                    && let Some(ref progress) = workflow.current_stage_progress
                {
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
            // PipelineEvent::StageOutput: response already built incrementally from StageSessionEvent
            PipelineEvent::StageCompleted { outcome, .. } => {
                if let Some(workflow) = &mut self.active_workflow {
                    if let Some(msg_index) = workflow.current_exchange_msg_index.take()
                        && let Some(AppMessage::Exchange { status, .. }) =
                            self.messages.get_mut(msg_index)
                    {
                        *status = if outcome.status.is_success() {
                            ExchangeStatus::Succeeded
                        } else {
                            ExchangeStatus::Failed
                        };
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
                    label: format!("Workflow {pipeline_name} completed"),
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
                    label: format!("Workflow {pipeline_name} failed"),
                    detail: Some(reason.to_string()),
                });
            }
            // Suppress parallel, interview, checkpoint, text delta, response end
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
