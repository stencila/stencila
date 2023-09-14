use crate::HtmlCodec;

impl<T> HtmlCodec for Box<T>
where
    T: HtmlCodec,
{
    fn to_html(&self) -> String {
        self.as_ref().to_html()
    }

    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        self.as_ref().to_html_parts()
    }

    fn to_html_attr(&self) -> String {
        self.as_ref().to_html_attr()
    }
}
