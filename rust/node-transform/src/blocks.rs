use super::Transform;
use codec_txt::ToTxt;
use stencila_schema::*;

impl Transform for BlockContent {
    /// Transform a `BlockContent` variant to a `InlineContent` variant
    fn to_inline(&self) -> InlineContent {
        match self.to_owned() {
            // Variants with inline analogues
            BlockContent::CodeBlock(code_block) => InlineContent::CodeFragment(CodeFragment {
                text: code_block.text,
                programming_language: code_block.programming_language,
                ..Default::default()
            }),
            BlockContent::MathBlock(math_block) => InlineContent::MathFragment(MathFragment {
                text: math_block.text,
                math_language: math_block.math_language,
                ..Default::default()
            }),
            BlockContent::QuoteBlock(quote_block) => {
                let content = quote_block.content.to_inlines();
                let cite = if let Some(cite) = &quote_block.cite {
                    match cite.as_ref() {
                        QuoteBlockCite::Cite(cite) => Some(Box::new(QuoteCite::Cite(cite.clone()))),
                        QuoteBlockCite::String(str) => {
                            Some(Box::new(QuoteCite::String(str.clone())))
                        }
                    }
                } else {
                    None
                };
                InlineContent::Quote(Quote {
                    content,
                    cite,
                    ..Default::default()
                })
            }
            // Variants with inline content
            BlockContent::Heading(heading) => heading.content.to_inline(),
            BlockContent::Paragraph(paragraph) => paragraph.content.to_inline(),
            // Variants with block content
            BlockContent::Claim(claim) => claim.content.to_inline(),
            BlockContent::Include(Include {
                source, content, ..
            })
            | BlockContent::Call(Call {
                source, content, ..
            }) => match content {
                Some(content) => content.to_inline(),
                None => InlineContent::String(source),
            },
            // Fallback to a string
            _ => InlineContent::String(self.to_txt()),
        }
    }

    /// Transform a `BlockContent` variant to a vector of `InlineContent` variants
    fn to_inlines(&self) -> Vec<InlineContent> {
        match self.to_owned() {
            // Variants with inline content
            BlockContent::Heading(heading) => heading.content,
            BlockContent::Paragraph(paragraph) => paragraph.content,
            // Variants with block content
            BlockContent::Claim(claim) => claim.content.to_inlines(),
            BlockContent::Include(Include { content, .. })
            | BlockContent::Call(Call { content, .. }) => match content {
                Some(content) => content.to_inlines(),
                None => vec![self.to_inline()],
            },
            // Fallback to a single item vector of self converted
            _ => vec![self.to_inline()],
        }
    }

    /// Is a node a `BlockContent` variant e.g. a `Node:CodeChunk`
    fn is_block(&self) -> bool {
        true
    }

    /// Transform a `BlockContent` variant to a `BlockContent` variant
    ///
    /// Returns self.
    fn to_block(&self) -> BlockContent {
        self.to_owned()
    }

    /// Transform a `BlockContent` variant to a `Node` variant
    ///
    /// Most variants can be converted directly. However, `CreativeWork` types that have
    /// simple block variants need "upcasting" to their more complex types.
    fn to_node(&self) -> Node {
        match self.to_owned() {
            BlockContent::Call(node) => Node::Call(node),
            BlockContent::Claim(node) => {
                let ClaimSimple {
                    claim_type,
                    content,
                    id,
                    label,
                    parts,
                    title,
                    type_: _type,
                } = node;
                Node::Claim(Claim {
                    claim_type,
                    content,
                    id,
                    label,
                    parts,
                    title,
                    ..Default::default()
                })
            }
            BlockContent::CodeBlock(node) => Node::CodeBlock(node),
            BlockContent::CodeChunk(node) => Node::CodeChunk(node),
            BlockContent::Collection(node) => {
                let CollectionSimple {
                    content,
                    id,
                    parts,
                    title,
                    type_: _type,
                } = node;
                Node::Collection(Collection {
                    content,
                    id,
                    parts,
                    title,
                    ..Default::default()
                })
            }
            BlockContent::Figure(node) => {
                let FigureSimple {
                    caption,
                    content,
                    id,
                    label,
                    type_: _type,
                } = node;
                Node::Figure(Figure {
                    caption,
                    content,
                    id,
                    label,
                    ..Default::default()
                })
            }
            BlockContent::Heading(node) => Node::Heading(node),
            BlockContent::Include(node) => Node::Include(node),
            BlockContent::List(node) => Node::List(node),
            BlockContent::MathBlock(node) => Node::MathBlock(node),
            BlockContent::Paragraph(node) => Node::Paragraph(node),
            BlockContent::QuoteBlock(node) => Node::QuoteBlock(node),
            BlockContent::Table(node) => {
                let TableSimple {
                    caption,
                    id,
                    label,
                    rows,
                    type_: _type,
                } = node;
                Node::Table(Table {
                    caption,
                    id,
                    label,
                    rows,
                    ..Default::default()
                })
            }
            BlockContent::ThematicBreak(node) => Node::ThematicBreak(node),
        }
    }

