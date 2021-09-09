use stencila_schema::{
    AudioObject, AudioObjectSimple, BlockContent, Claim, ClaimSimple, Collection, CollectionSimple,
    Figure, FigureSimple, ImageObject, ImageObjectSimple, InlineContent, Node, Table, TableSimple,
    VideoObject, VideoObjectSimple,
};

pub trait ToNode {
    /// Convert a `Node` subtype enum (e.g. `BlockContent::Strong`) to a `Node` enum (e.g. `Node::Strong`)
    fn to_node(self) -> Node;
}

impl ToNode for InlineContent {
    /// Convert an `InlineContent` node to a `Node`.
    ///
    /// Most variants can be converted directly. However, `CreativeWork` types that have
    /// simple inline variants need "upcasting" to their more complex types.
    fn to_node(self) -> Node {
        match self {
            InlineContent::AudioObject(node) => {
                let AudioObjectSimple {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    title,
                    transcript,
                    type_: _type,
                } = node;
                Node::AudioObject(AudioObject {
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
            InlineContent::Boolean(node) => Node::Boolean(node),
            InlineContent::Cite(node) => Node::Cite(node),
            InlineContent::CiteGroup(node) => Node::CiteGroup(node),
            InlineContent::CodeExpression(node) => Node::CodeExpression(node),
            InlineContent::CodeFragment(node) => Node::CodeFragment(node),
            InlineContent::Delete(node) => Node::Delete(node),
            InlineContent::Emphasis(node) => Node::Emphasis(node),
            InlineContent::ImageObject(node) => {
                let ImageObjectSimple {
                    bitrate,
                    caption,
                    content_size,
                    content_url,
                    embed_url,
                    id,
                    media_type,
                    thumbnail,
                    title,
                    type_: _type,
                } = node;
                Node::ImageObject(ImageObject {
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
            InlineContent::Integer(node) => Node::Integer(node),
            InlineContent::Link(node) => Node::Link(node),
            InlineContent::MathFragment(node) => Node::MathFragment(node),
            InlineContent::NontextualAnnotation(node) => Node::NontextualAnnotation(node),
            InlineContent::Note(node) => Node::Note(node),
            InlineContent::Null => Node::Null,
            InlineContent::Number(node) => Node::Number(node),
            InlineContent::Parameter(node) => Node::Parameter(node),
            InlineContent::Quote(node) => Node::Quote(node),
            InlineContent::String(node) => Node::String(node),
            InlineContent::Strong(node) => Node::Strong(node),
            InlineContent::Subscript(node) => Node::Subscript(node),
            InlineContent::Superscript(node) => Node::Superscript(node),
            InlineContent::VideoObject(node) => {
                let VideoObjectSimple {
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
                    type_: _type,
                } = node;
                Node::VideoObject(VideoObject {
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
        }
    }
}

impl ToNode for BlockContent {
    /// Convert an `BlockContent` node to a `Node`.
    ///
    /// Most variants can be converted directly. However, `CreativeWork` types that have
    /// simple block variants need "upcasting" to their more complex types.
    fn to_node(self) -> Node {
        match self {
            BlockContent::Claim(node) => {
                let ClaimSimple {
                    claim_type,
                    content,
                    id,
                    label,
                    parts,
                    title,
                    type_: _type,
                } = node.clone();
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
                } = node.clone();
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
                    parts,
                    title,
                    type_: _type,
                } = node.clone();
                Node::Figure(Figure {
                    caption,
                    content,
                    id,
                    label,
                    parts,
                    title,
                    ..Default::default()
                })
            }
            BlockContent::Heading(node) => Node::Heading(node),
            BlockContent::Include(node) => Node::Include(node),
            BlockContent::List(node) => Node::List(node),
            BlockContent::MathBlock(node) => Node::MathBlock(node),
            BlockContent::Paragraph(node) => Node::Paragraph(node),
            BlockContent::QuoteBlock(node) => Node::QuoteBlock(node),
            BlockContent::Table(table_simple) => {
                let TableSimple {
                    caption,
                    content,
                    id,
                    label,
                    parts,
                    rows,
                    title,
                    type_: _type,
                } = table_simple.clone();
                Node::Table(Table {
                    caption,
                    content,
                    id,
                    label,
                    parts,
                    rows,
                    title,
                    ..Default::default()
                })
            }
            BlockContent::ThematicBreak(node) => Node::ThematicBreak(node),
        }
    }
}
