use crate::{prelude::*, ModifyBlock, ModifyOperation};

impl ModifyBlock {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut md = String::from("!!\n\n");
        let mut losses = Losses::none();

        let (content_md, content_losses) = self.content.to_markdown(context);
        md += &content_md;
        losses.merge(content_losses);

        md += "!>\n\n";

        let modified =
            ModifyOperation::apply_many(&self.operations, &self.content).unwrap_or_default();
        let (modified_md, modified_losses) = modified.to_markdown(context);
        md += &modified_md;
        losses.merge(modified_losses);

        md += "!!\n\n";

        (md, losses)
    }
}
