use crate::prelude::*;

impl HtmlCodec for String {
    fn to_html(&self) -> String {
        text(self)
    }
}
