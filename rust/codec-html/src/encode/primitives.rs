//! Encode `Primitive` nodes to HTML

use super::{attr_itemtype_str, attr_prop, elem, json, EncodeContext, ToHtml};
use html_escape::encode_safe;
use stencila_schema::{Array, Boolean, Integer, Null, Number, Object};

/// Encode an atomic primitive to HTML
macro_rules! atomic_to_html {
    ($type:ty) => {
        impl ToHtml for $type {
            fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
                elem(
                    "span",
                    &[attr_prop(slot), attr_itemtype_str(stringify!($type))],
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
/// The string is escaped so that the generated HTML can be safely interpolated within HTML.
impl ToHtml for String {
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        elem(
            "span",
            &[attr_prop(slot), attr_itemtype_str("Text")],
            &encode_safe(self).to_string(),
        )
    }
}

/// Encode an `Array` to HTML
impl ToHtml for Array {
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        elem(
            "code",
            &[attr_prop(slot), attr_itemtype_str("Array")],
            &json(self),
        )
    }
}

/// Encode an `Object` to HTML
impl ToHtml for Object {
    fn to_html(&self, slot: &str, _context: &EncodeContext) -> String {
        elem(
            "code",
            &[attr_prop(slot), attr_itemtype_str("Object")],
            &json(self),
        )
    }
}
