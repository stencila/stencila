use crate::{HtmlCodec, HtmlEncodeContext};

impl HtmlCodec for u64 {
    fn to_html_parts(&self, _context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        ("stencila-unsigned-integer", vec![], vec![self.to_string()])
    }

    fn to_html_attr(&self, _context: &mut HtmlEncodeContext) -> String {
        self.to_string()
    }
}
