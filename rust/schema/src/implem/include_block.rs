use codec_info::{lost_exec_options, lost_options};
use node_url::NodePosition;

use crate::{IncludeBlock, prelude::*};

impl LatexCodec for IncludeBlock {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, media_type, select, execution_mode))
            .merge_losses(lost_exec_options!(self));

        if context.render {
            if context.reproducible {
                context
                    .str("\n\n\\centerline{")
                    .link_with(
                        Some(NodePosition::Begin),
                        &format!(r"\verb|[Begin {}]|", self.source),
                    )
                    .str("}\n\n");
            }

            context.property_fn(NodeProperty::Content, |context| {
                self.content.to_latex(context)
            });

            if context.reproducible {
                context
                    .str("\n\n\\centerline{")
                    .link_with(
                        Some(NodePosition::End),
                        &format!(r"\verb|[End {}]|", self.source),
                    )
                    .str("}\n\n");
            }
        } else {
            context
                .str("\\input{")
                .property_str(NodeProperty::Source, self.source.trim_end_matches(".tex"))
                .char('}')
                .newline()
                .newline();
        }

        context.exit_node();
    }
}

impl MarkdownCodec for IncludeBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self));

        if matches!(context.format, Format::Llmd) || context.render {
            context
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .exit_node();

            return;
        }

        if matches!(context.format, Format::Myst) {
            // For MyST, encode as an include directive
            context
                .myst_directive(
                    '`',
                    "include",
                    |context| {
                        context
                            .push_str(" ")
                            .push_prop_str(NodeProperty::Source, &self.source);
                    },
                    |context| {
                        if let Some(mode) = self.execution_mode.as_ref() {
                            context.myst_directive_option(
                                NodeProperty::ExecutionMode,
                                Some("mode"),
                                &mode.to_string().to_lowercase(),
                            );
                        }

                        if let Some(format) = self.media_type.as_ref() {
                            context.myst_directive_option(
                                NodeProperty::MediaType,
                                Some("format"),
                                format,
                            );
                        }

                        if let Some(select) = self.select.as_ref() {
                            context.myst_directive_option(NodeProperty::Select, None, select);
                        }
                    },
                    |_| {},
                )
                .exit_node()
                .newline();
        } else if matches!(context.format, Format::Smd) {
            // For SMD, encode as an include block
            context
                .push_colons()
                .push_str(" include ")
                .push_prop_str(NodeProperty::Source, &self.source);

            if self.execution_mode.is_some() || self.media_type.is_some() || self.select.is_some() {
                context.push_str(" {");

                let mut prefix = "";
                if let Some(mode) = &self.execution_mode {
                    context.push_str(" ").push_prop_str(
                        NodeProperty::ExecutionMode,
                        &mode.to_string().to_lowercase(),
                    );
                    prefix = " ";
                }

                if let Some(media_type) = &self.media_type {
                    context
                        .push_str(prefix)
                        .push_str("format=")
                        .push_prop_str(NodeProperty::MediaType, media_type);
                    prefix = " ";
                }

                if let Some(select) = &self.select {
                    context
                        .push_str(prefix)
                        .push_str("select=")
                        .push_prop_str(NodeProperty::Select, select);
                }

                context.push_str("}");
            }

            context.newline().exit_node().newline();
        } else {
            // For Markdown, QMD etc, which do not support include blocks, only encode content (if any)
            if let Some(content) = &self.content {
                if !content.is_empty() {
                    context.push_prop_fn(NodeProperty::Content, |context| {
                        content.to_markdown(context)
                    });
                }
            }
            context.exit_node();
        }
    }
}
