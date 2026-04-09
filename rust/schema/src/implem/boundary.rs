use crate::{Boundary, prelude::*};

impl MarkdownCodec for Boundary {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render || !matches!(context.format, Format::Smd) {
            return;
        }

        if let Some(id) = &self.id {
            if let Some(cid) = id
                .strip_prefix("comment-")
                .and_then(|s| s.strip_suffix("-start"))
            {
                context.push_str(&format!("{{>>{cid}}}"));
            } else if let Some(cid) = id
                .strip_prefix("comment-")
                .and_then(|s| s.strip_suffix("-end"))
            {
                context.push_str(&format!("{{<<{cid}}}"));
            }
        }
    }
}
