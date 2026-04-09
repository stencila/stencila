use crate::{Comment, prelude::*};

impl MarkdownCodec for Comment {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if context.render || !matches!(context.format, Format::Smd) {
            return;
        }

        // Derive the comment ID: use explicit id, or extract from startLocation (#comment-{id}-start)
        let id = self
            .id
            .as_deref()
            .or_else(|| {
                self.options
                    .start_location
                    .as_deref()
                    .and_then(|loc| loc.strip_prefix("#comment-"))
                    .and_then(|loc| loc.strip_suffix("-start"))
            })
            .unwrap_or("unknown");

        // Create the definition block: [>>id]: content
        let mut def_context = MarkdownEncodeContext::default();
        def_context
            .enter_node(self.node_type(), self.node_id())
            .push_str(&format!("[>>{id}]: "))
            .push_line_prefix("    ");
        self.content.to_markdown(&mut def_context);
        def_context.trim_end().push_str("\n\n").exit_node();

        context.comments.push(def_context);

        // Recursively encode reply comments
        if let Some(replies) = &self.options.comments {
            for (i, reply) in replies.iter().enumerate() {
                let reply_id = reply
                    .id
                    .clone()
                    .unwrap_or_else(|| format!("{id}.{}", i + 1));

                // Create a temporary comment with the derived ID for encoding
                let mut reply_with_id = reply.clone();
                reply_with_id.id = Some(reply_id);
                reply_with_id.to_markdown(context);
            }
        }
    }
}
