//! Encode `Primitive` nodes to HTML

use html_escape::encode_safe;
use node_dispatch::dispatch_primitive;
use stencila_schema::*;

use crate::encode::attr;

use super::{
    attr_itemtype, attr_itemtype_str, attr_prop, attr_slot, concat, elem, EncodeContext, ToHtml,
};

impl ToHtml for Primitive {
    fn to_html(&self, context: &mut EncodeContext) -> String {
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
            fn to_html(&self, _context: &mut EncodeContext) -> String {
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

atomic_to_html!(u32);

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
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        encode_safe(self).to_string()
    }
}

/// Encode a `Date` to HTML
impl ToHtml for Date {
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        // To allow for alternative formatting the Date could be decomposed as follows.
        // However, for now, keeping things simple by just encoding the raw value.
        /*
        use chrono::Datelike;
        let content = match chrono::NaiveDate::parse_from_str(&self.value, "%Y-%m-%d") {
            Ok(datetime) => [
                elem("span", &[], &datetime.year().to_string()),
                elem("span", &[], &datetime.month().to_string()),
                elem("span", &[], &datetime.day().to_string()),
            ]
            .concat(),
            Err(error) => {
                tracing::warn!("While parsing `Date` value `{}`: {}", self.value, error);
                self.value.clone()
            }
        };
        */
        elem(
            "span",
            &[],
            &elem(
                "time",
                &[
                    attr_itemtype::<Self>(),
                    attr_prop("value"),
                    attr("datetime", &self.value),
                ],
                &self.value,
            ),
        )
    }
}

/// Encode a `Time` to HTML
impl ToHtml for Time {
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        // As for `Date` this could be broken into parts but for now is kept simple
        elem(
            "span",
            &[],
            &elem(
                "time",
                &[
                    attr_itemtype::<Self>(),
                    attr_prop("value"),
                    attr("datetime", &self.value),
                ],
                &self.value,
            ),
        )
    }
}

/// Encode a `DateTime` to HTML
impl ToHtml for DateTime {
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        // As for `Date` this could be broken into parts but for now is kept simple
        elem(
            "span",
            &[],
            &elem(
                "time",
                &[
                    attr_itemtype::<Self>(),
                    attr_prop("value"),
                    attr("datetime", &self.value),
                ],
                &self.value,
            ),
        )
    }
}

/// Encode a `Timestamp` to HTML
impl ToHtml for Timestamp {
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        let iso8601 = self.to_iso8601().unwrap_or_else(|_| self.value.to_string());
        elem(
            "stencila-timestamp",
            &[
                attr("value", &self.value.to_string()),
                attr("time-unit", self.time_unit.as_ref()),
                attr("datetime", &iso8601),
            ],
            &iso8601,
        )
    }
}

/// Encode a `Duration` to HTML
impl ToHtml for Duration {
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        let content = self.humanize();
        elem(
            "stencila-duration",
            &[
                attr("value", &self.value.to_string()),
                attr("time-unit", self.time_unit.as_ref()),
            ],
            &content,
        )
    }
}

// Encode an `Array` to the HTML semantically equivalent `<ol>`, or if inline, to a `<span>`.
//
// We can't implement `ToHtml` for `Array` as that conflicts with `impl ToHtml for `Vec<Primitive>`.
// Instead this function is used to provide necessary structure for patching to work on arrays. Note that
// arrays have special handling in the `../web/patches` TypeScript so changes made to the HTML structure
// will need concomitant changes there.
#[allow(clippy::ptr_arg)]
pub fn array_to_html(array: &Array, context: &mut EncodeContext) -> String {
    let (container_tag, item_tag) = match context.inline {
        true => ("span", "span"),
        false => ("ol", "li"),
    };
    let items = concat(array, |item| elem(item_tag, &[], &item.to_html(context)));
    elem(
        "stencila-array",
        &[attr_itemtype_str("Array")],
        &elem(container_tag, &[attr_slot("items")], &items),
    )
}

/// Encode an `Object` to the HTML semantically equivalent `<dl>`, or if inline, to a `<span>`.
///
/// Note that objects have special handling in the `../web/patches` TypeScript so changes made to
/// the HTML structure will need concomitant changes there.
impl ToHtml for Object {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let (container_tag, key_tag, value_tag) = match context.inline {
            true => ("span", "span", "span"),
            false => ("dl", "dt", "dd"),
        };
        let pairs = self
            .iter()
            .map(|(key, value)| {
                [
                    elem(key_tag, &[], &key.to_html(context)),
                    elem(value_tag, &[], &value.to_html(context)),
                ]
                .concat()
            })
            .collect::<Vec<String>>()
            .concat();
        elem(
            "stencila-object",
            &[attr_itemtype_str("Object")],
            &elem(container_tag, &[attr_slot("pairs")], &pairs),
        )
    }
}
