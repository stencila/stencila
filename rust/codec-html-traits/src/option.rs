use crate::prelude::*;

impl<T> ToHtml for Option<T>
where
    T: ToHtml,
{
    fn to_html(&self) -> String {
        self.as_ref()
            .map(|value| value.to_html())
            .unwrap_or_default()
    }
}
