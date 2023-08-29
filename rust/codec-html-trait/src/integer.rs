use crate::prelude::*;

impl HtmlCodec for i64 {
    fn to_html(&self) -> String {
        elem(&name("Integer"), &[], &[self.to_string()])
    }
}
