use super::Transform;
use codec_txt::ToTxt;
use stencila_schema::*;

impl Transform for Primitive {
    /// Transform a `Primitive` variant to a `InlineContent` variant
    fn to_inline(&self) -> InlineContent {
        match self.to_owned() {
            Primitive::Null(node) => InlineContent::Null(node),
            Primitive::Boolean(node) => InlineContent::Boolean(node),
            Primitive::Integer(node) => InlineContent::Integer(node),
            Primitive::Number(node) => InlineContent::Number(node),
            Primitive::String(node) => InlineContent::String(node),
            Primitive::Date(node) => InlineContent::Date(node),
            Primitive::Time(node) => InlineContent::Time(node),
            Primitive::DateTime(node) => InlineContent::DateTime(node),
            Primitive::Timestamp(node) => InlineContent::Timestamp(node),
            Primitive::Duration(node) => InlineContent::Duration(node),
            Primitive::Array(node) => InlineContent::String(node.to_txt()),
            Primitive::Object(node) => InlineContent::String(node.to_txt()),
        }
    }

    /// Transform a `Primitive` variant to a `BlockContent` variant
    fn to_block(&self) -> BlockContent {
        BlockContent::Paragraph(Paragraph {
            content: self.to_inlines(),
            ..Default::default()
        })
    }

    /// Transform a `Primitive` variant to a `Node` variant
    fn to_node(&self) -> Node {
        match self.to_owned() {
            Primitive::Null(node) => Node::Null(node),
            Primitive::Boolean(node) => Node::Boolean(node),
            Primitive::Integer(node) => Node::Integer(node),
            Primitive::Number(node) => Node::Number(node),
            Primitive::String(node) => Node::String(node),
            Primitive::Date(node) => Node::Date(node),
            Primitive::Time(node) => Node::Time(node),
            Primitive::DateTime(node) => Node::DateTime(node),
            Primitive::Timestamp(node) => Node::Timestamp(node),
            Primitive::Duration(node) => Node::Duration(node),
            Primitive::Array(node) => Node::Array(node),
            Primitive::Object(node) => Node::Object(node),
        }
    }
}
