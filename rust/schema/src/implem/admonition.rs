use stencila_codec_info::lost_options;

use crate::{Admonition, AdmonitionType, prelude::*};

impl DomCodec for Admonition {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        let admonition_type = self.admonition_type.to_string();

        context
            .enter_node(self.node_type(), self.node_id())
            .push_attr("admonition-type", &admonition_type);

        if let Some(is_folded) = self.is_folded {
            context.push_attr("is-folded", &is_folded.to_string());
        }

        context.enter_elem("details");

        if !matches!(self.is_folded, Some(true)) {
            context.push_attr_boolean("open");
        }

        context
            .enter_elem("summary")
            .push_slot_fn("span", "title", |context| match &self.title {
                Some(title) => {
                    title.to_dom(context);
                }
                None => {
                    context.push_html(&admonition_type);
                }
            })
            .exit_elem()
            .push_slot_fn("div", "content", |context| self.content.to_dom(context))
            .exit_elem()
            .exit_node();
    }
}

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
                .exit_node()
                .newline();
        } else if matches!(context.format, Format::Qmd) {
            let name = match &self.admonition_type {
                AdmonitionType::Success => "note".to_string(),
                AdmonitionType::Danger | AdmonitionType::Error | AdmonitionType::Failure => {
                    "important".to_string()
                }
                other => other.to_string().to_lowercase(),
            };

            context
                .push_str(":::{.callout-")
                .push_prop_str(NodeProperty::AdmonitionType, &name);

            if let Some(is_folded) = self.is_folded {
                context
                    .push_str(" collapse=\"")
                    .push_prop_str(NodeProperty::IsFolded, &is_folded.to_string())
                    .push_str("\"");
            }

            context.push_str("}\n");

            if let Some(title) = &self.title {
                context
                    .push_str("## ")
                    .push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context))
                    .push_str("\n\n");
            }

            context.push_prop_fn(NodeProperty::Content, |context| {
                self.content.to_markdown(context)
            });

            if context.content.ends_with("\n") {
                context.content.pop();
            }

            context.push_str(":::\n").exit_node().newline();
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

            context.exit_node();
        }
    }
}
