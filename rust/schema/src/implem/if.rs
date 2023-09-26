use crate::{prelude::*, If, IfClause};

impl If {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut md = String::new();
        let mut losses = Losses::none();

        for (index, IfClause { code, content, .. }) in self.clauses.iter().enumerate() {
            md.push_str("::: ");
            let keyword = if index == 0 {
                "if "
            } else if code.is_empty() && index == self.clauses.len() - 1 {
                "else "
            } else {
                "elif "
            };
            md.push_str(keyword);
            md.push_str(&code.0);
            md.push_str("\n\n");

            let (content_md, mut content_losses) = content.to_markdown();
            md.push_str(&content_md);
            losses.add_all(&mut content_losses);
        }

        if !self.clauses.is_empty() {
            md.push_str(":::\n\n");
        }

        // TODO: losses for executable properties

        (md, losses)
    }
}
