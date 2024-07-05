use codec_info::{lost_exec_options, lost_options};

use crate::{prelude::*, IncludeBlock};

impl MarkdownCodec for IncludeBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render {
            // Encode content only
            if let Some(content) = &self.content {
                content.to_markdown(context);
            }
            return;
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id))
            .merge_losses(lost_exec_options!(self));

        if matches!(context.format, Format::Myst) {
            context.myst_directive(
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
            );
        } else {
            context
                .push_semis()
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
        }

        context.newline().exit_node().newline();
    }
}
