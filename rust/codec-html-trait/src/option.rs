use crate::HtmlCodec;

impl<T> HtmlCodec for Option<T>
where
    T: HtmlCodec,
{
    fn to_html(&self) -> String {
        self.as_ref()
            .map(|value| value.to_html())
            .unwrap_or_default()
    }

    fn to_html_parts(&self) -> (&str, Vec<String>, Vec<String>) {
        self.as_ref()
            .map(|value| value.to_html_parts())
            .unwrap_or_default()
    }

    fn to_html_attr(&self) -> String {
        self.as_ref()
            .map(|value| value.to_html_attr())
            .unwrap_or_default()
    }
}
