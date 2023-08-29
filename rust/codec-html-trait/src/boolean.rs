use crate::prelude::*;

impl HtmlCodec for bool {
    fn to_html(&self) -> String {
        elem(&name("Boolean"), &[], &[self.to_string()])
    }
}
