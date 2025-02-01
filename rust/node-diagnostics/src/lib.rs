use std::ops::Range;

use ariadne::{Config, Label, Report, ReportKind, Source};

use codec_info::{PoshMap, Position8, Positions};
use common::eyre::Result;
use format::Format;
use schema::{
    Block, CodeLocation, CompilationMessage, Cord, ExecutionMessage, Inline, MessageLevel, Node,
    NodeId, NodeProperty, NodeType, Visitor, WalkControl, WalkNode,
};

/// Collect all diagnostic messages from a a node
///
/// Currently, collects the [`CompilationMessage`] and [`ExecutionMessage`]s
/// from on and within the node. In the future, additional diagnostics,
/// (e.g verification of internal and external links) may be added
pub fn diagnostics<T>(node: &T) -> Vec<Diagnostic>
where
    T: WalkNode,
{
    let mut walker = Collector::default();
    walker.visit(node);
    walker.messages
}

pub struct Diagnostic {
    /// The type of node that the diagnostic is for
    node_type: NodeType,

    /// The id of the node that the diagnostic is for
    node_id: NodeId,

    /// The severity level of the diagnostic
    level: DiagnosticLevel,

    /// The kind of diagnostic
    kind: Option<String>,

    /// The diagnostic's message
    message: String,

    /// The format / programming language associated with the diagnostic
    format: Option<Format>,

    /// The source code associated with the diagnostic
    code: Option<String>,

    /// The location of the diagnostic within the code
    code_location: Option<CodeLocation>,
}

enum DiagnosticLevel {
    /// An advisory diagnostic
    Advice,
    /// A warning diagnostic
    Warning,
    /// An error diagnostic
    Error,
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
        let (report, source) = self.to_report(path, source, poshmap, false, true)?;
        let cache = (path, Source::from(source));

        let mut buffer = Vec::new();
        report.write(cache, &mut buffer)?;
        let string = String::from_utf8(buffer)?;

        Ok(string)
    }

    /// Pretty print the diagnostic to stderr
    pub fn to_stderr_pretty<'s>(
        self,
        path: &'s str,
        source: &'s str,
        poshmap: &Option<PoshMap<'s, 's>>,
    ) -> Result<()> {
        let (report, source) = self.to_report(path, source, poshmap, true, false)?;
        let cache = (path, Source::from(source));

        report.eprint(cache)?;

        Ok(())
    }

    fn to_report<'s>(
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

        // Generate details for at top of diagnostic
        let mut details = String::new();
        if let Some(format) = &self.format {
            details.push_str(format.name());
            details.push(' ');
        }
        details.push_str(&self.node_type.to_string());
        details.push(' ');
        if let Some(kind) = &self.kind {
            details.push_str(kind);
            details.push(' ');
        }

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

        // Guess the property of the node that the diagnostic is for
        let property = match self.node_type {
            NodeType::IncludeBlock => NodeProperty::Source,
            _ => NodeProperty::Code,
        };

        // Get the range of the node (or it's code if any) within the code
        // Note: this function is usually only passed a poshmap if using document source
        let range = poshmap
            .as_ref()
            .and_then(|poshmap| {
                poshmap
                    .node_property_to_range8(&self.node_id, property)
                    .or_else(|| poshmap.node_id_to_range8(&self.node_id))
            })
            .unwrap_or_default();

        // Convert line/column range to character range
        let range = if let Some(location) = self.code_location {
            // If there is a code location then shift the range
            let line = location.start_line.unwrap_or(0) as usize;
            let column = location.start_column.unwrap_or(0) as usize;
            let start = positions
                .index_at_position8(Position8 {
                    line: range.start.line + line,
                    column: range.start.column + column,
                })
                .unwrap_or(0);

            let line = location.end_line.map_or_else(|| line, |line| line as usize);
            let column = location
                .end_column
                .map_or_else(|| column, |col| col as usize);
            let end = positions
                .index_at_position8(Position8 {
                    line: range.start.line + line,
                    column: range.start.column + column,
                })
                .unwrap_or(start)
                .max(start);

            start..end
        } else {
            let start = positions.index_at_position8(range.start).unwrap_or(0);
            let end = positions
                .index_at_position8(range.end)
                .unwrap_or(start)
                .max(start);

            start..end
        };

        let report = Report::build(kind, (path, range.clone()))
            .with_message(&details)
            .with_label(Label::new((path, range)).with_message(self.message))
            .with_config(Config::new().with_color(color).with_compact(compact))
            .finish();

        Ok((report, source))
    }
}

/// A visitor that walks over a node and collects any messages
#[derive(Default)]
struct Collector {
    messages: Vec<Diagnostic>,
}

impl Collector {
    /// Collect the [`CompilationMessage`]s from a node
    fn compilation_messages(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        messages: &Option<Vec<CompilationMessage>>,
        lang: Option<&str>,
        code: Option<&Cord>,
    ) {
        let mut msgs = messages
            .iter()
            .flatten()
            .map(|msg| Diagnostic {
                node_type,
                node_id: node_id.clone(),
                level: DiagnosticLevel::from(&msg.level),
                kind: msg.error_type.clone(),
                message: msg.message.clone(),
                format: lang.map(Format::from_name),
                code: code.map(|cord| cord.to_string()),
                code_location: msg.code_location.clone(),
            })
            .collect();
        self.messages.append(&mut msgs)
    }

