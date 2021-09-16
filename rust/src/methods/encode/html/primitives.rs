use super::{attr_itemtype_string, attr_slot, elem, json, Context, ToHtml};
use html_escape::encode_safe;
use stencila_schema::{Array, Boolean, Integer, Number, Object};

/// Encode a `Null` to HTML
pub(crate) fn null_to_html() -> String {
    elem("span", &[attr_itemtype_string("Null")], &"null".to_string())
}

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
atomic_to_html!(Boolean);
atomic_to_html!(Integer);
atomic_to_html!(Number);

/// Encode a `String` to HTML
///
/// This is the only node type where an `itemtype` attribute, in this case `http://schema.org/String`,
/// is NOT added to the element.
///
/// The string is escaped so that the generated HTML can be safely interpolated
/// within HTML.
impl ToHtml for String {
    fn to_html(&self, slot: &str, _context: &Context) -> String {
        elem("span", &[attr_slot(slot)], &encode_safe(self))
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
