use kernels::{TaskInfo};
use stencila_schema::{CodeError, ExecuteStatus};

/// Determine the status of an executable code node from kernel `TaskInfo` and list of messages
pub fn code_execute_status(task_info: &TaskInfo, errors: &[CodeError]) -> ExecuteStatus {
    if task_info.was_finished() {
        if errors.is_empty() {
            ExecuteStatus::Succeeded
        } else {
            ExecuteStatus::Failed
        }
    } else if task_info.was_interrupted() {
        ExecuteStatus::Cancelled
    } else if task_info.was_started() {
        ExecuteStatus::Running
    } else {
        ExecuteStatus::Scheduled
    }
}
