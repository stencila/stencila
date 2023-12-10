use crate::HtmlCodec;

impl HtmlCodec for i64 {
    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        ("stencila-integer", vec![], vec![self.to_string()])
    }

    fn to_html_attr(&self) -> String {
        self.to_string()
    }
}
