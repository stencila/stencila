use codec_html_traits::to_html::{elem, name, ToHtml};

use crate::Null;

impl ToHtml for Null {
    fn to_html(&self) -> String {
        elem(&name("Null"), &[], &["null".to_string()])
    }
}
