use crate::prelude::*;

impl<T> HtmlCodec for Box<T>
where
    T: HtmlCodec,
{
    fn to_html(&self) -> String {
        self.as_ref().to_html()
    }
}
