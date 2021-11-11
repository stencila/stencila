//! Encode `Node` nodes to HTML

use super::{attr, attr_itemtype_str, elem, json, EncodeContext, ToHtml};
use stencila_schema::Node;

/// Encode a `Node` to HTML
///
/// All node types that have an `impl ToHtml` should be listed here. Not all node types
/// are supported, in which case this function returns HTML indicating that that is the case.
impl ToHtml for Node {
    fn to_html(&self, context: &EncodeContext) -> String {
        match self {
            Node::Array(node) => node.to_html(context),
            Node::Article(node) => node.to_html(context),
            Node::AudioObject(node) => node.to_html(context),
            Node::Boolean(node) => node.to_html(context),
            Node::Cite(node) => node.to_html(context),
            Node::CiteGroup(node) => node.to_html(context),
            Node::Claim(node) => node.to_html(context),
            Node::CodeBlock(node) => node.to_html(context),
            Node::CodeChunk(node) => node.to_html(context),
            Node::CodeExpression(node) => node.to_html(context),
            Node::CodeFragment(node) => node.to_html(context),
            Node::Delete(node) => node.to_html(context),
            Node::Emphasis(node) => node.to_html(context),
            Node::Figure(node) => node.to_html(context),
            Node::Heading(node) => node.to_html(context),
            Node::ImageObject(node) => node.to_html(context),
            Node::Integer(node) => node.to_html(context),
            Node::Link(node) => node.to_html(context),
            Node::List(node) => node.to_html(context),
            Node::MathBlock(node) => node.to_html(context),
            Node::MathFragment(node) => node.to_html(context),
            Node::NontextualAnnotation(node) => node.to_html(context),
            Node::Note(node) => node.to_html(context),
            Node::Null(node) => node.to_html(context),
            Node::Number(node) => node.to_html(context),
            Node::Object(node) => node.to_html(context),
            Node::Paragraph(node) => node.to_html(context),
            Node::Quote(node) => node.to_html(context),
            Node::QuoteBlock(node) => node.to_html(context),
            // Wrap strings with the `itemtype` attribute (see note under `ToHtml` for `InlineContent`)
            // This encoding will be used in places such as `CodeChunk.outputs`, `CodeExpression.output` etc
            Node::String(node) => {
                elem("span", &[attr_itemtype_str("Text")], &node.to_html(context))
            }
            Node::Strong(node) => node.to_html(context),
            Node::Subscript(node) => node.to_html(context),
            Node::Superscript(node) => node.to_html(context),
            Node::Table(node) => node.to_html(context),
            Node::ThematicBreak(node) => node.to_html(context),
            Node::VideoObject(node) => node.to_html(context),
            _ => elem("div", &[attr("class", "unsupported")], &json(self)),
        }
    }
}
