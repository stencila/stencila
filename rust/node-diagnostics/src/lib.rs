use std::{env, ops::Range};

use ariadne::{Config, Label, Report, ReportKind, Source};
use eyre::Result;
use serde::Serialize;
use serde_with::skip_serializing_none;
use strum::Display;

use stencila_codec_info::{PoshMap, Position8, Positions, Range8};
use stencila_format::Format;
use stencila_schema::{
    Bibliography, Block, Citation, CodeLocation, CompilationMessage, ExecutionMessage,
    IfBlockClause, Inline, MessageLevel, Node, NodeId, NodeProperty, NodeType, Visitor,
    WalkControl, WalkNode,
};

/// Collect all diagnostic messages from a node
///
/// Currently, collects the [`CompilationMessage`] and [`ExecutionMessage`]s
/// from on, and within, the node. In the future, additional diagnostics
/// not related to executable nodes, (e.g verification of internal and external links)
/// may be added.
pub fn diagnostics<T>(node: &T) -> Vec<Diagnostic>
where
    T: WalkNode,
{
    let mut walker = Collector::default();
    walker.walk(node);
    walker.messages
}

/// Collect all diagnostic messages with at least a given level
pub fn diagnostics_gte<T>(node: &T, level: DiagnosticLevel) -> Vec<Diagnostic>
where
    T: WalkNode,
{
    diagnostics(node)
        .into_iter()
        .filter(|diagnostic| diagnostic.level >= level)
        .collect()
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// The type of node that the diagnostic is for
    pub node_type: NodeType,

    /// The id of the node that the diagnostic is for
    pub node_id: NodeId,

    /// The property associated with the diagnostic
    ///
    /// Used for more accurate position of the diagnostic is code
    pub node_property: Option<NodeProperty>,

    /// The severity level of the diagnostic
    pub level: DiagnosticLevel,

    /// The kind of diagnostic
    pub kind: DiagnosticKind,

    /// The error type, if any, of the diagnostic
    pub error_type: Option<String>,

    /// The diagnostic's message
    pub message: String,

    /// Suggested action that may resolve the diagnostic
    pub help: Option<String>,

    /// Additional context for the diagnostic
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,

    /// The format / programming language associated with the diagnostic
    pub format: Option<Format>,

    /// The source code associated with the diagnostic
    pub code: Option<String>,

    /// The location of the diagnostic within the code
    pub code_location: Option<CodeLocation>,
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum DiagnosticLevel {
    /// An advisory diagnostic
    Advice,
    /// A warning diagnostic
    Warning,
    /// An error diagnostic
    Error,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum DiagnosticKind {
    Linting,
    Compilation,
    Execution,
    StaticAnalysis,
}

impl From<&MessageLevel> for DiagnosticLevel {
    fn from(value: &MessageLevel) -> Self {
        match value {
            MessageLevel::Warning => DiagnosticLevel::Warning,
            MessageLevel::Error | MessageLevel::Exception => DiagnosticLevel::Error,
            _ => DiagnosticLevel::Advice,
        }
    }
}

impl Diagnostic {
    /// Get the diagnostics level
    pub fn level(&self) -> DiagnosticLevel {
        self.level
    }

    /// Get the diagnostics message text
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Generate a title for the diagnostic
    fn title(&self) -> String {
        let mut details = String::new();
        if let Some(format) = &self.format {
            details.push_str(format.name());
            details.push(' ');
        }
        details.push_str(&self.node_type.to_string());
        details.push(' ');
        details.push_str(
            &self
                .error_type
                .clone()
                .unwrap_or_else(|| self.level.to_string().to_lowercase()),
        );

        details
    }

    /// Get the [`Range8`] for the node from a [`PoshMap`].
    fn node_range8<'s>(&self, poshmap: &PoshMap<'s, 's>) -> Option<Range8> {
        if let Some(node_property) = self.node_property {
            poshmap
                .node_property_to_range8(&self.node_id, node_property)
                .or_else(|| poshmap.node_id_to_range8(&self.node_id))
        } else {
            poshmap.node_id_to_range8(&self.node_id)
        }
    }

    /// Get the source range for this diagnostic.
    ///
    /// Node diagnostics use the node range from a PoshMap as their base and
    /// shift a relative code location into it. Standalone diagnostics, such as
    /// static analysis of source files, use their code location directly.
    fn source_range8<'s>(&self, poshmap: Option<&PoshMap<'s, 's>>) -> Option<Range8> {
        let code_range = self.code_location.as_ref().and_then(code_location_range8);

        let Some(poshmap) = poshmap else {
            return code_range;
        };
        match (self.node_range8(poshmap), code_range) {
            (Some(node), Some(code)) => Some(shift_range8(node.start, code)),
            (Some(node), None) => Some(node),
            // A code location attached to a node is relative to that node.
            // Without its PoshMap range it cannot safely become an absolute
            // location in the containing document.
            (None, _) => None,
        }
    }

    /// Print the diagnostic to stderr
    ///         
    /// If on GitHub message prints both a CI message and the pretty display so
    /// as to provide useful diagnostics output in various contexts.    
    pub fn to_stderr<'s>(
        self,
        path: &'s str,
        source: &'s str,
        poshmap: &Option<PoshMap<'s, 's>>,
    ) -> Result<()> {
        if env::var_os("GITHUB_ACTIONS").is_some() {
            self.to_stderr_github_message(path, poshmap);
        }

        self.to_stderr_pretty(path, source, poshmap)
    }

    /// Pretty print the diagnostic to a string
    ///
    /// Similar `to_stderr_pretty` but returns a string without terminal color codes
    /// and that is more compact.
    pub fn to_string_pretty<'s>(
        self,
        path: &'s str,
        source: &'s str,
        poshmap: &Option<PoshMap<'s, 's>>,
    ) -> Result<String> {
        let (report, source) = self.into_report(path, source, poshmap, false, true)?;
        let cache = (path, Source::from(source));

        let mut buffer = Vec::new();
        report.write(cache, &mut buffer)?;
        let string = String::from_utf8(buffer)?;

        Ok(string)
    }

    /// Pretty print the diagnostic to stderr
    #[allow(clippy::wrong_self_convention)]
    fn to_stderr_pretty<'s>(
        self,
        path: &'s str,
        source: &'s str,
        poshmap: &Option<PoshMap<'s, 's>>,
    ) -> Result<()> {
        let (report, source) = self.into_report(path, source, poshmap, true, false)?;
        let cache = (path, Source::from(source));

        report.eprint(cache)?;

        Ok(())
    }

    /// Print the diagnostic to stderr as a GitHub Action message
    ///         
    /// This is beneficial because the diagnostic will be displayed more
    /// prominently in the action summary and in-line in PR file diffs.
    ///
    /// https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands#setting-a-notice-message
    #[allow(clippy::print_stderr)]
    fn to_stderr_github_message<'s>(&self, path: &str, poshmap: &Option<PoshMap<'s, 's>>) {
        let type_ = match self.level {
            DiagnosticLevel::Advice => "notice",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Error => "error",
        };

        let mut message = ["::", type_, " file=", path].concat();

        if let Some(Range8 { start, end }) = self.source_range8(poshmap.as_ref()) {
            message.push_str(",line=");
            message.push_str(&(1 + start.line).to_string());
            message.push_str(",endLine=");
            message.push_str(&(1 + end.line).to_string());
            message.push_str(",col=");
            message.push_str(&(1 + start.column).to_string());
            message.push_str(",endColumn=");
            message.push_str(&(1 + end.column).to_string());
        }

        message.push_str(",title=");
        message.push_str(&self.title());

        message.push_str("::");
        message.push_str(&self.message);

        eprintln!("{message}");
    }

    #[allow(clippy::type_complexity)]
    fn into_report<'s>(
        self,
        path: &'s str,
        source: &'s str,
        poshmap: &Option<PoshMap<'s, 's>>,
        color: bool,
        compact: bool,
    ) -> Result<(Report<'s, (&'s str, Range<usize>)>, String)> {
        let kind: ReportKind<'_> = match self.level {
            DiagnosticLevel::Advice => ReportKind::Advice,
            DiagnosticLevel::Warning => ReportKind::Warning,
            DiagnosticLevel::Error => ReportKind::Error,
        };

        let title = self.title();

        // Decide if using the document's source or the message's source
        let source = if !source.is_empty() {
            source.to_string()
        } else if let Some(code) = &self.code {
            code.to_string()
        } else {
            String::new()
        };

        // Create a mapping between source line/column position and character index
        let positions = Positions::new(&source);

        // Convert line/column range to character range
        let range = if let Some(range8) = self.source_range8(poshmap.as_ref()) {
            let start = positions.index_at_position8(range8.start).unwrap_or(0);
            let end = positions
                .index_at_position8(range8.end)
                .unwrap_or(start)
                .max(start);
            start..end
        } else {
            0..0
        };

        let mut report = Report::build(kind, (path, range.clone()))
            .with_message(&title)
            .with_label(Label::new((path, range)).with_message(self.message))
            .with_config(Config::new().with_color(color).with_compact(compact));
        if let Some(help) = self.help {
            report = report.with_help(help);
        }
        for note in self.notes {
            report = report.with_note(note);
        }

        Ok((report.finish(), source))
    }
}

