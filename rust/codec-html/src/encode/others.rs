use super::{attr, elem, EncodeContext, ToHtml};
use chrono::{DateTime, Datelike};
use stencila_schema::*;

/// Encode a `Date` to HTML
///
/// Takes a similar approach to the encoding of `Cite` nodes in that it encodes parts
/// of the date as spans which the theme can choose to reorder and/or hide.
impl ToHtml for Date {
    fn to_html(&self, _context: &EncodeContext) -> String {
        let content = match DateTime::parse_from_rfc3339(&self.value) {
            Ok(datetime) => [
                elem("span", &[], &datetime.year().to_string()),
                elem("span", &[], &datetime.month().to_string()),
                elem("span", &[], &datetime.day().to_string()),
            ]
            .concat(),
            Err(error) => {
                tracing::warn!("While parsing date `{}`: {}", self.value, error);
                self.value.clone()
            }
        };
        elem("time", &[attr("datetime", &self.value)], &content)
    }
}
