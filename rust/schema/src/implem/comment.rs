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

        encode_comment(self, context, None);
    }
}

fn encode_comment(comment: &Comment, context: &mut MarkdownEncodeContext, parent_id: Option<&str>) {
    let parent_id = comment.options.parent_item.as_deref().or(parent_id);

    let id = comment
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
        .unwrap_or_else(|| "unknown".to_string());

    fn format_attr(name: &str, value: &str) -> String {
        format!(
            r#"{name}="{}""#,
            value.replace('\\', "\\\\").replace('"', "\\\"")
        )
    }

    let mut attrs = Vec::new();

    if let Some(author_date_attrs) =
        author_date_to_markdown(&comment.authors, &comment.date_published)
    {
        attrs.push(author_date_attrs);
    }

    if let Some(parent_id) = parent_id {
        attrs.push(format_attr("parent", parent_id));
    }

    // Create the definition block: [>>id]: content
    let mut def_context = MarkdownEncodeContext::default();
    def_context
        .enter_node(comment.node_type(), comment.node_id())
        .push_str(&format!("[>>{id}]"));

    if !attrs.is_empty() {
        def_context
            .push_str("{")
            .push_str(&attrs.join(", "))
            .push_str("}");
    }

    def_context.push_str(": ").push_line_prefix("    ");
    comment.content.to_markdown(&mut def_context);
    def_context.trim_end().push_str("\n\n").exit_node();

    context.comments.push(def_context);

    // Recursively encode reply comments
    if let Some(replies) = &comment.options.comments {
        for reply in replies {
            encode_comment(reply, context, Some(&id));
        }
    }
}
