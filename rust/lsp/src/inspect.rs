use async_lsp::lsp_types::Range;

use codec_text_trait::TextCodec;
use codecs::{Mapping, PoshMap};
use common::tracing;
use schema::{
    Admonition, Article, AudioObject, Block, Button, CallBlock, Chat, ChatMessage,
    ChatMessageGroup, Cite, CiteGroup, Claim, CodeBlock, CodeChunk, CodeExpression, CodeInline,
    Date, DateTime, DeleteBlock, DeleteInline, Duration, Emphasis, ExecutionStatus, Figure,
    ForBlock, Form, Heading, IfBlock, IfBlockClause, ImageObject, IncludeBlock, Inline,
    InsertBlock, InsertInline, InstructionBlock, InstructionInline, LabelType, Link, List,
    ListItem, MathBlock, MathInline, MediaObject, ModifyBlock, ModifyInline, Node, NodeId,
    NodeProperty, NodeType, Note, Paragraph, Parameter, Prompt, PromptBlock, ProvenanceCount,
    QuoteBlock, QuoteInline, RawBlock, ReplaceBlock, ReplaceInline, Section, Strikeout, Strong,
    StyledBlock, StyledInline, Subscript, SuggestionBlock, SuggestionInline, Superscript, Table,
    TableCell, TableRow, Text, ThematicBreak, Time, Timestamp, Underline, VideoObject, Visitor,
    WalkControl, Walkthrough, WalkthroughStep,
};

use crate::{
    text_document::{TextNode, TextNodeExecution},
    utils::range16_to_range,
};

/// A struct that implements the [`Visitor`] trait to collect
/// diagnostics, code lenses etc from the nodes in a document
pub(super) struct Inspector<'source, 'generated>
where
    'source: 'generated,
    'generated: 'source,
{
    /// The [`PoshMap`] used to correlate nodes with positions in the document
    poshmap: PoshMap<'source, 'generated>,

    /// The stack of nodes
    pub stack: Vec<TextNode>,

    /// Whether in a table cell
    ///
    /// Used to determine whether to collect provenance for paragraphs
    /// (we don't collect that for paragraphs in tables, to avoid noise)
    in_table_cell: bool,
}

impl<'source, 'generated> Inspector<'source, 'generated> {
    pub fn new(source: &'source str, generated: &'generated str, mapping: Mapping) -> Self {
        Self {
            poshmap: PoshMap::new(source, generated, mapping),
            stack: Vec::new(),
            in_table_cell: false,
        }
    }

    pub fn root(self) -> Option<TextNode> {
        self.stack.first().cloned()
    }

    fn enter_node(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        name: Option<String>,
        detail: Option<String>,
        execution: Option<TextNodeExecution>,
        provenance: Option<Vec<ProvenanceCount>>,
    ) -> &mut TextNode {
        let is_root = self.stack.is_empty();

        let range = if is_root {
            Range::default()
        } else {
            match self.poshmap.node_id_to_range16(&node_id) {
                Some(range) => range16_to_range(range),
                None => {
                    // A range may not exist for nodes that are not encoded into the
                    // text document (e.g. the non-active suggestions of an instruction).
                    // In these cases we return the default range (first char of document)
                    // and use that as a way of knowing whether to show code lenses of not
                    Range::default()
                }
            }
        };

        let (parent_type, parent_id) = self.stack.last().map_or_else(
            || (NodeType::Null, NodeId::null()),
            |node| (node.node_type, node.node_id.clone()),
        );

        let name = name.unwrap_or_else(|| match node_type {
            NodeType::InstructionBlock | NodeType::InstructionInline => "Command".to_string(),
            NodeType::SuggestionBlock | NodeType::SuggestionInline => "Suggestion".to_string(),
            NodeType::PromptBlock => "Prompt Preview".to_string(),
            NodeType::WalkthroughStep => "Step".to_string(),
            _ => node_type.to_string(),
        });

        let is_block = !is_root && node_type.is_block();

        self.stack.push(TextNode {
            range,
            is_root,
            parent_type,
            parent_id,
            is_block,
            node_type,
            node_id,
            name,
            detail,
            execution,
            provenance,
            ..Default::default()
        });

        self.stack.last_mut().expect("just pushed")
    }