    /// Transform a `BlockContent` variant to a vector of static `BlockContent` variants
    fn to_static_blocks(&self) -> Vec<BlockContent> {
        match self.to_owned() {
            // Dynamic node types: only include their "outputs"
            BlockContent::CodeChunk(chunk) => {
                let outputs = match chunk.outputs {
                    Some(outputs) => outputs,
                    None => return vec![],
                };

                if outputs.len() == 1 {
                    if let Node::Table(Table { rows, .. }) = outputs[0].to_owned() {
                        return vec![BlockContent::Table(TableSimple {
                            label: chunk.label,
                            caption: chunk.caption.as_ref().map(|caption| {
                                Box::new(match caption.as_ref() {
                                    CodeChunkCaption::String(string) => {
                                        TableCaption::String(string.to_owned())
                                    }
                                    CodeChunkCaption::VecBlockContent(blocks) => {
                                        TableCaption::VecBlockContent(blocks.to_static_blocks())
                                    }
                                })
                            }),
                            rows,
                            ..Default::default()
                        })];
                    }
                }

                vec![BlockContent::Figure(FigureSimple {
                    label: chunk.label,
                    caption: chunk.caption.as_ref().map(|caption| {
                        Box::new(match caption.as_ref() {
                            CodeChunkCaption::String(string) => {
                                FigureCaption::String(string.to_owned())
                            }
                            CodeChunkCaption::VecBlockContent(blocks) => {
                                FigureCaption::VecBlockContent(blocks.to_static_blocks())
                            }
                        })
                    }),
                    content: Some(Box::new(CreativeWorkContent::VecNode(outputs))),
                    ..Default::default()
                })]
            }
            BlockContent::Include(Include { content, .. })
            | BlockContent::Call(Call { content, .. }) => content
                .iter()
                .flat_map(Transform::to_static_blocks)
                .collect(),

            // Non-dynamic node types: make their content static
            BlockContent::Figure(figure) => vec![BlockContent::Figure(FigureSimple {
                caption: figure.caption.as_deref().map(|caption| match caption {
                    FigureCaption::String(..) => Box::new(caption.to_owned()),
                    FigureCaption::VecBlockContent(blocks) => {
                        Box::new(FigureCaption::VecBlockContent(blocks.to_static_blocks()))
                    }
                }),
                content: figure.content.map(|content| {
                    Box::new(match content.as_ref() {
                        CreativeWorkContent::String(string) => {
                            CreativeWorkContent::String(string.to_owned())
                        }
                        CreativeWorkContent::VecNode(nodes) => {
                            CreativeWorkContent::VecNode(nodes.to_static_nodes())
                        }
                    })
                }),
                ..figure
            })],
            BlockContent::Heading(heading) => vec![BlockContent::Heading(Heading {
                content: heading.content.to_static_inlines(),
                ..heading
            })],
            BlockContent::List(list) => vec![BlockContent::List(List {
                items: list
                    .items
                    .into_iter()
                    .map(|item| ListItem {
                        content: item.content.map(|content| match content {
                            ListItemContent::VecInlineContent(inlines) => {
                                ListItemContent::VecInlineContent(inlines.to_static_inlines())
                            }
                            ListItemContent::VecBlockContent(blocks) => {
                                ListItemContent::VecBlockContent(blocks.to_static_blocks())
                            }
                        }),
                        ..item
                    })
                    .collect(),
                ..list
            })],
            BlockContent::Paragraph(para) => vec![BlockContent::Paragraph(Paragraph {
                content: para.content.to_static_inlines(),
                ..para
            })],
            BlockContent::QuoteBlock(quote) => vec![BlockContent::QuoteBlock(QuoteBlock {
                content: quote.content.to_static_blocks(),
                ..quote
            })],
            BlockContent::Table(table) => vec![BlockContent::Table(TableSimple {
                caption: table.caption.as_deref().map(|caption| match caption {
                    TableCaption::String(..) => Box::new(caption.to_owned()),
                    TableCaption::VecBlockContent(blocks) => {
                        Box::new(TableCaption::VecBlockContent(blocks.to_static_blocks()))
                    }
                }),
                rows: table
                    .rows
                    .into_iter()
                    .map(|row| TableRow {
                        cells: row
                            .cells
                            .into_iter()
                            .map(|cell| TableCell { ..cell })
                            .collect(),
                        ..row
                    })
                    .collect(),
                ..table
            })],
            _ => self.to_blocks(),
        }
    }
}

