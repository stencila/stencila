use common::serde_json;

use crate::{encode::text, HtmlCodec};

impl HtmlCodec for String {
    fn to_html(&self) -> String {
        text(self)
    }

    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        unreachable!("should not be called for a string")
    }

    fn to_html_attr(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
