use crate::prelude::*;

impl ToHtml for u64 {
    fn to_html(&self) -> String {
        elem(&name("UnsignedInteger"), &[], &[self.to_string()])
    }
}