    fn exit_node(&mut self) {
        if self.stack.len() > 1 {
            if let Some(node) = self.stack.pop() {
                // If has parent, add to its children
                if let Some(parent) = self.stack.last_mut() {
                    parent.children.push(node)
                }
            }
        }
    }
}

impl<'source, 'generated> Visitor for Inspector<'source, 'generated> {
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        #[allow(clippy::single_match)]
        match node {
            Node::Article(node) => node.inspect(self),
            Node::Chat(node) => node.inspect(self),
            Node::Prompt(node) => node.inspect(self),
            _ => {
                tracing::warn!("Node type `{node}` not inspected");
            }
        };

        // Break walk because `variant` visited above
        WalkControl::Break
    }

    fn visit_block(&mut self, block: &Block) -> WalkControl {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match block {
                    $(Block::$variant(node) => node.inspect(self),)*
                    Block::SuggestionBlock(..) => {}
                }
            };
        }
        variants!(
            Admonition,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            DeleteBlock,
            Figure,
            ForBlock,
            Form,
            Heading,
            IfBlock,
            IncludeBlock,
            InsertBlock,
            InstructionBlock,
            List,
            MathBlock,
            ModifyBlock,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            ReplaceBlock,
            Section,
            StyledBlock,
            Table,
            ThematicBreak,
            Walkthrough
        );

        // Break walk because `variant` visited above
        WalkControl::Break
    }

    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match inline {
                    $(Inline::$variant(node) => node.inspect(self),)*
                    Inline::SuggestionInline(..) |Inline::Null(..) | Inline::Boolean(..) | Inline::Integer(..) | Inline::UnsignedInteger(..) | Inline::Number(..) => {}
                }
            };
        }
        variants!(
            AudioObject,
            Button,
            Cite,
            CiteGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            DeleteInline,
            Duration,
            Emphasis,
            ImageObject,
            InsertInline,
            InstructionInline,
            Link,
            MathInline,
            MediaObject,
            ModifyInline,
            Note,
            Parameter,
            QuoteInline,
            ReplaceInline,
            StyledInline,
            Strikeout,
            Strong,
            Subscript,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            VideoObject
        );

        // Break walk because `variant` visited above
        WalkControl::Break
    }

    fn visit_suggestion_block(&mut self, block: &SuggestionBlock) -> WalkControl {
        self.enter_node(block.node_type(), block.node_id(), None, None, None, None);
        self.visit(&block.content);
        self.exit_node();

        // Break walk because `content` already visited above
        WalkControl::Break
    }

    fn visit_suggestion_inline(&mut self, inline: &SuggestionInline) -> WalkControl {
        let execution = if inline.execution_duration.is_some() {
            Some(TextNodeExecution {
                // Although suggestions do not have a status we need to add
                // on here so that a status notification is generated
                status: Some(ExecutionStatus::Succeeded),
                duration: inline.execution_duration.clone(),
                ended: inline.execution_ended.clone(),
                authors: inline.authors.clone(),
                ..Default::default()
            })
        } else {
            None
        };

        let provenance = inline.provenance.clone();

        self.enter_node(
            inline.node_type(),
            inline.node_id(),
            None,
            None,
            execution,
            provenance,
        );
        self.visit(&inline.content);
        self.exit_node();

        // Break walk because `content` already visited above
        WalkControl::Break
    }

    fn visit_if_block_clause(&mut self, clause: &IfBlockClause) -> WalkControl {
        let node_id = clause.node_id();

        let code_range = self
            .poshmap
            .node_property_to_range16(&node_id, NodeProperty::Code)
            .map(range16_to_range);

        let execution = Some(TextNodeExecution {
            status: clause.options.execution_status.clone(),
            required: clause.options.execution_required.clone(),
            duration: clause.options.execution_duration.clone(),
            ended: clause.options.execution_ended.clone(),
            messages: clause.options.execution_messages.clone(),
            code_range,
            ..Default::default()
        });

        let provenance = clause.provenance.clone();

        let node = self.enter_node(
            clause.node_type(),
            node_id,
            None,
            None,
            execution,
            provenance,
        );
        node.is_active = clause.is_active;
        self.visit(&clause.content);
        self.exit_node();

        // Break walk because `content` already visited above
        WalkControl::Break
    }

    fn visit_list_item(&mut self, list_item: &ListItem) -> WalkControl {
        self.enter_node(
            list_item.node_type(),
            list_item.node_id(),
            None,
            None,
            None,
            None,
        );
        self.visit(&list_item.content);
        self.exit_node();

        // Break walk because `content` already visited above
        WalkControl::Break
    }

    fn visit_table_row(&mut self, table_row: &TableRow) -> WalkControl {
        self.enter_node(
            table_row.node_type(),
            table_row.node_id(),
            None,
            None,
            None,
            None,
        );
        self.visit(&table_row.cells);
        self.exit_node();

        // Break walk because `cells` already visited above
        WalkControl::Break
    }

    fn visit_table_cell(&mut self, table_cell: &TableCell) -> WalkControl {
        self.enter_node(
            table_cell.node_type(),
            table_cell.node_id(),
            None,
            None,
            None,
            None,
        );
        self.in_table_cell = true;
        self.visit(&table_cell.content);
        self.in_table_cell = false;
        self.exit_node();

        // Break walk because `content` already visited above
        WalkControl::Break
    }

    fn visit_walkthrough_step(&mut self, step: &WalkthroughStep) -> WalkControl {
        let node = self.enter_node(step.node_type(), step.node_id(), None, None, None, None);
        node.is_active = if matches!(step.is_collapsed, Some(true)) {
            None
        } else {
            Some(true)
        };
        self.visit(&step.content);
        self.exit_node();

        // Break walk because `content` already visited above
        WalkControl::Break
    }
}

