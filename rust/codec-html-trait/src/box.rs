use crate::{HtmlCodec, HtmlEncodeContext};

impl<T> HtmlCodec for Box<T>
where
    T: HtmlCodec,
{
    fn to_html(&self, context: &mut HtmlEncodeContext) -> String {
        self.as_ref().to_html(context)
    }

    fn to_html_parts(&self, context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        self.as_ref().to_html_parts(context)
    }

    fn to_html_attr(&self, context: &mut HtmlEncodeContext) -> String {
        self.as_ref().to_html_attr(context)
    }
}
