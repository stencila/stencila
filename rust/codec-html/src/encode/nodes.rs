//! Encode `Node` nodes to HTML

use super::{
    attr, attr_itemtype_str, elem, json, primitives::array_to_html, EncodeContext, ToHtml,
};
use node_dispatch::dispatch_node;
use stencila_schema::Node;

/// Encode a `Node` to HTML
///
/// Not all node types have `impl ToHtml` in which case this function
/// returns HTML indicating that that is the case.
impl ToHtml for Node {
    fn to_html(&self, context: &EncodeContext) -> String {
        // Call `array_to_html` to avoid `Vec<Primitive>.to_html()` for arrays
        if let Node::Array(array) = self {
            return array_to_html(array, context);
        }

        // In `CodeChunk` outputs we need to use <pre> so that newlines are preserved and in
        // `CodeExpression` output we must use a <span>.
        // See note under `ToHtml` for `InlineContent` for how strings are handled in that context.
        if let Node::String(string) = self {
            return elem(
                if context.inline { "span" } else { "pre" },
                &[attr_itemtype_str("String")],
                &string.to_html(context),
            );
        }

        // Fallback to default `to_html` for other `Node` variants
        dispatch_node!(
            self,
            elem("div", &[attr("class", "unsupported")], &json(self)),
            to_html,
            context
        )
    }
}
