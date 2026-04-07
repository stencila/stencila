/// Severity level for compilation messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Warning,
    Error,
}

/// A diagnostic message produced during overlay compilation or linting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationMessage {
    pub level: MessageLevel,
    pub message: String,
    /// 0-based line number in the SVG source.
    pub start_line: Option<u64>,
    /// 0-based column number in the SVG source.
    pub start_column: Option<u64>,
}

impl CompilationMessage {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: MessageLevel::Error,
            message: message.into(),
            start_line: None,
            start_column: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: MessageLevel::Warning,
            message: message.into(),
            start_line: None,
            start_column: None,
        }
    }

    /// Attach source location (0-based line and column).
    #[must_use]
    pub fn at(mut self, line: u64, column: u64) -> Self {
        self.start_line = Some(line);
        self.start_column = Some(column);
        self
    }

    /// Attach source location from a byte offset in the source string.
    #[must_use]
    pub fn at_offset(mut self, source: &str, byte_offset: usize) -> Self {
        let (line, col) = byte_offset_to_line_col(source, byte_offset);
        self.start_line = Some(line);
        self.start_column = Some(col);
        self
    }
}

/// Convert a byte offset into a (0-based line, 0-based column) pair.
#[must_use]
pub fn byte_offset_to_line_col(source: &str, offset: usize) -> (u64, u64) {
    let offset = offset.min(source.len());
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|&b| b == b'\n').count() as u64;
    let last_newline = prefix.rfind('\n').map_or(0, |pos| pos + 1);
    let col = (offset - last_newline) as u64;
    (line, col)
}
