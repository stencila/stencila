use crate::{HtmlCodec, HtmlEncodeContext};

impl HtmlCodec for i64 {
    fn to_html_parts(&self, _context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        ("stencila-integer", vec![], vec![self.to_string()])
    }

    fn to_html_attr(&self, _context: &mut HtmlEncodeContext) -> String {
        self.to_string()
    }
}