impl Transform for Vec<BlockContent> {
    /// Transform a vector of `BlockContent` variants to a `InlineContent` variant
    ///
    /// If there is just one item, returns that converted to an inline. Otherwise,
    /// converts to a vector of inlines and converts those to a single inline.
    fn to_inline(&self) -> InlineContent {
        if self.len() == 1 {
            self[0].to_inline()
        } else {
            InlineContent::Emphasis(Emphasis {
                content: self.to_inlines(),
                ..Default::default()
            })
        }
    }

    /// Transform a vector of `BlockContent` variants to a vector of `InlineContent` variants
    ///
    /// Returns self mapped into inlines.
    fn to_inlines(&self) -> Vec<InlineContent> {
        self.iter().flat_map(|node| node.to_inlines()).collect()
    }

    /// Transform a vector of `BlockContent` variants to a `BlockContent` variant
    ///
    /// If there is just one item, returns that. Otherwise, wraps into a `QuoteBock`
    /// (one of the few node types that has block content).
    fn to_block(&self) -> BlockContent {
        if self.len() == 1 {
            self[0].to_owned()
        } else {
            BlockContent::QuoteBlock(QuoteBlock {
                content: self.to_owned(),
                ..Default::default()
            })
        }
    }

    /// Transform a vector of `BlockContent` variants to a vector of `BlockContent` variants
    ///
    /// Returns self.
    fn to_blocks(&self) -> Vec<BlockContent> {
        self.to_owned()
    }

    /// Transform a vector of `BlockContent` variants to a `Node` variant
    ///
    /// Wraps self into a `QuoteBock`.
    fn to_node(&self) -> Node {
        Node::QuoteBlock(QuoteBlock {
            content: self.to_owned(),
            ..Default::default()
        })
    }

    /// Transform a vector of `BlockContent` variants to a vector of `Node` variants
    ///
    /// Returns self mapped into nodes.
    fn to_nodes(&self) -> Vec<Node> {
        self.iter().flat_map(|node| node.to_nodes()).collect()
    }

    /// Transform a vector of `BlockContent` variants to a vector of static `InlineContent` variants
    fn to_static_inlines(&self) -> Vec<InlineContent> {
        self.iter()
            .flat_map(|block| block.to_static_inlines())
            .collect()
    }

    /// Transform a vector of `BlockContent` variants to a vector of static `BlockContent` variants
    fn to_static_blocks(&self) -> Vec<BlockContent> {
        self.iter()
            .flat_map(|block| block.to_static_blocks())
            .collect()
    }

    /// Transform a vector of `BlockContent` variants to a vector of static `Node` variants
    fn to_static_nodes(&self) -> Vec<Node> {
        self.iter()
            .flat_map(|block| block.to_static_nodes())
            .collect()
    }
}
