use codec_info::lost_options;

use crate::{prelude::*, Claim};

impl MarkdownCodec for Claim {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

        if matches!(context.format, Format::Myst) {
            context.myst_directive(
                ':',
                &["prf:", &self.claim_type.to_string().to_lowercase()].concat(),
                |context| {
                    if let Some(title) = &self.options.title {
                        context
                            .push_str(" ")
                            .push_prop_fn(NodeProperty::Title, |context| {
                                title.to_markdown(context)
                            });
                    }
                },
                |context| {
                    if let Some(label) = &self.label {
                        context.myst_directive_option(NodeProperty::Label, None, label);
                    }
                },
                |context| {
                    context.push_prop_fn(NodeProperty::Content, |context| {
                        self.content.to_markdown(context)
                    });
                },
            );
        } else {
            context.push_semis().push_str(" ").push_prop_str(
                NodeProperty::ClaimType,
                &self.claim_type.to_string().to_lowercase(),
            );

            if let Some(label) = &self.label {
                context
                    .push_str(" ")
                    .push_prop_str(NodeProperty::Label, label);
            }

            context
                .push_str("\n\n")
                .increase_depth()
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .decrease_depth()
                .push_semis();
        }

        context.newline().exit_node().newline();
    }
}
