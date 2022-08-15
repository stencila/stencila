use super::Transform;
use stencila_schema::*;

impl Transform for Node {
    /// Is a node an `InlineContent` variant e.g. a `Node:Strong`
    fn is_inline(&self) -> bool {
        matches!(
            self,
            Node::AudioObject(..)
                | Node::Boolean(..)
                | Node::Cite(..)
                | Node::CiteGroup(..)
                | Node::CodeExpression(..)
                | Node::CodeFragment(..)
                | Node::Delete(..)
                | Node::Emphasis(..)
                | Node::ImageObject(..)
                | Node::Integer(..)
                | Node::Link(..)
                | Node::MathFragment(..)
                | Node::NontextualAnnotation(..)
                | Node::Note(..)
                | Node::Null(..)
                | Node::Number(..)
                | Node::Parameter(..)
                | Node::Quote(..)
                | Node::Strikeout(..)
                | Node::String(..)
                | Node::Strong(..)
                | Node::Subscript(..)
                | Node::Superscript(..)
                | Node::Underline(..)
                | Node::VideoObject(..)
        )
    }

    /// Transform a `Node` variant to a `InlineContent` variant
    ///
    /// Most inline variants can be converted directly. However, `CreativeWork` types that have
    /// simple inline variants need "downcasting" to their simpler types.
    fn to_inline(&self) -> InlineContent {
        match self.to_owned() {
            // Inline variants
            Node::AudioObject(node) => {
                let AudioObject {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    title,
                    transcript,
                    ..
                } = node;
                InlineContent::AudioObject(AudioObjectSimple {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    title,
                    transcript,
                    ..Default::default()
                })
            }
            Node::Boolean(node) => InlineContent::Boolean(node),
            Node::Cite(node) => InlineContent::Cite(node),
            Node::CiteGroup(node) => InlineContent::CiteGroup(node),
            Node::CodeExpression(node) => InlineContent::CodeExpression(node),
            Node::CodeFragment(node) => InlineContent::CodeFragment(node),
            Node::Delete(node) => InlineContent::Delete(node),
            Node::Emphasis(node) => InlineContent::Emphasis(node),
            Node::ImageObject(node) => {
                let ImageObject {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    thumbnail,
                    title,
                    ..
                } = node;
                InlineContent::ImageObject(ImageObjectSimple {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    thumbnail,
                    title,
                    ..Default::default()
                })
            }
            Node::Integer(node) => InlineContent::Integer(node),
            Node::Link(node) => InlineContent::Link(node),
            Node::MathFragment(node) => InlineContent::MathFragment(node),
            Node::NontextualAnnotation(node) => InlineContent::NontextualAnnotation(node),
            Node::Note(node) => InlineContent::Note(node),
            Node::Null(node) => InlineContent::Null(node),
            Node::Number(node) => InlineContent::Number(node),
            Node::Parameter(node) => InlineContent::Parameter(node),
            Node::Quote(node) => InlineContent::Quote(node),
            Node::Strikeout(node) => InlineContent::Strikeout(node),
            Node::String(node) => InlineContent::String(node),
            Node::Strong(node) => InlineContent::Strong(node),
            Node::Subscript(node) => InlineContent::Subscript(node),
            Node::Superscript(node) => InlineContent::Superscript(node),
            Node::Underline(node) => InlineContent::Underline(node),
            Node::VideoObject(node) => {
                let VideoObject {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    thumbnail,
                    title,
                    transcript,
                    ..
                } = node;
                InlineContent::VideoObject(VideoObjectSimple {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    thumbnail,
                    title,
                    transcript,
                    ..Default::default()
                })
            }
            // Fallback to transforming to block content and then inline
            _ => self.to_block().to_inline(),
        }
    }

