//! Encode `Primitive` nodes to HTML

use html_escape::encode_safe;
use node_dispatch::dispatch_primitive;
use stencila_schema::*;
use suids::Suid;

use crate::encode::attr;

use super::{attr_json, attr_prop, attr_slot, concat, elem, EncodeContext, ToHtml};

impl ToHtml for Primitive {
    /// Encode a `Primitive` sum type as HTML
    ///
    /// Calls `array_to_html` to avoid `Vec<Primitive>.to_html()` for arrays.
    /// Otherwise, simply dispatches to the implementation for the variant type.
    fn to_html(&self, context: &mut EncodeContext) -> String {
        if let Primitive::Array(array) = self {
            return array_to_html(array, context);
        }

        dispatch_primitive!(self, to_html, context)
    }

    /// Encode a `Primitive` sum type as an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        dispatch_primitive!(self, to_attr, name)
    }
}

/// Encode an atomic primitive to HTML element or attribute
macro_rules! atomic_to_html {
    ($type:ty) => {
        impl ToHtml for $type {
            fn to_html(&self, _context: &mut EncodeContext) -> String {
                elem("span", &[], &self.to_string())
            }

            fn to_attr(&self, name: &str) -> String {
                attr(name, &self.to_string())
            }
        }
    };
}
atomic_to_html!(Null);
atomic_to_html!(Boolean);
atomic_to_html!(Integer);
atomic_to_html!(Number);

atomic_to_html!(u32);

impl ToHtml for String {
    /// Encode a string as HTML
    ///
    /// The string is escaped so that the generated HTML can be safely interpolated within HTML.
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        encode_safe(self).to_string()
    }

    /// Encode a string as an HTML element attribute
    ///
    /// Note that the `attr` function does escaping so there is no need to do it here.
    fn to_attr(&self, name: &str) -> String {
        attr(name, self)
    }
}

impl ToHtml for Suid {
    /// Encode a string as HTML
    ///
    /// The string is escaped so that the generated HTML can be safely interpolated within HTML.
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        encode_safe(self).to_string()
    }

    /// Encode a string as an HTML element attribute
    ///
    /// Note that the `attr` function does escaping so there is no need to do it here.
    fn to_attr(&self, name: &str) -> String {
        attr(name, self)
    }
}

impl ToHtml for Date {
    /// Encode a `Date` to HTML
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
                &[attr_prop("value"), attr("datetime", &self.value)],
                &self.value,
            ),
        )
    }

    /// Encode a `Date` to an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        attr_json(name, self)
    }
}

impl ToHtml for Time {
    /// Encode a `Time` to HTML
    ///
    /// As for `Date` this could be broken into parts but for now is kept simple
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "span",
            &[],
            &elem(
                "time",
                &[attr_prop("value"), attr("datetime", &self.value)],
                &self.value,
            ),
        )
    }

    /// Encode a `Time` to an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        attr_json(name, self)
    }
}

impl ToHtml for DateTime {
    /// Encode a `DateTime` to HTML
    ///
    /// As for `Date` this could be broken into parts but for now is kept simple
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        elem(
            "span",
            &[],
            &elem(
                "time",
                &[attr_prop("value"), attr("datetime", &self.value)],
                &self.value,
            ),
        )
    }

    /// Encode a `DateTime` to an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        attr_json(name, self)
    }
}

impl ToHtml for Timestamp {
    /// Encode a `Timestamp` to HTML
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        let iso8601 = self.to_iso8601().unwrap_or_else(|_| self.value.to_string());
        elem(
            "stencila-timestamp",
            &[
                self.value.to_attr("value"),
                self.time_unit.to_attr("time_unit"),
                attr("datetime", &iso8601),
            ],
            &iso8601,
        )
    }

    /// Encode a `Timestamp` to an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        attr_json(name, self)
    }
}

impl ToHtml for Duration {
    /// Encode a `Duration` to HTML
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        let content = self.humanize();
        elem(
            "stencila-duration",
            &[
                self.value.to_attr("value"),
                self.time_unit.to_attr("time_unit"),
            ],
            &content,
        )
    }

    /// Encode a `Duration` to an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        attr_json(name, self)
    }
}

impl ToHtml for TimeUnit {
    /// Encode a `Timestamp` to HTML
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        self.as_ref().to_string()
    }

    /// Encode a `TimeUnit` to an HTML element attribute
    fn to_attr(&self, name: &str) -> String {
        attr(name, self.as_ref())
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
        &[],
        &elem(container_tag, &[attr_slot("items")], &items),
    )
}

impl ToHtml for Object {
    /// Encode an `Object` to the HTML semantically equivalent `<dl>`, or if inline, to a `<span>`.
    ///
    /// Note that objects have special handling in the `../web/patches` TypeScript so changes made to
    /// the HTML structure will need concomitant changes there.
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
            &[],
            &elem(container_tag, &[attr_slot("pairs")], &pairs),
        )
    }
}
