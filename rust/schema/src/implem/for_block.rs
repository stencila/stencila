use crate::{prelude::*, ForBlock};

impl ForBlock {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let fence = ":".repeat(3 + context.depth * 2);

        let mut md = [&fence, " for ", &self.symbol, " in ", &self.code].concat();
        let mut losses = Losses::none();

        if let Some(lang) = &self.programming_language {
            if !lang.is_empty() {
                md.push('{');
                md.push_str(lang);
                md.push('}');
            }
        }

        md.push_str("\n\n");

        context.down();

        let (content, content_losses) = self.content.to_markdown(context);
        md.push_str(&content);
        losses.merge(content_losses);

        if let Some(otherwise) = &self.otherwise {
            md.push_str(&fence);
            md.push_str(" else\n\n");

            let (otherwise, otherwise_losses) = otherwise.to_markdown(context);
            md.push_str(&otherwise);
            losses.merge(otherwise_losses);
        }

        context.up();

        md.push_str(&fence);
        md.push_str("\n\n");

        // TODO: losses for executable properties

        (md, losses)
    }
}
