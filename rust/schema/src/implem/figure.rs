use crate::{prelude::*, Figure};

impl MarkdownCodec for Figure {
    fn to_markdown(&self, _context: &mut MarkdownEncodeContext) {
        /*
        TODO

        let mut md = String::new();
        let mut losses = lost_options!(self, id);

        let fence = ":".repeat(3 + context.depth * 2);

        context.down();

        md += &fence;
        md += " figure";
        if let Some(label) = &self.label {
            md += " ";
            md += label;
        }
        md += "\n\n";

        if let Some(caption) = &self.caption {
            let (caption_md, caption_losses) = caption.to_markdown(context);
            md += &caption_md;
            losses.merge(caption_losses)
        }

        let (content_md, content_losses) = self.content.to_markdown(context);
        md += &content_md;
        losses.merge(content_losses);

        md += &fence;
        md += "\n\n";

        context.up();
        */
    }
}
