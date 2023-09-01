use crate::Array;

use super::prelude::*;

impl HtmlCodec for Array {
    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        (
            "ol",
            vec![attr("is", "stencila-array")],
            self.iter()
                .map(|value| {
                    elem(
                        "li",
                        &[attr("is", "stencila-array-item")],
                        &[value.to_html()],
                    )
                })
                .collect_vec(),
        )
    }

    fn to_html_attr(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}
