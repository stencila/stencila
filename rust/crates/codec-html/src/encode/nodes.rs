//! Encode `Node` nodes to HTML

use super::{attr, elem, json, EncodeContext, ToHtml};
use stencila_schema::Node;

/// Encode a `Node` to HTML
///
/// All node types that have an `impl ToHtml` should be listed here. Not all node types
/// are supported, in which case this function returns HTML indicating that that is the case.
impl ToHtml for Node {
    fn to_html(&self, slot: &str, context: &EncodeContext) -> String {
        match self {
            Node::Array(node) => node.to_html(slot, context),
            Node::Article(node) => node.to_html(slot, context),
            Node::AudioObject(node) => node.to_html(slot, context),
            Node::Boolean(node) => node.to_html(slot, context),
            Node::Cite(node) => node.to_html(slot, context),
            Node::CiteGroup(node) => node.to_html(slot, context),
            Node::Claim(node) => node.to_html(slot, context),
            Node::CodeBlock(node) => node.to_html(slot, context),
            Node::CodeChunk(node) => node.to_html(slot, context),
            Node::CodeExpression(node) => node.to_html(slot, context),
            Node::CodeFragment(node) => node.to_html(slot, context),
            Node::Delete(node) => node.to_html(slot, context),
            Node::Emphasis(node) => node.to_html(slot, context),
            Node::Figure(node) => node.to_html(slot, context),
            Node::Heading(node) => node.to_html(slot, context),
            Node::ImageObject(node) => node.to_html(slot, context),
            Node::Integer(node) => node.to_html(slot, context),
            Node::Link(node) => node.to_html(slot, context),
            Node::List(node) => node.to_html(slot, context),
            Node::MathBlock(node) => node.to_html(slot, context),
            Node::MathFragment(node) => node.to_html(slot, context),
            Node::NontextualAnnotation(node) => node.to_html(slot, context),
            Node::Note(node) => node.to_html(slot, context),
            Node::Null(node) => node.to_html(slot, context),
            Node::Number(node) => node.to_html(slot, context),
            Node::Object(node) => node.to_html(slot, context),
            Node::Paragraph(node) => node.to_html(slot, context),
            Node::Quote(node) => node.to_html(slot, context),
            Node::QuoteBlock(node) => node.to_html(slot, context),
            Node::String(node) => node.to_html(slot, context),
            Node::Strong(node) => node.to_html(slot, context),
            Node::Subscript(node) => node.to_html(slot, context),
            Node::Superscript(node) => node.to_html(slot, context),
            Node::Table(node) => node.to_html(slot, context),
            Node::ThematicBreak(node) => node.to_html(slot, context),
            Node::VideoObject(node) => node.to_html(slot, context),
            _ => elem("div", &[attr("class", "unsupported")], &json(self)),
        }
    }
}
