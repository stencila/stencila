use crate::{CompilationMessage, ExecutionMessage};

impl ExecutionMessage {
    /// Format the execution message, including error type and stack trace if present
    pub fn formatted(&self) -> String {
        let prefix = self.error_type.as_ref().map_or_else(
            || self.level.to_string(),
            |error_type| error_type.to_string(),
        );

        let mut formatted = [&prefix, ": ", &self.message].concat();

        if let Some(stack_trace) = self.stack_trace.as_ref() {
            formatted.push_str("\n\n");
            formatted.push_str(stack_trace);
        };

        formatted
    }
}

impl From<CompilationMessage> for ExecutionMessage {
    fn from(message: CompilationMessage) -> Self {
        ExecutionMessage {
            level: message.level,
            message: message.message,
            error_type: message.error_type,
            code_location: message.code_location,
            ..Default::default()
        }
    }
}
