//! Author-facing diagnostics for I/O that static analysis could not resolve.
//!
//! The analyzer is deliberately conservative: an I/O path that cannot be proven
//! from the source is not turned into a resource node. Silence about that
//! decision is the problem this module solves. Every unresolved I/O operation
//! produces a structured diagnostic carrying where it is, what could not be
//! resolved, why, and what would make it resolvable.
//!
//! Diagnostics are a sidecar result rather than graph content. They travel
//! beside the [`crate::Graph`] in [`crate::GraphAnalysis`] so library callers and
//! `stencila graph --explain` can report them without graph consumers having to
//! interpret analyzer output encoded as resource nodes.

use std::fmt::{self, Display};

use eyre::Result;
use stencila_codecs::Format;
use stencila_node_diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel};
use stencila_schema::{CodeLocation, NodeId, NodeProperty, NodeType};

use super::facts::{IoDirection, IoMode};

pub use stencila_code_analysis::UnresolvedIoReason;

/// One unresolved I/O operation reported to the author.
///
/// The diagnostic keeps everything needed to render an actionable message
/// without re-reading the source: a source location, the expression that could
/// not be resolved, any template segments that were resolved, the reason, and a
/// remedy.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticAnalysisDiagnostic {
    /// Common diagnostic fields and rendering behavior.
    pub diagnostic: Diagnostic,

    /// Whether the operation reads, writes, or both.
    pub direction: IoDirection,

    /// Source text of the unresolved path expression.
    pub expression: String,

    /// Byte offset of the operation within the code unit, when known.
    pub offset: Option<usize>,

    /// I/O function or method name, when known.
    pub function: Option<String>,

    /// File mode, when statically visible.
    pub mode: Option<IoMode>,

    /// Template segments that were resolved, in source order.
    ///
    /// A partially resolved URL template keeps its concrete prefix here even
    /// though the operation produces no resource node. That is often enough to
    /// identify the upstream service.
    pub known_segments: Vec<String>,
}

impl StaticAnalysisDiagnostic {
    /// Build a diagnostic for an unresolved I/O operation.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        node_type: NodeType,
        node_id: Option<&NodeId>,
        scope: &str,
        language: &str,
        source_text: Option<&str>,
        direction: IoDirection,
        expression: &str,
        offset: Option<usize>,
        function: Option<String>,
        mode: Option<IoMode>,
        known_segments: Vec<String>,
        reason: &UnresolvedIoReason,
    ) -> Self {
        let (line, column) = match (source_text, offset) {
            (Some(text), Some(offset)) => line_and_column(text, offset),
            _ => (None, None),
        };
        let mut code_location = CodeLocation::new();
        code_location.source = Some(scope.to_string());
        code_location.start_line = line
            .and_then(|line| line.checked_sub(1))
            .map(|line| line as u64);
        code_location.start_column = column
            .and_then(|column| column.checked_sub(1))
            .map(|column| column as u64);
        let message = reason.reason();
        let help = reason.remedy();
        let mut notes = vec![format!("expression: {expression}")];
        if !known_segments.is_empty() {
            notes.push(format!("resolved: {}", known_segments.join(" … ")));
        }
        let operation = match direction {
            IoDirection::Read => "read",
            IoDirection::Write => "write",
            IoDirection::ReadWrite => "read/write",
        };

        Self {
            diagnostic: Diagnostic {
                node_type,
                node_id: node_id.cloned().unwrap_or_else(NodeId::null),
                node_property: Some(NodeProperty::Code),
                level: DiagnosticLevel::Advice,
                kind: DiagnosticKind::StaticAnalysis,
                error_type: Some(format!("unresolved {operation}")),
                message,
                help: Some(help),
                notes,
                format: Some(Format::from_name(language)),
                code: source_text.map(str::to_string),
                code_location: Some(code_location),
            },
            direction,
            expression: expression.to_string(),
            offset,
            function,
            mode,
            known_segments,
        }
    }

    /// Scope of the code unit, usually a workspace-relative path.
    pub fn scope(&self) -> &str {
        self.diagnostic
            .code_location
            .as_ref()
            .and_then(|location| location.source.as_deref())
            .unwrap_or("<code>")
    }

    /// One-based line number within the code unit, when known.
    pub fn line(&self) -> Option<usize> {
        self.diagnostic
            .code_location
            .as_ref()
            .and_then(|location| location.start_line)
            .map(|line| line as usize + 1)
    }

    /// Why the path could not be resolved.
    pub fn reason(&self) -> &str {
        &self.diagnostic.message
    }

    /// What would make the path resolvable.
    pub fn remedy(&self) -> Option<&str> {
        self.diagnostic.help.as_deref()
    }

    /// Render this diagnostic to stderr using the shared diagnostic renderer.
    pub fn to_stderr(self) -> Result<()> {
        let path = self.scope().to_string();
        let source = self.diagnostic.code.clone().unwrap_or_default();
        self.diagnostic.to_stderr(&path, &source, &None)
    }

    /// Short label for the operation direction.
    pub fn operation(&self) -> &'static str {
        match self.direction {
            IoDirection::Read => "read",
            IoDirection::Write => "write",
            IoDirection::ReadWrite => "read/write",
        }
    }
}

impl Display for StaticAnalysisDiagnostic {
    /// Render the diagnostic in the `--explain` text format.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.scope())?;
        if let Some(line) = self.line() {
            write!(formatter, ":{line}")?;
        }
        writeln!(formatter, "  unresolved {}", self.operation())?;
        writeln!(formatter, "  {}", self.expression)?;
        if !self.known_segments.is_empty() {
            writeln!(formatter, "  resolved: {}", self.known_segments.join(" … "))?;
        }
        writeln!(formatter, "  {}", self.reason())?;
        if let Some(remedy) = self.remedy() {
            write!(formatter, "  {remedy}")?;
        }
        Ok(())
    }
}

/// Convert a byte offset into one-based line and column numbers.
fn line_and_column(text: &str, offset: usize) -> (Option<usize>, Option<usize>) {
    match stencila_code_analysis::integration::source_position(text, offset) {
        Some(position) => (Some(position.line + 1), Some(position.column + 1)),
        None => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_line_and_column() {
        assert_eq!(line_and_column("abc", 0), (Some(1), Some(1)));
        assert_eq!(line_and_column("abc\ndef", 4), (Some(2), Some(1)));
        assert_eq!(line_and_column("abc\ndef", 6), (Some(2), Some(3)));
        assert_eq!(line_and_column("abc", 99), (None, None));
    }

    #[test]
    fn formats_explain_output() {
        let diagnostic = StaticAnalysisDiagnostic::new(
            NodeType::SoftwareSourceCode,
            None,
            "download.py",
            "python",
            Some("\n".repeat(35).as_str()),
            IoDirection::Read,
            "urlopen(request)",
            Some(35),
            Some("urlopen".to_string()),
            None,
            Vec::new(),
            &UnresolvedIoReason::UnresolvedParameter {
                parameter: "request".to_string(),
                function: "fetch".to_string(),
            },
        );

        let text = diagnostic.to_string();
        assert!(text.starts_with("download.py:36  unresolved read\n"));
        assert!(text.contains("  urlopen(request)\n"));
        assert!(text.contains("`request` is a parameter of `fetch`"));
        assert!(text.ends_with("call `fetch` with literal or constant arguments"));
        assert_eq!(diagnostic.diagnostic.kind, DiagnosticKind::StaticAnalysis);
        assert_eq!(
            diagnostic.diagnostic.notes,
            ["expression: urlopen(request)"]
        );
    }
}
