use super::{attr_itemtype_string, attr_slot, elem, json, Context, ToHtml};
use html_escape::encode_safe;
use stencila_schema::{Array, Boolean, Integer, Null, Number, Object};

/// Encode an atomic primitive to HTML
macro_rules! atomic_to_html {
    ($type:ident) => {
        impl ToHtml for $type {
            fn to_html(&self, slot: &str, _context: &Context) -> String {
                elem(
                    "span",
                    &[attr_slot(slot), attr_itemtype_string(stringify!($type))],
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
/// This reduces the size of the generated HTML, but is also useful in applying [`DomOperation`]s
/// in the `web` module because it allows discrimination between strings and other node types.
///
/// The string is escaped so that the generated HTML can be safely interpolated within HTML.
impl ToHtml for String {
    fn to_html(&self, _slot: &str, _context: &Context) -> String {
        encode_safe(self).to_string()
    }
}

/// Encode an `Array` to HTML
impl ToHtml for Array {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem(
            "code",
            &[attr_slot(slot), attr_itemtype_string("Array")],
            &json(self),
        )
    }
}

/// Encode an `Object` to HTML
impl ToHtml for Object {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem(
            "code",
            &[attr_slot(slot), attr_itemtype_string("Object")],
            &json(self),
        )
    }
}
