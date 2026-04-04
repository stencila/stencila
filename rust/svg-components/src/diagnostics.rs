/// Severity level for compilation messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Warning,
    Error,
}

/// A diagnostic message produced during overlay compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationMessage {
    pub level: MessageLevel,
    pub message: String,
}

impl CompilationMessage {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: MessageLevel::Error,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: MessageLevel::Warning,
            message: message.into(),
        }
    }
}
