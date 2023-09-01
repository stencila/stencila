use crate::Null;

use super::prelude::*;

impl HtmlCodec for Null {
    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        (
            "span",
            vec![attr("is", "stencila-null")],
            vec!["null".to_string()],
        )
    }

    fn to_html_attr(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