/// Convert a direct code location to a source range.
fn code_location_range8(location: &CodeLocation) -> Option<Range8> {
    if location.start_line.is_none()
        && location.start_column.is_none()
        && location.end_line.is_none()
        && location.end_column.is_none()
    {
        return None;
    }

    let start_line = usize::try_from(location.start_line.unwrap_or(0)).ok()?;
    let start_column = usize::try_from(location.start_column.unwrap_or(0)).ok()?;
    let end_line = location
        .end_line
        .map_or(Some(start_line), |line| usize::try_from(line).ok())?;
    let mut end_column = location
        .end_column
        .map_or(Some(start_column), |column| usize::try_from(column).ok())?;
    if end_line == start_line && end_column == start_column {
        end_column += 1;
    }

    Some(Range8::new(
        Position8::new(start_line, start_column),
        Position8::new(end_line, end_column),
    ))
}

/// Shift a code-relative range by the start of its containing node.
fn shift_range8(base: Position8, range: Range8) -> Range8 {
    let shift = |position: Position8| {
        Position8::new(
            base.line + position.line,
            if position.line == 0 {
                base.column + position.column
            } else {
                position.column
            },
        )
    };

    Range8::new(shift(range.start), shift(range.end))
}

/// A visitor that walks over a node and collects any messages
#[derive(Default)]
struct Collector {
    /// The collected messages
    messages: Vec<Diagnostic>,

