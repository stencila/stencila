use crate::{Comment, implem::utils::author_date_to_markdown, prelude::*};

impl MarkdownCodec for Comment {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        if matches!(
            context.mode,
            MarkdownEncodeMode::Clean | MarkdownEncodeMode::Render
        ) || !matches!(context.format, Format::Smd)
        {
            return;
        }

        encode_comment(self, context, None, None);
    }
}

fn encode_comment(
    comment: &Comment,
    context: &mut MarkdownEncodeContext,
    parent_id: Option<&str>,
    reply_index: Option<usize>,
) {
    // Nested replies use canonical hierarchical IDs derived from their parent
    // comment and position so that round-tripped comment threads are encoded
    // consistently as `parent.index`, `parent.index.index`, etc.
    let id = match (parent_id, reply_index) {
        (Some(parent_id), Some(index)) => format!("{parent_id}.{index}"),
        _ => comment
            .id
            .clone()
            .or_else(|| {
                comment
                    .options
                    .start_location
                    .as_deref()
                    .and_then(|loc| loc.strip_prefix("#comment-"))
                    .and_then(|loc| loc.strip_suffix("-start"))
                    .map(ToString::to_string)
            })
            .unwrap_or_else(|| "unknown".to_string()),
    };

    // Create the definition block: [>>id]: content
    let mut def_context = MarkdownEncodeContext::default();
    def_context
        .enter_node(comment.node_type(), comment.node_id())
        .push_str(&format!("[>>{id}]"));

    if let Some(attrs) = author_date_to_markdown(&comment.authors, &comment.date_published) {
        def_context
            .push_str("{")
            .push_str(attrs.trim_start())
            .push_str("}");
    }

    def_context.push_str(": ").push_line_prefix("    ");
    comment.content.to_markdown(&mut def_context);
    def_context.trim_end().push_str("\n\n").exit_node();

    context.comments.push(def_context);

    // Recursively encode reply comments
    if let Some(replies) = &comment.options.comments {
        for (index, reply) in replies.iter().enumerate() {
            encode_comment(reply, context, Some(&id), Some(index + 1));
        }
    }
}
