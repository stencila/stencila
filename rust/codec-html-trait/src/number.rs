use crate::prelude::*;

impl HtmlCodec for f64 {
    fn to_html(&self) -> String {
        elem(&name("Number"), &[], &[self.to_string()])
    }
}
