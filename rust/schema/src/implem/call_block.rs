use stencila_codec_info::{lost_exec_options, lost_options};

use crate::{CallBlock, prelude::*};

impl LatexCodec for CallBlock {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        const ENVIRON: &str = "call";

        context
            .enter_node(self.node_type(), self.node_id())
            .add_loss("CallBlock.arguments")
            .merge_losses(lost_options!(self, id, media_type, select, execution_mode))
            .merge_losses(lost_exec_options!(self))
            .environ_begin(ENVIRON)
            .char('{')
            .property_str(NodeProperty::Source, &self.source)
            .char('}')
            .newline()
            .environ_end(ENVIRON)
            .exit_node();
    }
}

impl MarkdownCodec for CallBlock {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, content))
            .merge_losses(lost_exec_options!(self))
            .push_colons()
            .push_str(" call ")
            .push_prop_str(NodeProperty::Source, &self.source);

        if !self.arguments.is_empty() {
            context.push_str(" (");

            for (index, arg) in self.arguments.iter().enumerate() {
                if index != 0 {
                    context.push_str(", ");
                }
                arg.to_markdown(context);
            }

            context.push_str(")");
        }

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

        context.newline();

        if let (Format::Llmd, Some(content)) = (&context.format, &self.content) {
            context
                .push_str("\n=>\n\n")
                .push_prop_fn(NodeProperty::Content, |context| {
                    content.to_markdown(context)
                });
        }

        context.exit_node().newline();
    }
}