    /// The node id and file name of any included, or called, file
    ///
    /// Used to locate diagnostics properly to the top level include.
    /// At this stage we are unable to to provide more precise location within
    /// included file.
    within: Option<(NodeId, String)>,
}

impl Collector {
    /// Collect the [`CompilationMessage`]s from a node
    fn compilation_messages(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        node_property: &Option<NodeProperty>,
        messages: &Option<Vec<CompilationMessage>>,
        lang: Option<&str>,
        code: Option<&str>,
    ) {
        let (node_id, prefix) = if let Some((node_id, source)) = &self.within {
            (node_id.clone(), format!("Within `{source}`: "))
        } else {
            (node_id, String::new())
        };

        let mut msgs = messages
            .iter()
            .flatten()
            .map(|msg| {
                let kind = if msg.error_type.as_deref() == Some("Linting") {
                    DiagnosticKind::Linting
                } else {
                    DiagnosticKind::Compilation
                };

                Diagnostic {
                    node_type,
                    node_id: node_id.clone(),
                    node_property: *node_property,
                    level: DiagnosticLevel::from(&msg.level),
                    kind,
                    error_type: msg.error_type.clone(),
                    message: format!("{}{}", prefix, msg.message),
                    help: None,
                    notes: Vec::new(),
                    format: lang.map(Format::from_name),
                    code: code.map(|cord| cord.to_string()),
                    code_location: msg.code_location.clone(),
                }
            })
            .collect();
        self.messages.append(&mut msgs)
    }

    /// Collect the [`ExecutionMessage`]s from a node
    fn execution_messages(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        node_property: &Option<NodeProperty>,
        messages: &Option<Vec<ExecutionMessage>>,
        lang: Option<&str>,
        code: Option<&str>,
    ) {
        let (node_id, prefix) = if let Some((node_id, source)) = &self.within {
            (node_id.clone(), format!("Within `{source}`: "))
        } else {
            (node_id, String::new())
        };

        let mut msgs = messages
            .iter()
            .flatten()
            .map(|msg| Diagnostic {
                node_type,
                node_id: node_id.clone(),
                node_property: *node_property,
                level: DiagnosticLevel::from(&msg.level),
                kind: DiagnosticKind::Execution,
                error_type: msg.error_type.clone(),
                message: format!("{}{}", prefix, msg.message),
                help: None,
                notes: Vec::new(),
                format: lang.map(Format::from_name),
                code: code.map(|cord| cord.to_string()),
                code_location: msg.code_location.clone(),
            })
            .collect();
        self.messages.append(&mut msgs)
    }

