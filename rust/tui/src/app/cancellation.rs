use crate::shell::RunningShellCommand;

use super::{
    ActiveWorkflowState, App, AppMessage, ExchangeKind, ExchangeStatus, WorkflowProgressKind,
    WorkflowStatusState,
};

impl App {
    /// Cancel a running command or agent exchange by its message index.
    ///
    /// Searches `running_shell_commands` and all sessions'
    /// `running_agent_exchanges` for the matching `msg_index`, removes it,
    /// and marks the exchange as cancelled.
    pub fn cancel_by_msg_index(&mut self, msg_index: usize) {
        if let Some(pos) = self
            .running_shell_commands
            .iter()
            .position(|(idx, _)| *idx == msg_index)
        {
            let entry = self.running_shell_commands.remove(pos);
            Self::cancel_entry(&mut self.messages, entry);
            return;
        }

        for session in &mut self.sessions {
            if let Some(pos) = session
                .running_agent_exchanges
                .iter()
                .position(|(idx, _)| *idx == msg_index)
            {
                let (idx, exchange) = session.running_agent_exchanges.remove(pos);
                exchange.cancel();
                Self::mark_exchange_cancelled(&mut self.messages, idx);
                return;
            }
        }
    }

    /// Cancel the most recent running command or agent exchange.
    ///
    /// Compares the highest message index across all sessions'
    /// `running_agent_exchanges` and `running_shell_commands`.
    pub(super) fn cancel_most_recent_running(&mut self) {
        // If a workflow is running, cancel it first (it's the most prominent running task).
        if self
            .active_workflow
            .as_ref()
            .is_some_and(|w| w.run_handle.is_some())
        {
            self.cancel_active_workflow();
            return;
        }

        let cmd_max = self.running_shell_commands.last().map(|(idx, _)| *idx);
        let agent_max = self
            .sessions
            .iter()
            .filter_map(|s| s.running_agent_exchanges.last())
            .map(|(idx, _)| *idx)
            .max();

        match (cmd_max, agent_max) {
            (Some(c), Some(a)) if a > c => {
                // Find which session has that max agent exchange
                for session in &mut self.sessions {
                    if session
                        .running_agent_exchanges
                        .last()
                        .is_some_and(|(idx, _)| *idx == a)
                    {
                        if let Some((idx, exchange)) = session.running_agent_exchanges.pop() {
                            exchange.cancel();
                            Self::mark_exchange_cancelled(&mut self.messages, idx);
                        }
                        break;
                    }
                }
            }
            (Some(_), _) => {
                if let Some(entry) = self.running_shell_commands.pop() {
                    Self::cancel_entry(&mut self.messages, entry);
                }
            }
            (None, Some(_)) => {
                // Find which session has the max agent exchange
                if let Some(a) = agent_max {
                    for session in &mut self.sessions {
                        if session
                            .running_agent_exchanges
                            .last()
                            .is_some_and(|(idx, _)| *idx == a)
                        {
                            if let Some((idx, exchange)) = session.running_agent_exchanges.pop() {
                                exchange.cancel();
                                Self::mark_exchange_cancelled(&mut self.messages, idx);
                            }
                            break;
                        }
                    }
                }
            }
            (None, None) => {}
        }
    }

    /// Cancel all running shell commands, agent exchanges, and workflows.
    pub fn cancel_all_running(&mut self) {
        for entry in self.running_shell_commands.drain(..) {
            Self::cancel_entry(&mut self.messages, entry);
        }
        for session in &mut self.sessions {
            for (idx, exchange) in session.running_agent_exchanges.drain(..) {
                exchange.cancel();
                Self::mark_exchange_cancelled(&mut self.messages, idx);
            }
        }
        self.cancel_active_workflow();
    }

    /// Cancel the active workflow if one is running.
    pub(super) fn cancel_active_workflow(&mut self) {
        if let Some(workflow) = &mut self.active_workflow
            && let Some(handle) = workflow.run_handle.take()
        {
            handle.abort();
            let name = workflow.info.name.clone();
            // Mark all workflow progress/status messages with spinners as cancelled
            for msg in &mut self.messages {
                match msg {
                    AppMessage::WorkflowStatus { state: phase, .. }
                        if *phase == WorkflowStatusState::Running =>
                    {
                        *phase = WorkflowStatusState::Cancelled;
                    }
                    AppMessage::WorkflowProgress { kind, .. }
                        if *kind == WorkflowProgressKind::Running =>
                    {
                        *kind = WorkflowProgressKind::Cancelled;
                    }
                    AppMessage::Exchange {
                        kind: ExchangeKind::Workflow,
                        status,
                        ..
                    } if *status == ExchangeStatus::Running => {
                        *status = ExchangeStatus::Cancelled;
                    }
                    _ => {}
                }
            }
            self.messages.push(AppMessage::System {
                content: format!("Workflow '{name}' cancelled"),
            });
        }
        if let Some(workflow) = &mut self.active_workflow {
            workflow.state = ActiveWorkflowState::Cancelled;
        }
    }

    /// Cancel a single running command and mark its exchange as cancelled.
    pub(super) fn cancel_entry(
        messages: &mut [AppMessage],
        (msg_index, running): (usize, RunningShellCommand),
    ) {
        let _command = running.cancel();
        Self::update_exchange_at(
            messages,
            msg_index,
            ExchangeStatus::Cancelled,
            Some("[cancelled]".to_string()),
            Some(-1),
        );
    }

    /// Update the exchange at `msg_index` in-place.
    ///
    /// Clears `response_segments` since this is used for cancellation and
    /// shell command completion which produce only plain text.
    pub(super) fn update_exchange_at(
        messages: &mut [AppMessage],
        msg_index: usize,
        new_status: ExchangeStatus,
        response: Option<String>,
        exit_code: Option<i32>,
    ) {
        if let Some(AppMessage::Exchange {
            status,
            response: resp,
            response_segments,
            exit_code: ec,
            ..
        }) = messages.get_mut(msg_index)
        {
            *status = new_status;
            *resp = response;
            *response_segments = None;
            *ec = exit_code;
        }
    }

    /// Mark the exchange at `msg_index` as cancelled without replacing its response.
    pub(super) fn mark_exchange_cancelled(messages: &mut [AppMessage], msg_index: usize) {
        if let Some(AppMessage::Exchange { status, .. }) = messages.get_mut(msg_index) {
            *status = ExchangeStatus::Cancelled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ActiveWorkflowState, App};
    use crate::autocomplete::workflows::WorkflowDefinitionInfo;

    #[tokio::test]
    async fn cancel_workflow_keeps_active_workflow() {
        let mut app = App::new_for_test();
        app.activate_workflow(WorkflowDefinitionInfo {
            name: "test-wf".to_string(),
            description: String::new(),
            goal: None,
        });
        // Manually set state to Running to simulate an active workflow
        if let Some(wf) = &mut app.active_workflow {
            wf.state = ActiveWorkflowState::Running;
        }

        app.cancel_active_workflow();

        assert!(app.active_workflow.is_some());
        assert_eq!(
            app.active_workflow.as_ref().expect("checked above").state,
            ActiveWorkflowState::Cancelled
        );
    }
}
