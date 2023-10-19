use crate::{prelude::*, For};

impl For {
    pub fn to_markdown_special(&self, context: &MarkdownEncodeContext) -> (String, Losses) {
        let fence = ":".repeat(3 + context.depth * 2);

        let mut md = [&fence, " for ", &self.symbol, " in ", &self.code].concat();
        let mut losses = Losses::none();

        if !self.programming_language.is_empty() && self.guess_language != Some(true) {
            md.push('{');
            md.push_str(&self.programming_language);
            md.push('}');
        }

        md.push_str("\n\n");

        let context = MarkdownEncodeContext {
            depth: context.depth + 1,
        };

        let (content, content_losses) = self.content.to_markdown(&context);
        md.push_str(&content);
        losses.merge(content_losses);

        if let Some(otherwise) = &self.otherwise {
            md.push_str(&fence);
            md.push_str(" else\n\n");

            let (otherwise, otherwise_losses) = otherwise.to_markdown(&context);
            md.push_str(&otherwise);
            losses.merge(otherwise_losses);
        }

        md.push_str(&fence);
        md.push_str("\n\n");

        // TODO: losses for executable properties

        (md, losses)
    }
}
