//! Encode `Primitive` nodes to HTML

use super::{attr_itemtype_str, attr_slot, concat, elem, EncodeContext, ToHtml};
use html_escape::encode_safe;
use node_dispatch::dispatch_primitive;
use stencila_schema::*;

impl ToHtml for Primitive {
    fn to_html(&self, context: &EncodeContext) -> String {
        // Call `array_to_html` to avoid `Vec<Primitive>.to_html()` for arrays
        if let Primitive::Array(array) = self {
            return array_to_html(array, context);
        }

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

// Encode an `Array` to the HTML semantically equivalent `<ol>`.
//
// We can't implement `ToHtml` for `Array` as that conflicts with `impl ToHtml for `Vec<Primitive>`.
// Instead this function is used to provide necessary structure for patching to work on arrays. Note that
// arrays have special handling in the `../web/patches` TypeScript so changes made to the HTML structure
// will need concomitant changes there.
#[allow(clippy::ptr_arg)]
pub fn array_to_html(array: &Array, context: &EncodeContext) -> String {
    let items = concat(array, |item| elem("li", &[], &item.to_html(context)));
    elem(
        "stencila-array",
        &[attr_itemtype_str("Array")],
        &elem("ol", &[attr_slot("items")], &items),
    )
}

/// Encode an `Object` to the HTML semantically equivalent `<dl>`.
///
/// Note that objects have special handling in the `../web/patches` TypeScript so changes made to
/// the HTML structure will need concomitant changes there.
impl ToHtml for Object {
    fn to_html(&self, context: &EncodeContext) -> String {
        let pairs = self
            .iter()
            .map(|(key, value)| {
                [
                    elem("dt", &[], &key.to_html(context)),
                    elem("dd", &[], &value.to_html(context)),
                ]
                .concat()
            })
            .collect::<Vec<String>>()
            .concat();
        elem(
            "stencila-object",
            &[attr_itemtype_str("Object")],
            &elem("dl", &[attr_slot("pairs")], &pairs),
        )
    }
}
