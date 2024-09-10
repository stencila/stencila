use crate::{CompilationMessage, ExecutionMessage};

impl CompilationMessage {
    /// Format the compilation message, including error type if present
    pub fn formatted(&self) -> String {
        let prefix = self.error_type.as_ref().map_or_else(
            || self.level.to_string(),
            |error_type| error_type.to_string(),
        );

        [&prefix, ": ", &self.message].concat()
    }
}
impl From<ExecutionMessage> for CompilationMessage {
    fn from(message: ExecutionMessage) -> Self {
        CompilationMessage {
            level: message.level,
            message: message.message,
            error_type: message.error_type,
            code_location: message.code_location,
            ..Default::default()
        }
    }
}