trait Inspect {
    fn inspect(&self, inspector: &mut Inspector);
}

impl Inspect for ChatMessage {
    fn inspect(&self, inspector: &mut Inspector) {
        let execution = Some(TextNodeExecution {
            mode: self.execution_mode.clone(),
            status: self.options.execution_status.clone(),
            required: self.options.execution_required.clone(),
            duration: self.options.execution_duration.clone(),
            ended: self.options.execution_ended.clone(),
            messages: self.options.execution_messages.clone(),
            ..Default::default()
        });

        inspector.enter_node(
            self.node_type(),
            self.node_id(),
            Some("Message".into()),
            Some(self.role.to_string()),
            execution,
            None,
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for CodeChunk {
    fn inspect(&self, inspector: &mut Inspector) {
        let name = match &self.label_type {
            Some(LabelType::FigureLabel) => "Figure",
            Some(LabelType::TableLabel) => "Table",
            None => "CodeChunk",
        }
        .to_string();

        let mut detail = String::new();
        if let Some(label) = &self.label {
            detail.push_str(label);
        };
        if let Some(caption) = &self
            .caption
            .as_ref()
            .and_then(|caption| caption.first())
            .map(|first| first.to_text().0)
        {
            if !detail.is_empty() {
                detail.push_str(": ");
            }
            detail.push_str(caption);
        }
        let detail = if detail.is_empty() {
            None
        } else {
            Some(detail)
        };

        let node_id = self.node_id();

        let code_range = inspector
            .poshmap
            .node_property_to_range16(&node_id, NodeProperty::Code)
            .map(range16_to_range);

        let execution = Some(TextNodeExecution {
            mode: self.execution_mode.clone(),
            status: self.options.execution_status.clone(),
            required: self.options.execution_required.clone(),
            bounded: self.options.execution_bounded.clone(),
            duration: self.options.execution_duration.clone(),
            ended: self.options.execution_ended.clone(),
            outputs: self.outputs.as_ref().map(|outputs| outputs.len()),
            messages: self.options.execution_messages.clone(),
            code_range,
            ..Default::default()
        });

        let provenance = self.provenance.clone();

        inspector.enter_node(
            self.node_type(),
            node_id,
            Some(name),
            detail,
            execution,
            provenance,
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for CodeExpression {
    fn inspect(&self, inspector: &mut Inspector) {
        let node_id = self.node_id();

        let code_range = inspector
            .poshmap
            .node_property_to_range16(&node_id, NodeProperty::Code)
            .map(range16_to_range);

        let execution = Some(TextNodeExecution {
            mode: self.execution_mode.clone(),
            status: self.options.execution_status.clone(),
            required: self.options.execution_required.clone(),
            duration: self.options.execution_duration.clone(),
            ended: self.options.execution_ended.clone(),
            messages: self.options.execution_messages.clone(),
            outputs: self.output.is_some().then_some(1),
            code_range,
            ..Default::default()
        });

        let provenance = self.provenance.clone();

        inspector.enter_node(self.node_type(), node_id, None, None, execution, provenance);
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for InstructionBlock {
    fn inspect(&self, inspector: &mut Inspector) {
        let mut execution = Some(TextNodeExecution {
            mode: self.execution_mode.clone(),
            status: self.options.execution_status.clone(),
            required: self.options.execution_required.clone(),
            duration: self.options.execution_duration.clone(),
            ended: self.options.execution_ended.clone(),
            messages: self.options.execution_messages.clone(),
            ..Default::default()
        });
        let mut index_of = None;

        if let Some(suggestions) = &self.suggestions {
            if !suggestions.is_empty() {
                // If there is an active suggestion and the instruction is not running, then
                // show the suggestion's duration, authors etc as the status
                if !matches!(
                    self.options.execution_status,
                    Some(ExecutionStatus::Running)
                ) {
                    if let Some(index) = self.active_suggestion {
                        if let Some(suggestion) = suggestions.get(index as usize) {
                            if suggestion.execution_duration.is_some() {
                                execution = Some(TextNodeExecution {
                                    // Although suggestions do not have a status we need to add
                                    // on here so that a status notification is generated
                                    status: Some(ExecutionStatus::Succeeded),
                                    duration: suggestion.execution_duration.clone(),
                                    ended: suggestion.execution_ended.clone(),
                                    authors: suggestion.authors.clone(),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }

                // Note that 0 = the original, 1 = the first suggestion, and so on...
                let index = self.active_suggestion.map(|index| index + 1).unwrap_or(0) as usize;
                let of = suggestions.len();
                index_of = Some((index, of));
            }
        }

        let node = inspector.enter_node(
            self.node_type(),
            self.node_id(),
            None,
            None,
            execution,
            None,
        );
        node.index_of = index_of;

        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for ForBlock {
    fn inspect(&self, inspector: &mut Inspector) {
        let node_id = self.node_id();

        let code_range = inspector
            .poshmap
            .node_property_to_range16(&node_id, NodeProperty::Code)
            .map(range16_to_range);

        let execution = Some(TextNodeExecution {
            mode: self.execution_mode.clone(),
            status: self.options.execution_status.clone(),
            required: self.options.execution_required.clone(),
            duration: self.options.execution_duration.clone(),
            ended: self.options.execution_ended.clone(),
            messages: self.options.execution_messages.clone(),
            code_range,
            ..Default::default()
        });

        let provenance = self.provenance.clone();

        inspector.enter_node(self.node_type(), node_id, None, None, execution, provenance);
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for Heading {
    fn inspect(&self, inspector: &mut Inspector) {
        let name = Some(format!("Heading {}", self.level));

        let (detail, provenance) = if !inspector.in_table_cell {
            (
                self.content.first().map(|first| first.to_text().0),
                self.provenance.clone(),
            )
        } else {
            (None, None)
        };

        inspector.enter_node(
            self.node_type(),
            self.node_id(),
            name,
            detail,
            None,
            provenance,
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for Paragraph {
    fn inspect(&self, inspector: &mut Inspector) {
        let (detail, provenance) = if !inspector.in_table_cell {
            (
                self.content.first().map(|first| first.to_text().0),
                self.provenance.clone(),
            )
        } else {
            (None, None)
        };

        inspector.enter_node(
            self.node_type(),
            self.node_id(),
            None,
            detail,
            None,
            provenance,
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for MathBlock {
    fn inspect(&self, inspector: &mut Inspector) {
        let node_id = self.node_id();

        let messages = self.options.compilation_messages.as_ref().map(|messages| {
            messages
                .iter()
                .map(|message| message.clone().into())
                .collect()
        });

        let code_range = inspector
            .poshmap
            .node_property_to_range16(&node_id, NodeProperty::Code)
            .map(range16_to_range);

        let execution = Some(TextNodeExecution {
            messages,
            code_range,
            ..Default::default()
        });

        inspector.enter_node(
            self.node_type(),
            node_id,
            None,
            None,
            execution,
            self.provenance.clone(),
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for RawBlock {
    fn inspect(&self, inspector: &mut Inspector) {
        let node_id = self.node_id();

        let messages = self.compilation_messages.as_ref().map(|messages| {
            messages
                .iter()
                .map(|message| message.clone().into())
                .collect()
        });

        let code_range = inspector
            .poshmap
            .node_property_to_range16(&node_id, NodeProperty::Content)
            .map(range16_to_range);

        let execution = Some(TextNodeExecution {
            messages,
            code_range,
            ..Default::default()
        });

        inspector.enter_node(
            self.node_type(),
            node_id,
            None,
            None,
            execution,
            self.provenance.clone(),
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for Walkthrough {
    fn inspect(&self, inspector: &mut Inspector) {
        inspector.enter_node(self.node_type(), self.node_id(), None, None, None, None);
        inspector.visit(self);
        inspector.exit_node();
    }
}

/// Default implementation for inline and content nodes
macro_rules! default {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                //eprintln!("INSPECT DEFAULT {}", self.node_id());

                inspector.enter_node(self.node_type(), self.node_id(), None, None, None, None);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

default!(
    // Works
    Article,
    Prompt,
    Chat,
    // Blocks
    Admonition,
    ChatMessageGroup,
    Claim,
    CodeBlock,
    DeleteBlock,
    Form,
    InsertBlock,
    List,
    ModifyBlock,
    QuoteBlock,
    ReplaceBlock,
    Section,
    ThematicBreak,
    // Inlines
    AudioObject,
    Button,
    Cite,
    CiteGroup,
    CodeInline,
    Date,
    DateTime,
    DeleteInline,
    Duration,
    Emphasis,
    InsertInline,
    ImageObject,
    Link,
    MathInline,
    MediaObject,
    ModifyInline,
    Note,
    QuoteInline,
    ReplaceInline,
    StyledInline,
    Strikeout,
    Strong,
    Subscript,
    Superscript,
    Text,
    Time,
    Timestamp,
    Underline,
    VideoObject
);

/// Implementation for nodes with compilation messages
macro_rules! compiled_options {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // eprintln!("INSPECT COMPILED {}", self.node_id());

                let execution = Some(TextNodeExecution{
                    messages: self.options.compilation_messages.as_ref().map(|messages| {
                        messages
                            .iter()
                            .map(|message| message.clone().into())
                            .collect()
                    }),
                    ..Default::default()
                });

                inspector.enter_node(self.node_type(), self.node_id(), None, None, execution, self.provenance.clone());
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}
compiled_options!(StyledBlock);

/// Implementation for tables and figures which have a label and caption to used for details
macro_rules! captioned {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // eprintln!("INSPECT CAPTIONED {}", self.node_id());

                let mut detail = String::new();
                if let Some(label) = &self.label {
                    detail.push_str(label);
                };
                if let Some(caption) = &self
                    .caption
                    .as_ref()
                    .and_then(|caption| caption.first())
                    .map(|first| first.to_text().0)
                {
                    if !detail.is_empty() {
                        detail.push_str(": ");
                    }
                    detail.push_str(caption);
                }
                let detail = if detail.is_empty() {
                    None
                } else {
                    Some(detail)
                };

                let provenance = self.provenance.clone();

                inspector.enter_node(self.node_type(), self.node_id(), None, detail, None, provenance);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

captioned!(Table, Figure);

/// Implementation for executable nodes
macro_rules! executable {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // eprintln!("INSPECT EXEC {}", self.node_id());

                let execution = Some(TextNodeExecution{
                    mode: self.execution_mode.clone(),
                    status: self.options.execution_status.clone(),
                    required: self.options.execution_required.clone(),
                    duration: self.options.execution_duration.clone(),
                    ended: self.options.execution_ended.clone(),
                    messages: self.options.execution_messages.clone(),
                    ..Default::default()
                });

                inspector.enter_node(self.node_type(), self.node_id(), None, None, execution, None);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

executable!(CallBlock, IfBlock, Parameter, InstructionInline);

/// Implementation for executable nodes but not recursing into
/// `content` to avoid lenses for content not rendered to Markdown
macro_rules! executable_not_content {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // eprintln!("INSPECT EXEC NO CONTENT {}", self.node_id());

                let execution =  Some(TextNodeExecution{
                    mode: self.execution_mode.clone(),
                    status: self.options.execution_status.clone(),
                    required: self.options.execution_required.clone(),
                    duration: self.options.execution_duration.clone(),
                    ended: self.options.execution_ended.clone(),
                    messages: self.options.execution_messages.clone(),
                    ..Default::default()
                });

                inspector.enter_node(self.node_type(), self.node_id(), None, None, execution, None);
                inspector.exit_node();
            }
        })*
    };
}

executable_not_content!(IncludeBlock, PromptBlock);
