use crate::TextValue;

use super::prelude::*;

impl HtmlCodec for TextValue {
    fn to_html(&self) -> String {
        text(&self.0)
    }

    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        unreachable!("should not be called for text value")
    }

    fn to_html_attr(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