    /// Transform a `Node` variant to a vector of `InlineContent` variants
    fn to_inlines(&self) -> Vec<InlineContent> {
        match self.to_owned() {
            // Variants with inline content
            Node::Delete(node) => node.content,
            Node::Emphasis(node) => node.content,
            Node::Heading(node) => node.content,
            Node::NontextualAnnotation(node) => node.content,
            Node::Paragraph(node) => node.content,
            Node::Quote(node) => node.content,
            Node::Strikeout(node) => node.content,
            Node::Strong(node) => node.content,
            Node::Subscript(node) => node.content,
            Node::Superscript(node) => node.content,
            Node::Underline(node) => node.content,

            // Variants with block content
            Node::Article(Article { content, .. }) => match content {
                Some(content) => content.to_inlines(),
                None => Vec::new(),
            },

            // Fallback to a single item vector of self converted
            _ => vec![self.to_inline()],
        }
    }

    /// Is a node a `BlockContent` variant e.g. a `Node:CodeChunk`
    fn is_block(&self) -> bool {
        matches!(
            self,
            Node::Claim(..)
                | Node::CodeBlock(..)
                | Node::CodeChunk(..)
                | Node::Collection(..)
                | Node::Figure(..)
                | Node::Heading(..)
                | Node::Include(..)
                | Node::List(..)
                | Node::MathBlock(..)
                | Node::Paragraph(..)
                | Node::QuoteBlock(..)
                | Node::Table(..)
                | Node::ThematicBreak(..)
        )
    }

    /// Transform a `Node` variant to a `BlockContent` variant
    fn to_block(&self) -> BlockContent {
        match self.to_owned() {
            // Block variants
            Node::Claim(node) => {
                let Claim {
                    claim_type,
                    content,
                    id,
                    label,
                    parts,
                    title,
                    ..
                } = node;
                BlockContent::Claim(ClaimSimple {
                    claim_type,
                    content,
                    id,
                    label,
                    parts,
                    title,
                    ..Default::default()
                })
            }
            Node::CodeBlock(node) => BlockContent::CodeBlock(node),
            Node::CodeChunk(node) => BlockContent::CodeChunk(node),
            Node::Collection(node) => {
                let Collection {
                    content,
                    id,
                    parts,
                    title,
                    ..
                } = node;
                BlockContent::Collection(CollectionSimple {
                    content,
                    id,
                    parts,
                    title,
                    ..Default::default()
                })
            }
            Node::Figure(node) => {
                let Figure {
                    caption,
                    content,
                    id,
                    label,
                    ..
                } = node;
                BlockContent::Figure(FigureSimple {
                    caption,
                    content,
                    id,
                    label,
                    ..Default::default()
                })
            }
            Node::Heading(node) => BlockContent::Heading(node),
            Node::Include(node) => BlockContent::Include(node),
            Node::List(node) => BlockContent::List(node),
            Node::MathBlock(node) => BlockContent::MathBlock(node),
            Node::Paragraph(node) => BlockContent::Paragraph(node),
            Node::QuoteBlock(node) => BlockContent::QuoteBlock(node),
            Node::Table(node) => {
                let Table {
                    caption,
                    id,
                    label,
                    rows,
                    ..
                } = node;
                BlockContent::Table(TableSimple {
                    caption,
                    id,
                    label,
                    rows,
                    ..Default::default()
                })
            }
            Node::ThematicBreak(node) => BlockContent::ThematicBreak(node),

            // Fallback to transforming to inline content and then block
            _ => self.to_inline().to_block(),
        }
    }

    /// Transform a `Node` variant to a vector of `BlockContent` variants
    fn to_blocks(&self) -> Vec<BlockContent> {
        match self.to_owned() {
            // Variants with block content
            Node::Article(Article { content, .. }) => match content {
                Some(content) => content,
                None => Vec::new(),
            },

            // Variants with inline content
            Node::Delete(node) => node.content.to_blocks(),
            Node::Emphasis(node) => node.content.to_blocks(),
            Node::Heading(node) => node.content.to_blocks(),
            Node::NontextualAnnotation(node) => node.content.to_blocks(),
            Node::Paragraph(node) => node.content.to_blocks(),
            Node::Quote(node) => node.content.to_blocks(),
            Node::Strikeout(node) => node.content.to_blocks(),
            Node::Strong(node) => node.content.to_blocks(),
            Node::Subscript(node) => node.content.to_blocks(),
            Node::Superscript(node) => node.content.to_blocks(),
            Node::Underline(node) => node.content.to_blocks(),

            // Fallback to a single item vector of self converted
            _ => vec![self.to_block()],
        }
    }

