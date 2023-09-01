use crate::Object;

use super::prelude::*;

impl HtmlCodec for Object {
    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        (
            "ul",
            vec![attr("is", "stencila-object")],
            self.iter()
                .map(|(key, value)| {
                    elem(
                        "li",
                        &[attr("is", "stencila-object-item"), attr("key", key)],
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
