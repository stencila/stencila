use crate::{prelude::*, Section};

impl Section {
    pub fn to_markdown_special(&self, context: &MarkdownEncodeContext) -> (String, Losses) {
        let (md, losses) = self.content.to_markdown(&MarkdownEncodeContext {
            depth: context.depth + 1,
        });

        let fence = ":".repeat(3 + context.depth * 2);

        let md = [&fence, " section\n\n", &md, &fence, "\n\n"].concat();

        (md, losses)
    }
}
