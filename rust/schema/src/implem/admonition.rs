use codec_info::lost_options;

use crate::{prelude::*, Admonition, AdmonitionType};

impl MarkdownCodec for Admonition {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id));

        if matches!(context.format, Format::Myst) {
            let name = match &self.admonition_type {
                AdmonitionType::Failure => "error".to_string(),
                other => other.to_string().to_lowercase(),
            };

            context
                .myst_directive(
                    ':',
                    &name,
                    |context| {
                        if let Some(title) = &self.title {
                            context
                                .push_str(" ")
                                .push_prop_fn(NodeProperty::Title, |context| {
                                    title.to_markdown(context)
                                });
                        }
                    },
                    |context| {
                        if self.is_folded.is_some() {
                            context.myst_directive_option(
                                NodeProperty::IsFolded,
                                Some("class"),
                                "dropdown",
                            );
                        }
                    },
                    |context| {
                        context.push_prop_fn(NodeProperty::Content, |context| {
                            self.content.to_markdown(context)
                        });
                    },
                )
                .newline();
        } else {
            context
                .push_str("> [!")
                .push_prop_str(
                    NodeProperty::AdmonitionType,
                    &self.admonition_type.to_string().to_lowercase(),
                )
                .push_str("]");

            if let Some(is_folded) = self.is_folded {
                context.push_prop_str(NodeProperty::IsFolded, if is_folded { "+" } else { "-" });
            }

            if let Some(title) = &self.title {
                context
                    .push_str(" ")
                    .push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context));
            }

            context
                .newline()
                .push_line_prefix("> ")
                .prefix_empty_lines(true)
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .pop_line_prefix();

            if context.content.ends_with("> \n") {
                context.content.pop();
                context.content.pop();
                context.content.pop();
                context.content.push('\n');
            }
        }

        context.exit_node();
    }
}
