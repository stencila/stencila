use crate::{HtmlCodec, HtmlEncodeContext};

impl<T> HtmlCodec for Option<T>
where
    T: HtmlCodec,
{
    fn to_html(&self, context: &mut HtmlEncodeContext) -> String {
        self.as_ref()
            .map(|value| value.to_html(context))
            .unwrap_or_default()
    }

    fn to_html_parts(&self, context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        self.as_ref()
            .map(|value| value.to_html_parts(context))
            .unwrap_or_default()
    }

    fn to_html_attr(&self, context: &mut HtmlEncodeContext) -> String {
        self.as_ref()
            .map(|value| value.to_html_attr(context))
            .unwrap_or_default()
    }
}
