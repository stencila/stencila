use node_html::{attr_maybe, elem, ToHtml};

use crate::Paragraph;

impl ToHtml for Paragraph {
    fn to_html(&self) -> String {
        elem(
            "p",
            &[attr_maybe("id", &self.id)],
            &[self.content.to_html()],
        )
    }
}