    /// Transform a `Node` variant to a `Node` variant
    ///
    /// Returns self.
    fn to_node(&self) -> Node {
        self.to_owned()
    }

    /// Transform a `Node` variant to a vector of static `InlineContent` variants
    fn to_static_inlines(&self) -> Vec<InlineContent> {
        if self.is_inline() {
            self.to_inline().to_static_inlines()
        } else {
            self.to_block().to_static_inlines()
        }
    }

    /// Transform a `Node` variant to a vector of static `BlockContent` variants
    fn to_static_blocks(&self) -> Vec<BlockContent> {
        if self.is_inline() {
            self.to_inline().to_static_blocks()
        } else {
            self.to_block().to_static_blocks()
        }
    }

    /// Transform a `Node` variant to a vector of static `Node` variants
    fn to_static_nodes(&self) -> Vec<Node> {
        if self.is_inline() {
            self.to_inline().to_static_inlines().to_nodes()
        } else {
            self.to_block().to_static_blocks().to_nodes()
        }
    }
}

impl Transform for Vec<Node> {
    /// Transform a vector of `Node` variants to a `InlineContent` variant
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

    /// Transform a vector of `Node` variants to a vector of `InlineContent` variants
    ///
    /// Returns self mapped into inlines.
    fn to_inlines(&self) -> Vec<InlineContent> {
        self.iter().flat_map(|node| node.to_inlines()).collect()
    }

    /// Transform a vector of `Node` variants to a `BlockContent` variant
    ///
    /// If there is just one item, returns that. Otherwise, wraps into a `CodeChunk`
    /// (one of the few node types that can hold a vector of nodes).
    fn to_block(&self) -> BlockContent {
        if self.len() == 1 {
            self[0].to_block()
        } else {
            BlockContent::CodeChunk(CodeChunk {
                outputs: Some(self.to_owned()),
                ..Default::default()
            })
        }
    }

    /// Transform a vector of `Node` variants to a vector of `BlockContent` variants
    ///
    /// Returns self mapped into blocks.
    fn to_blocks(&self) -> Vec<BlockContent> {
        self.iter().flat_map(|node| node.to_blocks()).collect()
    }

    /// Transform a vector of `Node` variants to a `Node` variant
    ///
    /// Wraps self into a `CodeChunk` with outputs being the vector of nodes.
    fn to_node(&self) -> Node {
        Node::CodeChunk(CodeChunk {
            outputs: Some(self.to_owned()),
            ..Default::default()
        })
    }

    /// Transform a vector of `Node` variants to a vector of `Node` variants
    ///
    /// Returns self.
    fn to_nodes(&self) -> Vec<Node> {
        self.to_owned()
    }

    /// Transform a vector of `Node` variants to a vector of static `InlineContent` variants
    fn to_static_inlines(&self) -> Vec<InlineContent> {
        self.iter()
            .flat_map(|node| node.to_static_inlines())
            .collect()
    }

    /// Transform a vector of `Node` variants to a vector of static `BlockContent` variants
    fn to_static_blocks(&self) -> Vec<BlockContent> {
        self.iter()
            .flat_map(|node| node.to_static_blocks())
            .collect()
    }

    /// Transform a vector of `Node` variants to a vector of static `Node` variants
    fn to_static_nodes(&self) -> Vec<Node> {
        self.iter()
            .flat_map(|node| node.to_static_nodes())
            .collect()
    }
}
