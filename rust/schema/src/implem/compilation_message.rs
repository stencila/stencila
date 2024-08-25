use crate::CompilationMessage;

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
