use crate::{encode::attr, HtmlCodec};

impl HtmlCodec for i64 {
    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        (
            "span",
            vec![attr("is", "stencila-integer")],
            vec![self.to_string()],
        )
    }

    fn to_html_attr(&self) -> String {
        self.to_string()
    }
}
