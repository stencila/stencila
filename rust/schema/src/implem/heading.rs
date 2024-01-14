use codec_losses::lost_options;

use crate::{prelude::*, Heading};

impl Heading {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::elem;

        let (content, mut losses) = self.content.to_jats();

        // The `level` attribute is not part of the JATS standard but allows
        // lossless roundtrip conversion of Stencila documents to/from JATS.
        let attrs = [("level", &self.level)];

        if self.id.is_some() {
            losses.add("Heading.id")
        }

        (elem("title", attrs, content), losses)
    }

    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        use codec_html_trait::encode::{attr, elem};
        elem(
            &["h", &self.level.max(1).min(6).to_string()].concat(),
            &[attr("id", &self.id.to_html_attr(context))],
            &[self.content.to_html(context)],
        )
    }
}

impl MarkdownCodec for Heading {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .push_prop_str("level", &"#".repeat(self.level.max(1).min(6) as usize))
            .push_str(" ")
            .push_prop_fn("content", |context| self.content.to_markdown(context))
            .push_str("\n")
            .exit_node()
            .push_str("\n");
    }
}
