use super::Transform;
use codec_txt::ToTxt;
use stencila_schema::*;

impl Transform for InlineContent {
    /// Is a node an `InlineContent` variant e.g. a `Node:Strong`
    fn is_inline(&self) -> bool {
        true
    }

    /// Transform an `InlineContent` variant to a `InlineContent` variant
    ///
    /// Returns self.
    fn to_inline(&self) -> InlineContent {
        self.to_owned()
    }

    /// Transform an `InlineContent` variant to a `BlockContent` variant
    ///
    /// Returns self wrapped into a paragraph.
    fn to_block(&self) -> BlockContent {
        BlockContent::Paragraph(Paragraph {
            content: self.to_inlines(),
            ..Default::default()
        })
    }

    /// Transform an `InlineContent` variant to a `Node` variant
    ///
    /// Most variants can be converted directly. However, `CreativeWork` types that have
    /// simple inline variants need "upcasting" to their more complex types.
    fn to_node(&self) -> Node {
        match self.to_owned() {
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
            InlineContent::Date(node) => Node::Date(node),
            InlineContent::DateTime(node) => Node::DateTime(node),
            InlineContent::Delete(node) => Node::Delete(node),
            InlineContent::Duration(node) => Node::Duration(node),
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
            InlineContent::Null(node) => Node::Null(node),
            InlineContent::Number(node) => Node::Number(node),
            InlineContent::Parameter(node) => Node::Parameter(node),
            InlineContent::Quote(node) => Node::Quote(node),
            InlineContent::Strikeout(node) => Node::Strikeout(node),
            InlineContent::String(node) => Node::String(node),
            InlineContent::Strong(node) => Node::Strong(node),
            InlineContent::Subscript(node) => Node::Subscript(node),
            InlineContent::Superscript(node) => Node::Superscript(node),
            InlineContent::Time(node) => Node::Time(node),
            InlineContent::Timestamp(node) => Node::Timestamp(node),
            InlineContent::Underline(node) => Node::Underline(node),
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

    /// Transform an `InlineContent` variant to a vector of static `InlineContent` variants
    fn to_static_inlines(&self) -> Vec<InlineContent> {
        match self.to_owned() {
            // Dynamic node types: only include their "outputs", if any
            InlineContent::CodeExpression(node) => node
                .output
                .as_ref()
                .map(|value| vec![value.to_inline()])
                .unwrap_or_default(),
            InlineContent::Parameter(node) => node
                .value
                .as_ref()
                .or(node.default.as_ref())
                .map(|value| vec![value.to_inline()])
                .unwrap_or_default(),

            // Non-dynamic node types: make their content static
            InlineContent::Delete(node) => vec![InlineContent::Delete(Delete {
                content: node.content.to_static_inlines(),
                ..node
            })],
            InlineContent::Emphasis(node) => vec![InlineContent::Emphasis(Emphasis {
                content: node.content.to_static_inlines(),
                ..node
            })],
            InlineContent::NontextualAnnotation(node) => {
                vec![InlineContent::NontextualAnnotation(NontextualAnnotation {
                    content: node.content.to_static_inlines(),
                    ..node
                })]
            }
            InlineContent::Quote(node) => vec![InlineContent::Quote(Quote {
                content: node.content.to_static_inlines(),
                ..node
            })],
            InlineContent::Strikeout(node) => vec![InlineContent::Strikeout(Strikeout {
                content: node.content.to_static_inlines(),
                ..node
            })],
            InlineContent::Strong(node) => vec![InlineContent::Strong(Strong {
                content: node.content.to_static_inlines(),
                ..node
            })],
            InlineContent::Subscript(node) => vec![InlineContent::Subscript(Subscript {
                content: node.content.to_static_inlines(),
                ..node
            })],
            InlineContent::Superscript(node) => vec![InlineContent::Superscript(Superscript {
                content: node.content.to_static_inlines(),
                ..node
            })],
            InlineContent::Underline(node) => vec![InlineContent::Underline(Underline {
                content: node.content.to_static_inlines(),
                ..node
            })],
            _ => self.to_inlines(),
        }
    }
}

impl Transform for Vec<InlineContent> {
    /// Transform a vector of `InlineContent` variants to a `InlineContent` variant
    ///
    /// If there is just one item, returns that. Otherwise, returns a string using the `ToTxt` trait.
    fn to_inline(&self) -> InlineContent {
        if self.len() == 1 {
            self[0].to_owned()
        } else {
            InlineContent::String(self.to_txt())
        }
    }

    /// Transform a vector of `InlineContent` variants to a vector of `InlineContent` variants
    ///
    /// Returns self.
    fn to_inlines(&self) -> Vec<InlineContent> {
        self.to_owned()
    }

    /// Transform a vector of `InlineContent` variants to a `BlockContent` variant
    ///
    /// Returns self wrapped by a paragraph.
    fn to_block(&self) -> BlockContent {
        BlockContent::Paragraph(Paragraph {
            content: self.to_owned(),
            ..Default::default()
        })
    }

    /// Transform a vector of `InlineContent` variants to a `Node` variant
    ///
    /// Returns self wrapped by a paragraph.
    fn to_node(&self) -> Node {
        Node::Paragraph(Paragraph {
            content: self.to_owned(),
            ..Default::default()
        })
    }

    /// Transform a vector of `InlineContent` variants to a vector of static `InlineContent` variants
    fn to_static_inlines(&self) -> Vec<InlineContent> {
        self.iter()
            .flat_map(|inline| inline.to_static_inlines())
            .collect()
    }

    /// Transform a vector of `InlineContent` variants to a vector of static `BlockContent` variants
    fn to_static_blocks(&self) -> Vec<BlockContent> {
        self.iter()
            .flat_map(|inline| inline.to_static_blocks())
            .collect()
    }

    /// Transform a vector of `InlineContent` variants to a vector of static `Node` variants
    fn to_static_nodes(&self) -> Vec<Node> {
        self.iter()
            .flat_map(|inline| inline.to_static_nodes())
            .collect()
    }
}
