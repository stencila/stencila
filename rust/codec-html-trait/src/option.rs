use crate::prelude::*;

impl<T> HtmlCodec for Option<T>
where
    T: HtmlCodec,
{
    fn to_html(&self) -> String {
        self.as_ref()
            .map(|value| value.to_html())
            .unwrap_or_default()
    }
}
