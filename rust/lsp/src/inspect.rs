use async_lsp::lsp_types::{Position, Range};

use codec_text_trait::TextCodec;
use codecs::{Mapping, PoshMap, Position16, Range16};
use schema::{
    Admonition, Article, AudioObject, Block, Button, CallBlock, Cite, CiteGroup, Claim, CodeBlock,
    CodeChunk, CodeExpression, CodeInline, Date, DateTime, DeleteBlock, DeleteInline, Duration,
    Emphasis, Figure, ForBlock, Form, Heading, IfBlock, IfBlockClause, ImageObject, IncludeBlock,
    Inline, InsertBlock, InsertInline, InstructionBlock, InstructionInline, Link, List, ListItem,
    MathBlock, MathInline, MediaObject, ModifyBlock, ModifyInline, Node, NodeId, NodeType, Note,
    Paragraph, Parameter, QuoteBlock, QuoteInline, ReplaceBlock, ReplaceInline, Section, Strikeout,
    Strong, StyledBlock, StyledInline, Subscript, Superscript, Table, TableCell, TableRow, Text,
    ThematicBreak, Time, Timestamp, Underline, VideoObject, Visitor, WalkControl,
};

use crate::text_document::TextNode;

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
}

impl<'source, 'generated> Inspector<'source, 'generated> {
    pub fn new(source: &'source str, generated: &'generated str, mapping: Mapping) -> Self {
        Self {
            poshmap: PoshMap::new(source, generated, mapping),
            stack: Vec::new(),
        }
    }

    pub fn root(self) -> Option<TextNode> {
        self.stack.first().cloned()
    }

    fn enter_node(&mut self, node_type: NodeType, node_id: NodeId, detail: Option<String>) {
        if let Some(range) = self.poshmap.node_id_to_range16(&node_id) {
            self.stack.push(TextNode {
                range: range16_to_range(range),
                node_type,
                node_id,
                detail,
                children: Vec::new(),
            })
        }
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

/// Convert a Stencila [`Range16`] to a LSP [`Range`]
fn range16_to_range(range: Range16) -> Range {
    Range {
        start: position16_to_position(range.start),
        end: position16_to_position(range.end),
    }
}

/// Convert a Stencila [`Position16`] to a LSP [`Position`]
fn position16_to_position(position: Position16) -> Position {
    Position {
        line: position.line as u32,
        character: position.column as u32,
    }
}

impl<'source, 'generated> Visitor for Inspector<'source, 'generated> {
    fn visit_node(&mut self, node: &Node) -> WalkControl {
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

    fn visit_if_block_clause(&mut self, clause: &IfBlockClause) -> WalkControl {
        self.enter_node(clause.node_type(), clause.node_id(), None);
        self.visit(&clause.content);
        self.exit_node();

        WalkControl::Break
    }

    fn visit_list_item(&mut self, list_item: &ListItem) -> WalkControl {
        self.enter_node(list_item.node_type(), list_item.node_id(), None);
        self.visit(&list_item.content);
        self.exit_node();

        WalkControl::Break
    }

    fn visit_table_row(&mut self, table_row: &TableRow) -> WalkControl {
        self.enter_node(table_row.node_type(), table_row.node_id(), None);
        self.visit(&table_row.cells);
        self.exit_node();

        WalkControl::Break
    }

    fn visit_table_cell(&mut self, table_cell: &TableCell) -> WalkControl {
        self.enter_node(table_cell.node_type(), table_cell.node_id(), None);
        self.visit(&table_cell.content);
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
            node_type: self.node_type(),
            node_id: self.node_id(),
            detail: None,
            children: Vec::new(),
        });

        // Visit the article
        inspector.visit(self);
    }
}

/// Default implementation for inline and content nodes
macro_rules! default {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                inspector.enter_node(self.node_type(), self.node_id(), None);
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
    Figure,
    Form,
    InsertBlock,
    List,
    MathBlock,
    ModifyBlock,
    QuoteBlock,
    ReplaceBlock,
    Section,
    StyledBlock,
    Table,
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
                let detail = self.content.first().map(|first| first.to_text().0);

                inspector.enter_node(self.node_type(), self.node_id(), detail);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

contented!(Paragraph, Heading);

/// Implementation for executable nodes
macro_rules! executable {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                // TODO: record diagnostics here

                inspector.enter_node(self.node_type(), self.node_id(), None);
                inspector.visit(self);
                inspector.exit_node();
            }
        })*
    };
}

executable!(
    CallBlock,
    CodeChunk,
    CodeExpression,
    ForBlock,
    IfBlock,
    IncludeBlock,
    InstructionBlock,
    InstructionInline,
    Parameter
);
