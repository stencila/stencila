use crate::prelude::*;

impl ToHtml for i64 {
    fn to_html(&self) -> String {
        elem(&name("Integer"), &[], &[self.to_string()])
    }
}
