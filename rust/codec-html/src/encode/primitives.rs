//! Encode `Primitive` nodes to HTML

use super::{attr_itemtype_str, elem, json, EncodeContext, ToHtml};
use html_escape::encode_safe;
use node_dispatch::dispatch_primitive;
use stencila_schema::*;

impl ToHtml for Primitive {
    fn to_html(&self, context: &EncodeContext) -> String {
        dispatch_primitive!(self, to_html, context)
    }
}

/// Encode an atomic primitive to HTML
macro_rules! atomic_to_html {
    ($type:ty) => {
        impl ToHtml for $type {
            fn to_html(&self, _context: &EncodeContext) -> String {
                elem(
                    "span",
                    &[attr_itemtype_str(stringify!($type))],
                    &self.to_string(),
                )
            }
        }
    };
}
atomic_to_html!(Null);
atomic_to_html!(Boolean);
atomic_to_html!(Integer);
atomic_to_html!(Number);

/// Encode a `String` to HTML
///
/// This is the only `Node` type that is NOT represented by an element
/// (with an `itemtype` attribute, which in this case would be `https://schema.org/Text`).
/// This reduces the size of the generated HTML (whole page and in patches), but is also
/// useful in applying [`Operation`]s in the `web` module because it allows discrimination
/// between strings and other node types.
///
/// The string is escaped so that the generated HTML can be safely interpolated within HTML.
impl ToHtml for String {
    fn to_html(&self, _context: &EncodeContext) -> String {
        encode_safe(self).to_string()
    }
}

// Encoding an `Array` to HTML is implemented by `Vec<Primitive>.to_html()`

/// Encode an `Object` to HTML
impl ToHtml for Object {
    fn to_html(&self, _context: &EncodeContext) -> String {
        elem("code", &[attr_itemtype_str("Object")], &json(self))
    }
}
