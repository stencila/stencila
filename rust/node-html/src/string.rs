use crate::prelude::*;

impl ToHtml for String {
    fn to_html(&self) -> String {
        text(&self)
    }
}
