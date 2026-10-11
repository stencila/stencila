/// The generic remedy shown when no more specific advice applies.
const DEFAULT_REMEDY: &str =
    "I/O paths resolve from literals, module constants, or single-assignment locals";

/// Why an I/O path expression could not be resolved to a concrete resource.
///
/// Reasons are recorded by the analyzer at the point the resolution attempt
/// fails, so the message shown to an author names the specific binding or call
/// that defeated resolution rather than restating that the path was dynamic.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum UnresolvedIoReason {
    /// The path argument is an expression rather than a string literal.
    NotALiteral,

    /// A template resolved some segments but not all of them.
    PartialTemplate,

    /// An identifier has no binding visible in the enclosing scopes.
    UnboundIdentifier {
        /// Name of the identifier.
        name: String,
    },

    /// An identifier is assigned more than once in its scope.
    MultipleAssignments {
        /// Name of the identifier.
        name: String,
    },

    /// An identifier is assigned inside a branch or loop body.
    ConditionalAssignment {
        /// Name of the identifier.
        name: String,
    },

    /// An identifier is used before its only visible assignment.
    UseBeforeDefinition {
        /// Name of the identifier.
        name: String,
    },

    /// An identifier is bound to an expression rather than a literal value.
    NonLiteralValue {
        /// Name of the identifier.
        name: String,
    },

    /// A parameter reaches the path but its function has no resolvable calls.
    UnresolvedParameter {
        /// Name of the parameter.
        parameter: String,

        /// Name of the function declaring the parameter.
        function: String,
    },

    /// A call could not be bound to exactly one local function declaration.
    AmbiguousCallee {
        /// Callee spelling at the call site.
        callee: String,
    },
}

impl UnresolvedIoReason {
    /// One-line explanation of what defeated resolution.
    pub fn reason(&self) -> String {
        match self {
            Self::NotALiteral => "the path is an expression, not a static literal".to_string(),
            Self::PartialTemplate => {
                "the template has placeholders this pass could not resolve".to_string()
            }
            Self::UnboundIdentifier { name } => {
                format!("`{name}` has no assignment visible in this file")
            }
            Self::MultipleAssignments { name } => {
                format!("`{name}` is assigned more than once in its scope")
            }
            Self::ConditionalAssignment { name } => {
                format!("`{name}` is assigned conditionally, so its value depends on control flow")
            }
            Self::UseBeforeDefinition { name } => {
                format!("`{name}` is used before its only visible assignment")
            }
            Self::NonLiteralValue { name } => {
                format!("`{name}` is assigned an expression rather than a literal value")
            }
            Self::UnresolvedParameter {
                parameter,
                function,
            } => format!(
                "`{parameter}` is a parameter of `{function}`, called with values this pass could not resolve"
            ),
            Self::AmbiguousCallee { callee } => {
                format!("`{callee}` does not resolve to exactly one function declared in this file")
            }
        }
    }

    /// One-line suggestion for making the operation resolvable.
    pub fn remedy(&self) -> String {
        match self {
            Self::PartialTemplate => {
                "assign each interpolated value to a module constant or single-assignment local"
                    .to_string()
            }
            Self::MultipleAssignments { name } | Self::ConditionalAssignment { name } => {
                format!("assign `{name}` exactly once, unconditionally, before it is used")
            }
            Self::UseBeforeDefinition { name } => {
                format!("move the assignment of `{name}` above the I/O call")
            }
            Self::NonLiteralValue { name } => {
                format!("assign `{name}` from a literal, or pass the literal at the call site")
            }
            Self::UnresolvedParameter { function, .. } => {
                format!("call `{function}` with literal or constant arguments")
            }
            _ => DEFAULT_REMEDY.to_string(),
        }
    }
}

/// A position in the original source: zero-based lines/Unicode scalar columns,
/// and a UTF-8 byte offset. Positions never refer to host files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// A source span. The end is null when the analyzer knows only an operation's start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: Option<SourcePosition>,
}

/// Convert an offset to a position; invalid UTF-8 boundaries return none.
pub fn source_position(source: &str, offset: usize) -> Option<SourcePosition> {
    let prefix = source.get(..offset)?;
    Some(SourcePosition {
        line: prefix.bytes().filter(|byte| *byte == b'\n').count(),
        column: prefix
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .count(),
        offset,
    })
}

/// One unresolved operation, without document or schema dependencies.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnresolvedIoDiagnostic {
    pub direction: crate::IoDirection,
    pub expression: String,
    pub offset: Option<usize>,
    pub location: Option<SourceSpan>,
    pub function: Option<String>,
    pub mode: Option<crate::IoMode>,
    pub known_segments: Vec<String>,
    /// Owning workflow unit, or null for whole-file facts.
    pub workflow_unit: Option<String>,
    pub reason: UnresolvedIoReason,
    pub message: String,
    pub remedy: String,
}

/// Structured diagnostics for the public JSON contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AnalysisDiagnostic {
    /// Parsing failed; facts are otherwise empty. The parser does not retain a span.
    SyntaxError,
    /// An operation's path could not be proven statically.
    UnresolvedIo(UnresolvedIoDiagnostic),
}

/// Shared by the file API and graph's document-specific diagnostic adapter.
pub fn unresolved_io_diagnostics(
    facts: &crate::CodeFacts,
    source: Option<&str>,
) -> Vec<UnresolvedIoDiagnostic> {
    let whole_file = facts.io.iter().map(|fact| (None, fact));
    let units = facts
        .workflow_unit_facts
        .iter()
        .flat_map(|(name, unit)| unit.io.iter().map(move |fact| (Some(name), fact)));
    whole_file
        .chain(units)
        .filter_map(|(unit, fact)| {
            if fact.path.is_static() {
                return None;
            }
            let reason = fact.unresolved_reason.clone().unwrap_or(match fact.path {
                crate::IoPath::Template(..) => UnresolvedIoReason::PartialTemplate,
                _ => UnresolvedIoReason::NotALiteral,
            });
            Some(UnresolvedIoDiagnostic {
                direction: fact.direction,
                expression: fact.path.value().to_string(),
                offset: fact.operation_offset,
                location: source
                    .zip(fact.operation_offset)
                    .and_then(|(source, offset)| {
                        source_position(source, offset).map(|start| SourceSpan { start, end: None })
                    }),
                function: fact.function.clone(),
                mode: fact.mode,
                known_segments: fact.path.resolved_segments(),
                workflow_unit: unit.cloned(),
                message: reason.reason(),
                remedy: reason.remedy(),
                reason,
            })
        })
        .collect()
}

pub(crate) fn collect_diagnostics(
    facts: &crate::CodeFacts,
    source: Option<&str>,
) -> Vec<AnalysisDiagnostic> {
    if facts.syntax_error {
        return vec![AnalysisDiagnostic::SyntaxError];
    }
    unresolved_io_diagnostics(facts, source)
        .into_iter()
        .map(AnalysisDiagnostic::UnresolvedIo)
        .collect()
}