    /// Collect the [`CompilationMessage`]s and [`ExecutionMessage`]s from a node
    #[allow(clippy::too_many_arguments)]
    fn compilation_and_execution_messages(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        node_property: &Option<NodeProperty>,
        compilation_messages: &Option<Vec<CompilationMessage>>,
        execution_messages: &Option<Vec<ExecutionMessage>>,
        lang: Option<&str>,
        code: Option<&str>,
    ) {
        self.compilation_messages(
            node_type,
            node_id.clone(),
            node_property,
            compilation_messages,
            lang,
            code,
        );
        self.execution_messages(
            node_type,
            node_id,
            node_property,
            execution_messages,
            lang,
            code,
        );
    }

    /// Visit a [`Bibliography`] node and collect its compilation messages
    fn visit_bibliography(&mut self, bibliography: &Bibliography) {
        self.compilation_messages(
            bibliography.node_type(),
            bibliography.node_id(),
            &Some(NodeProperty::Source),
            &bibliography.options.compilation_messages,
            None,
            None,
        );
    }
}

macro_rules! cms {
    ($self:expr, $node:expr, $prop:expr, $lang:expr, $code:expr) => {{
        $self.compilation_messages(
            $node.node_type(),
            $node.node_id(),
            &$prop,
            &$node.options.compilation_messages,
            $lang,
            $code,
        );
    }};
}

macro_rules! cms_core {
    ($self:expr, $node:expr, $prop:expr, $lang:expr, $code:expr) => {{
        $self.compilation_messages(
            $node.node_type(),
            $node.node_id(),
            &$prop,
            &$node.compilation_messages,
            $lang,
            $code,
        );
    }};
}

macro_rules! cms_ems {
    ($self:expr, $node:expr, $prop:expr, $lang:expr, $code:expr) => {{
        $self.compilation_and_execution_messages(
            $node.node_type(),
            $node.node_id(),
            &$prop,
            &$node.options.compilation_messages,
            &$node.options.execution_messages,
            $lang,
            $code,
        );
    }};
}

macro_rules! code_chunk {
    ($self:expr, $node:expr) => {{
        $self.compilation_and_execution_messages(
            $node.node_type(),
            $node.node_id(),
            &Some(NodeProperty::Code),
            &$node.options.compilation_messages,
            &$node.options.execution_messages,
            $node.programming_language.as_deref(),
            Some(&$node.code),
        );

        $self.compilation_messages(
            $node.node_type(),
            $node.node_id(),
            &Some(NodeProperty::Overlay),
            &$node.options.overlay_compilation_messages,
            Some("svg"),
            $node.overlay.as_deref(),
        );
    }};
}

impl Visitor for Collector {
    #[rustfmt::skip]
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        match node {
            Node::AppendixBreak(node) => cms!(self, node, None, None, None),
            Node::Article(node) => {
                cms_ems!(self, node, None, None, None);
                // Also collect compilation messages from the bibliography if present
                if let Some(bibliography) = &node.options.bibliography {
                    self.visit_bibliography(bibliography);
                }
            }
            Node::CallBlock(node) => cms_ems!(self, node, None, None, None),
            Node::Chat(node) => cms_ems!(self, node, None, None, None),
            Node::ChatMessage(node) => cms_ems!(self, node, None, None, None),
            Node::CodeBlock(node) => cms!(self, node, Some(NodeProperty::Code), node.programming_language.as_deref(), Some(&node.code)),
            Node::CodeChunk(node) => code_chunk!(self, node),
            Node::Figure(node) => cms!(self, node, Some(NodeProperty::Overlay), Some("svg"), node.options.overlay.as_deref()),
            Node::ForBlock(node) => cms_ems!(self, node, Some(NodeProperty::Code), node.programming_language.as_deref(), Some(&node.code)),
            Node::IfBlock(node) => cms_ems!(self, node, None, None, None),
            Node::IncludeBlock(node) => cms_ems!(self, node, Some(NodeProperty::Source), None, None),
            Node::InstructionBlock(node) => cms_ems!(self, node, None, None, None),
            Node::MathBlock(node) => cms!(self, node, Some(NodeProperty::Code), node.math_language.as_deref(), Some(&node.code)),
            Node::Prompt(node) => cms_ems!(self, node, None, None, None),
            Node::PromptBlock(node) => cms_ems!(self, node, None, None, None),
            Node::StyledBlock(node) => cms!(self, node, Some(NodeProperty::Code), node.style_language.as_deref(), Some(&node.code)),
            _ => {}
        }

