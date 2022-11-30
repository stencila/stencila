use kernels::TaskInfo;
use stencila_schema::{CodeError, ExecutionStatus};

/// Determine the status of an executable code node from kernel `TaskInfo` and list of messages
pub fn code_execution_status(task_info: &TaskInfo, errors: &[CodeError]) -> ExecutionStatus {
    if task_info.was_finished() {
        if errors.is_empty() {
            ExecutionStatus::Succeeded
        } else {
            ExecutionStatus::Failed
        }
    } else if task_info.was_interrupted() {
        ExecutionStatus::Cancelled
    } else if task_info.was_started() {
        ExecutionStatus::Running
    } else {
        ExecutionStatus::Scheduled
    }
}
