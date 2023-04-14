use crate::prelude::*;

impl ToHtml for bool {
    fn to_html(&self) -> String {
        elem(&name("Boolean"), &[], &[self.to_string()])
    }
}