        WalkControl::Continue
    }

    #[rustfmt::skip]
    fn visit_block(&mut self, block: &Block) -> WalkControl {
        match block {
            Block::AppendixBreak(block) => cms!(self, block, None, None, None),
            Block::CallBlock(block) => cms_ems!(self, block, None, None, None),
            Block::ChatMessage(block) => cms_ems!(self, block, None, None, None),
            Block::CodeChunk(block) => code_chunk!(self, block),
            Block::Figure(block) => cms!(self, block, Some(NodeProperty::Overlay), Some("svg"), block.options.overlay.as_deref()),
            Block::ForBlock(block) => cms_ems!(self, block, Some(NodeProperty::Code), block.programming_language.as_deref(), Some(&block.code)),
            Block::IfBlock(block) => cms_ems!(self, block, None, None, None),
            Block::IncludeBlock(block) => {
                // Collect diagnostics on the include block itself..
                cms_ems!(self, block, None, None, None);

                // Continue walk but with `within` set
                self.within = Some((block.node_id(), block.source.clone()));
                block.content.walk(self);
                self.within = None;

                // Break walk because content already walked over
                return WalkControl::Break
            },
            Block::InstructionBlock(block) => cms_ems!(self, block, None, None, None),
            Block::MathBlock(block) => cms!(self, block, Some(NodeProperty::Code), block.math_language.as_deref(), Some(&block.code)),
            Block::PromptBlock(block) => cms_ems!(self, block, None, None, None),
            Block::StyledBlock(block) => cms!(self, block, Some(NodeProperty::Code), block.style_language.as_deref(), Some(&block.code)),
            _ => {}
        }

        WalkControl::Continue
    }

    #[rustfmt::skip]
    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        match inline {
            Inline::CodeExpression(inline) => cms_ems!(self, inline, Some(NodeProperty::Code), inline.programming_language.as_deref(), Some(&inline.code)),
            Inline::InstructionInline(inline) => cms_ems!(self, inline, None, None, None),
            Inline::MathInline(inline) => cms!(self, inline, Some(NodeProperty::Code), inline.math_language.as_deref(), Some(&inline.code)),
            Inline::StyledInline(inline) => cms!(self, inline, Some(NodeProperty::Code), inline.style_language.as_deref(), Some(&inline.code)),
            Inline::Link(inline) => cms_core!(self, inline, Some(NodeProperty::Target), None, None),
            Inline::Text(inline) => cms_core!(self, inline, None, None, Some(&inline.value)),
            _ => {}
        }

        WalkControl::Continue
    }

    fn visit_citation(&mut self, citation: &Citation) -> WalkControl {
        cms!(self, citation, Some(NodeProperty::Target), None, None);

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &IfBlockClause) -> WalkControl {
        cms_ems!(
            self,
            clause,
            Some(NodeProperty::Code),
            clause.programming_language.as_deref(),
            Some(&clause.code)
        );

        WalkControl::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_direct_code_location_without_poshmap() {
        let mut location = CodeLocation::new();
        location.source = Some("analysis.py".to_string());
        location.start_line = Some(2);
        location.start_column = Some(4);
        location.end_line = Some(2);
        location.end_column = Some(8);

        let diagnostic = Diagnostic {
            node_type: NodeType::SoftwareSourceCode,
            node_id: NodeId::null(),
            node_property: None,
            level: DiagnosticLevel::Advice,
            kind: DiagnosticKind::StaticAnalysis,
            error_type: Some("unresolved read".to_string()),
            message: "the path is dynamic".to_string(),
            help: Some("use a static path".to_string()),
            notes: vec!["resolved: data/".to_string()],
            format: Some(Format::Python),
            code: Some("\n\n    path".to_string()),
            code_location: Some(location),
        };

        assert_eq!(
            diagnostic.source_range8(None),
            Some(Range8::new(Position8::new(2, 4), Position8::new(2, 8)))
        );
    }
}
