use crate::TextValue;

use super::prelude::*;

impl HtmlCodec for TextValue {
    fn to_html(&self) -> String {
        text(&self.0)
    }
}
