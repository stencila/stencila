use crate::Null;

use super::prelude::*;

impl HtmlCodec for Null {
    fn to_html(&self) -> String {
        elem(&name("Null"), &[], &["null".to_string()])
    }
}
