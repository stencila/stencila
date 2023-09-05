use crate::Array;

use super::prelude::*;

impl HtmlCodec for Array {
    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        // Uses spans, rather than say <ol>/<li> because needs to be
        // include e.g for output of a `CodeExpression`
        (
            "span",
            vec![attr("is", "stencila-array")],
            self.iter()
                .map(|value| {
                    elem(
                        "span",
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
