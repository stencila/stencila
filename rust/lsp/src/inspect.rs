use async_lsp::lsp_types::Range;

use codec_text_trait::TextCodec;
use codecs::{Mapping, PoshMap};
use common::tracing;
use schema::{
    Admonition, Article, AudioObject, Block, Button, CallBlock, Cite, CiteGroup, Claim, CodeBlock,
    CodeChunk, CodeExpression, CodeInline, Date, DateTime, DeleteBlock, DeleteInline, Duration,
    Emphasis, Figure, ForBlock, Form, Heading, IfBlock, IfBlockClause, ImageObject, IncludeBlock,
    Inline, InsertBlock, InsertInline, InstructionBlock, InstructionInline, LabelType, Link, List,
    ListItem, MathBlock, MathInline, MediaObject, ModifyBlock, ModifyInline, Node, NodeId,
    NodeType, Note, Paragraph, Parameter, ProvenanceCount, QuoteBlock, QuoteInline, ReplaceBlock,
    ReplaceInline, Section, Strikeout, Strong, StyledBlock, StyledInline, Subscript,
    SuggestionBlockType, SuggestionInlineType, Superscript, Table, TableCell, TableRow, Text,
    ThematicBreak, Time, Timestamp, Underline, VideoObject, Visitor, WalkControl,
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
    ) {
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
            provenance,
            children: Vec::new(),
        })
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

        WalkControl::Break
    }

    fn visit_block(&mut self, block: &Block) -> WalkControl {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match block {
                    $(Block::$variant(node) => node.inspect(self),)*
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
            ReplaceBlock,
            Section,
            StyledBlock,
            Table,
            ThematicBreak
        );

        WalkControl::Break
    }

    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match inline {
                    $(Inline::$variant(node) => node.inspect(self),)*
                    Inline::Null(..) | Inline::Boolean(..) | Inline::Integer(..) | Inline::UnsignedInteger(..) | Inline::Number(..) => {}
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

        WalkControl::Break
    }

    fn visit_suggestion_block(&mut self, block: &SuggestionBlockType) -> WalkControl {
        use SuggestionBlockType::*;
        match block {
            InsertBlock(node) => node.inspect(self),
            DeleteBlock(node) => node.inspect(self),
            ModifyBlock(node) => node.inspect(self),
            ReplaceBlock(node) => node.inspect(self),
        };

        WalkControl::Break
    }

    fn visit_suggestion_inline(&mut self, inline: &SuggestionInlineType) -> WalkControl {
        use SuggestionInlineType::*;
        match inline {
            InsertInline(node) => node.inspect(self),
            DeleteInline(node) => node.inspect(self),
            ModifyInline(node) => node.inspect(self),
            ReplaceInline(node) => node.inspect(self),
        };

        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &IfBlockClause) -> WalkControl {
        self.enter_node(clause.node_type(), clause.node_id(), None, None, None, None);
        self.visit(&clause.content);
        self.exit_node();

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

        let execution = if let Some(execution_status) = &self.options.execution_status {
            Some(TextNodeExecution {
                status: execution_status.clone(),
                duration: self.options.execution_duration.clone(),
                ended: self.options.execution_ended.clone(),
                messages: self.options.execution_messages.clone(),
            })
        } else {
            None
        };

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
    MathBlock,
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
    ImageObject,
    InsertInline,
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

/// Implementation for nodes with content that can be used for detail
macro_rules! contented {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // eprintln!("INSPECT CONT {}", self.node_id());

                let (detail, provenance) = if !inspector.in_table_cell {
                    (
                        self.content.first().map(|first| first.to_text().0),
                        self.provenance.clone()
                    )
                } else{
                    (None, None)
                };

                inspector.enter_node(self.node_type(), self.node_id(), None, detail, None, provenance);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

contented!(StyledBlock);

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

                let execution = if let Some(execution_status) = &self.options.execution_status {
                    Some(TextNodeExecution{
                        status: execution_status.clone(),
                        duration: self.options.execution_duration.clone(),
                        ended: self.options.execution_ended.clone(),
                        messages: self.options.execution_messages.clone(),
                    })
                } else {
                    None
                };

                inspector.enter_node(self.node_type(), self.node_id(), None, None, execution, None);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

executable!(CallBlock, ForBlock, IfBlock, IncludeBlock, Parameter);

/// Implementation for executable nodes with provenance
macro_rules! executable_with_provenance {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // eprintln!("INSPECT EXEC PROV {}", self.node_id());

                let execution = if let Some(execution_status) = &self.options.execution_status {
                    Some(TextNodeExecution{
                        status: execution_status.clone(),
                        duration: self.options.execution_duration.clone(),
                        ended: self.options.execution_ended.clone(),
                        messages: self.options.execution_messages.clone(),
                    })
                } else {
                    None
                };

                let provenance = self.provenance.clone();

                inspector.enter_node(self.node_type(), self.node_id(), None, None, execution, provenance);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

executable_with_provenance!(CodeExpression, InstructionBlock, InstructionInline);
