use async_lsp::lsp_types::Range;

use codec_text_trait::TextCodec;
use codecs::{Mapping, PoshMap};
use common::tracing;
use schema::{
    Admonition, Article, AudioObject, Block, Button, CallBlock, Cite, CiteGroup, Claim, CodeBlock,
    CodeChunk, CodeExpression, CodeInline, Date, DateTime, DeleteBlock, DeleteInline, Duration,
    Emphasis, ExecutionStatus, Figure, ForBlock, Form, Heading, IfBlock, IfBlockClause,
    ImageObject, IncludeBlock, Inline, InsertBlock, InsertInline, InstructionBlock,
    InstructionInline, LabelType, Link, List, ListItem, MathBlock, MathInline, MediaObject,
    ModifyBlock, ModifyInline, Node, NodeId, NodeType, Note, Paragraph, Parameter, ProvenanceCount,
    QuoteBlock, QuoteInline, RawBlock, ReplaceBlock, ReplaceInline, Section, Strikeout, Strong,
    StyledBlock, StyledInline, Subscript, SuggestionBlock, SuggestionInline, Superscript, Table,
    TableCell, TableRow, Text, ThematicBreak, Time, Timestamp, Underline, VideoObject, Visitor,
    WalkControl,
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
        let range = match self.poshmap.node_id_to_range16(&node_id) {
            Some(range) => range16_to_range(range),
            None => {
                tracing::warn!("No range for {node_id}");
                Range::default()
            }
        };

        let (parent_type, parent_id) = self.stack.last().map_or_else(
            || (NodeType::Null, NodeId::null()),
            |node| (node.node_type, node.node_id.clone()),
        );

        let name = name.unwrap_or_else(|| node_type.to_string());

        self.stack.push(TextNode {
            range,
            parent_type,
            parent_id,
            node_type,
            node_id,
            name,
            detail,
            execution,
            is_active: None,
            provenance,
            children: Vec::new(),
        });

        self.stack.last_mut().expect("just pushed")
    }

    fn exit_node(&mut self) {
        if self.stack.len() > 1 {
            if let Some(node) = self.stack.pop() {
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
            _ => {}
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
            QuoteBlock,
            RawBlock,
            ReplaceBlock,
            Section,
            StyledBlock,
            Table,
            ThematicBreak
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
        let execution = if block.execution_duration.is_some() {
            Some(TextNodeExecution {
                // Although suggestions do not have a status we need to add
                // on here so that a status notification is generated
                status: Some(ExecutionStatus::Succeeded),
                duration: block.execution_duration.clone(),
                ended: block.execution_ended.clone(),
                authors: block.authors.clone(),
                ..Default::default()
            })
        } else {
            None
        };

        let provenance = block.provenance.clone();

        self.enter_node(
            block.node_type(),
            block.node_id(),
            None,
            None,
            execution,
            provenance,
        );
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
        let execution = Some(TextNodeExecution {
            status: clause.options.execution_status.clone(),
            required: clause.options.execution_required.clone(),
            duration: clause.options.execution_duration.clone(),
            ended: clause.options.execution_ended.clone(),
            messages: clause.options.execution_messages.clone(),
            ..Default::default()
        });

        let provenance = clause.provenance.clone();

        let node = self.enter_node(
            clause.node_type(),
            clause.node_id(),
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
}

trait Inspect {
    fn inspect(&self, inspector: &mut Inspector);
}

impl Inspect for Article {
    fn inspect(&self, inspector: &mut Inspector) {
        // Set this as the root node that others will become children of
        inspector.stack.push(TextNode {
            range: Range::default(),
            parent_type: NodeType::Null,
            parent_id: NodeId::null(),
            node_type: self.node_type(),
            node_id: self.node_id(),
            name: "Article".to_string(),
            detail: None,
            // Do not collect execution details or provenance because
            // we do not want these displayed on the first line in code lenses etc
            execution: None,
            provenance: None,
            is_active: None,
            children: Vec::new(),
        });

        // Visit the article
        inspector.visit(self);
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

        let execution = Some(TextNodeExecution {
            mode: self.execution_mode.clone(),
            status: self.options.execution_status.clone(),
            required: self.options.execution_required.clone(),
            kind: self.options.execution_kind.clone(),
            duration: self.options.execution_duration.clone(),
            ended: self.options.execution_ended.clone(),
            outputs: self.outputs.as_ref().map(|outputs| outputs.len()),
            messages: self.options.execution_messages.clone(),
            ..Default::default()
        });

        let provenance = self.provenance.clone();

        inspector.enter_node(
            self.node_type(),
            self.node_id(),
            Some(name),
            detail,
            execution,
            provenance,
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}

impl Inspect for Heading {
    fn inspect(&self, inspector: &mut Inspector) {
        let name = Some(format!("H{}", self.level));

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
        let name = Some("Para.".to_string());

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
    // Blocks
    Admonition,
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
impl Inspect for RawBlock {
    fn inspect(&self, inspector: &mut Inspector) {
        // eprintln!("INSPECT COMPILED {}", self.node_id());

        let execution = Some(TextNodeExecution {
            messages: self.compilation_messages.as_ref().map(|messages| {
                messages
                    .iter()
                    .map(|message| message.clone().into())
                    .collect()
            }),
            ..Default::default()
        });

        inspector.enter_node(
            self.node_type(),
            self.node_id(),
            None,
            None,
            execution,
            self.provenance.clone(),
        );
        inspector.visit(self);
        inspector.exit_node();
    }
}
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
compiled_options!(MathBlock, StyledBlock);

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

executable!(
    CallBlock,
    ForBlock,
    IfBlock,
    IncludeBlock,
    Parameter,
    InstructionBlock,
    InstructionInline
);

/// Implementation for executable nodes with provenance
macro_rules! executable_with_provenance {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // eprintln!("INSPECT EXEC PROV {}", self.node_id());

                let execution =  Some(TextNodeExecution{
                    mode: self.execution_mode.clone(),
                    status: self.options.execution_status.clone(),
                    required: self.options.execution_required.clone(),
                    duration: self.options.execution_duration.clone(),
                    ended: self.options.execution_ended.clone(),
                    messages: self.options.execution_messages.clone(),
                    ..Default::default()
                });

                let provenance = self.provenance.clone();

                inspector.enter_node(self.node_type(), self.node_id(), None, None, execution, provenance);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

executable_with_provenance!(CodeExpression);
