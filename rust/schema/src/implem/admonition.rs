use codec_losses::lost_options;

use crate::{prelude::*, Admonition};

impl Admonition {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id);

        let (content, content_losses) = self.content.to_markdown(context);
        losses.merge(content_losses);

        let md = [
            "> [!",
            &self.admonition_type.to_string().to_lowercase(),
            "]",
            match &self.is_folded {
                Some(true) => "+",
                Some(false) => "-",
                None => "",
            },
            &match &self.title {
                Some(title) => {
                    let (title, title_losses) = title.to_markdown(context);
                    losses.merge(title_losses);
                    [" ", &title].concat()
                }
                None => String::new(),
            },
            "\n",
            &content
                .trim()
                .lines()
                .map(|line| ["> ", line].concat())
                .join("\n"),
            "\n\n",
        ]
        .concat();

        (md, losses)
    }
}