    /// Collect the [`ExecutionMessage`]s from a node
    fn execution_messages(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        messages: &Option<Vec<ExecutionMessage>>,
        lang: Option<&str>,
        code: Option<&Cord>,
    ) {
        let mut msgs = messages
            .iter()
            .flatten()
            .map(|msg| Diagnostic {
                node_type,
                node_id: node_id.clone(),
                level: DiagnosticLevel::from(&msg.level),
                kind: msg.error_type.clone(),
                message: msg.message.clone(),
                format: lang.map(Format::from_name),
                code: code.map(|cord| cord.to_string()),
                code_location: msg.code_location.clone(),
            })
            .collect();
        self.messages.append(&mut msgs)
    }

    /// Collect the [`CompilationMessage`]s and [`ExecutionMessage`]s from a node
    fn compilation_and_execution_messages(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        compilation_messages: &Option<Vec<CompilationMessage>>,
        execution_messages: &Option<Vec<ExecutionMessage>>,
        lang: Option<&str>,
        code: Option<&Cord>,
    ) {
        self.compilation_messages(node_type, node_id.clone(), compilation_messages, lang, code);
        self.execution_messages(node_type, node_id, execution_messages, lang, code);
    }
}

macro_rules! cms {
    ($self:expr, $node:expr, $lang:expr, $code:expr) => {{
        $self.compilation_messages(
            $node.node_type(),
            $node.node_id(),
            &$node.options.compilation_messages,
            $lang,
            $code,
        );
    }};
}

macro_rules! cms_ems {
    ($self:expr, $node:expr, $lang:expr, $code:expr) => {{
        $self.compilation_and_execution_messages(
            $node.node_type(),
            $node.node_id(),
            &$node.options.compilation_messages,
            &$node.options.execution_messages,
            $lang,
            $code,
        );
    }};
}

impl Visitor for Collector {
    #[rustfmt::skip]
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        match node {
            Node::Article(node) => cms_ems!(self, node, None, None),
            Node::CallBlock(node) => cms_ems!(self, node, None, None),
            Node::Chat(node) => cms_ems!(self, node, None, None),
            Node::ChatMessage(node) => cms_ems!(self, node, None, None),
            Node::CodeChunk(node) => cms_ems!(self, node, node.programming_language.as_deref(), Some(&node.code)),
            Node::ForBlock(node) => cms_ems!(self, node, node.programming_language.as_deref(), Some(&node.code)),
            Node::IfBlock(node) => cms_ems!(self, node, None, None),
            Node::IncludeBlock(node) => cms_ems!(self, node, None, None),
            Node::InstructionBlock(node) => cms_ems!(self, node, None, None),
            Node::MathBlock(node) => cms!(self, node, node.math_language.as_deref(), Some(&node.code)),
            Node::Prompt(node) => cms_ems!(self, node, None, None),
            Node::PromptBlock(node) => cms_ems!(self, node, None, None),
            Node::StyledBlock(node) => cms!(self, node, node.style_language.as_deref(), Some(&node.code)),
            _ => {}
        }

        WalkControl::Continue
    }

    #[rustfmt::skip]
    fn visit_block(&mut self, block: &schema::Block) -> WalkControl {
        match block {
            Block::CallBlock(block) => cms_ems!(self, block, None, None),
            Block::ChatMessage(block) => cms_ems!(self, block, None, None),
            Block::CodeChunk(block) => cms_ems!(self, block, block.programming_language.as_deref(), Some(&block.code)),
            Block::ForBlock(block) => cms_ems!(self, block, block.programming_language.as_deref(), Some(&block.code)),
            Block::IfBlock(block) => cms_ems!(self, block, None, None),
            Block::IncludeBlock(block) => {
                // Collect diagnostics on the include block itself but do
                // not continue walk into the included content because
                // we are unable to link any diagnostics in there to the
                // original source location.
                cms_ems!(self, block, None, None);
                return WalkControl::Break;
            },
            Block::InstructionBlock(block) => cms_ems!(self, block, None, None),
            Block::MathBlock(block) => cms!(self, block, block.math_language.as_deref(), Some(&block.code)),
            Block::PromptBlock(block) => cms_ems!(self, block, None, None),
            Block::StyledBlock(block) => cms!(self, block, block.style_language.as_deref(), Some(&block.code)),
            _ => {}
        }

        WalkControl::Continue
    }

    #[rustfmt::skip]
    fn visit_inline(&mut self, inline: &schema::Inline) -> WalkControl {
        match inline {
            Inline::CodeExpression(inline) => cms_ems!(self, inline, inline.programming_language.as_deref(), Some(&inline.code)),
            Inline::InstructionInline(inline) => cms_ems!(self, inline, None, None),
            Inline::MathInline(inline) => cms!(self, inline, inline.math_language.as_deref(), Some(&inline.code)),
            Inline::StyledInline(inline) => cms!(self, inline, inline.style_language.as_deref(), Some(&inline.code)),
            _ => {}
        }

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &schema::IfBlockClause) -> WalkControl {
        cms_ems!(self, clause, clause.programming_language.as_deref(), None);

        WalkControl::Continue
    }
}
