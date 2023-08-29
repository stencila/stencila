use crate::prelude::*;

impl<T> ToHtml for Box<T>
where
    T: ToHtml,
{
    fn to_html(&self) -> String {
        self.as_ref().to_html()
    }
}
