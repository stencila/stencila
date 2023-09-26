use crate::{prelude::*, For};

impl For {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut md = ["::: for ", &self.symbol, " in ", &self.code.0].concat();
        let mut losses = Losses::none();

        if !self.programming_language.is_empty() && self.guess_language != Some(true) {
            md.push('{');
            md.push_str(&self.programming_language);
            md.push('}');
        }

        md.push_str("\n\n");

        let (content, mut content_losses) = self.content.to_markdown();
        md.push_str(&content);
        losses.add_all(&mut content_losses);

        if let Some(otherwise) = &self.otherwise {
            md.push_str("::: else\n\n");

            let (otherwise, mut otherwise_losses) = otherwise.to_markdown();
            md.push_str(&otherwise);
            losses.add_all(&mut otherwise_losses);
        }

        md.push_str(":::\n\n");

        // TODO: losses for executable properties

        (md, losses)
    }
}
