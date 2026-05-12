use stencila_codec_dom_trait::{DomCodec, DomEncodeContext};
use stencila_codec_info::NodeProperty;
use stencila_codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext};
use stencila_node_id::NodeId;
use stencila_node_type::NodeType;

use crate::{Author, Block, DateTime};

/// Create curly-braced Markdown attrs for author and date metadata
pub(crate) fn author_date_to_markdown(
    authors: &Option<Vec<Author>>,
    date_published: &Option<DateTime>,
) -> Option<String> {
    let mut attrs = Vec::new();

    if let Some(authors) = authors
        && !authors.is_empty()
    {
        let by = authors
            .iter()
            .map(|author| author.name())
            .collect::<Vec<_>>()
            .join("; ");

        attrs.push(markdown_attr("by", &by));
    }

    if let Some(at) = date_published {
        attrs.push(markdown_attr("at", &at.value));
    }

    (!attrs.is_empty()).then(|| attrs.join(", "))
}

/// Create curly-braced Markdown attrs for suggestion metadata.
pub(crate) fn suggestion_attrs_to_markdown(
    id: &Option<String>,
    authors: &Option<Vec<Author>>,
    date_published: &Option<DateTime>,
) -> Option<String> {
    let mut attrs = Vec::new();

    if let Some(id) = id {
        attrs.push(markdown_attr("id", id));
    }

    if let Some(author_date) = author_date_to_markdown(authors, date_published) {
        attrs.push(author_date);
    }

    (!attrs.is_empty()).then(|| attrs.join(", "))
}

fn markdown_attr(name: &str, value: &str) -> String {
    format!(
        r#"{name}="{}""#,
        value.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

/// Encode the `caption` of a `Figure`, `Table` or `CodeChunk` to DOM HTML
///
/// Injects a `<span>` for the label into the first paragraph.
pub(super) fn caption_to_dom(
    context: &mut DomEncodeContext,
    class: &str,
    kind: &str,
    label: &Option<String>,
    caption: &Option<Vec<Block>>,
) {
    let Some(caption) = caption else {
        // No caption so just render label type and label. Render within a
        // pseudo-<stencila-paragraph> element for styling consistency with when
        // there is a caption (the usual case) It is best to use enter & exit
        // node here so that custom elements have expected attributes e.g. depth
        // & ancestors.
        context
            .enter_node(NodeType::Paragraph, NodeId::random(*b"pgh"))
            .enter_elem_attrs("p", [("slot", "content")])
            .enter_elem_attrs("span", [("class", class)]);

        if let Some(label) = label {
            if !label.to_lowercase().contains(&kind.to_lowercase()) {
                context.push_text(kind).push_text(" ").push_text(label);
            } else {
                context.push_text(label);
            }
        } else {
            context.push_text(kind);
        }

        context.exit_elem().exit_elem().exit_node();

        return;
    };

    for (index, block) in caption.iter().enumerate() {
        if let (0, Block::Paragraph(paragraph)) = (index, block) {
            context.enter_node(paragraph.node_type(), paragraph.node_id());

            if let Some(authors) = &paragraph.authors {
                context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
            }

            if let Some(provenance) = &paragraph.provenance {
                context.push_slot_fn("div", "provenance", |context| provenance.to_dom(context));
            }

            context
                .enter_elem_attrs("p", [("slot", "content")])
                .enter_elem_attrs("span", [("class", class)]);

            if let Some(label) = label {
                if !label.to_lowercase().contains(&kind.to_lowercase()) {
                    context.push_text(kind).push_text(" ").push_text(label);
                } else {
                    context.push_text(label);
                }
            } else {
                context.push_text(kind);
            }

            context.exit_elem().push_text(": ");

            paragraph.content.to_dom(context);
            context.exit_elem().exit_node();
        } else {
            block.to_dom(context);
        }
    }
}

/// Encode the label and caption prefix of a `Figure`, `Table` or `CodeChunk` to Markdown.
pub(super) fn caption_to_markdown(
    context: &mut MarkdownEncodeContext,
    kind: &str,
    label: &Option<String>,
    caption: &Option<Vec<Block>>,
) {
    context.push_str(kind);

    if let Some(label) = label {
        context.push_str(" ");
        context.push_prop_str(NodeProperty::Label, label);
    }

    if let Some(caption) = caption {
        context.push_str(": ");
        context.push_prop_fn(NodeProperty::Caption, |context| {
            caption.to_markdown(context);
            context.trim_end();
        });
    }
}

/// Ensure Markdown block content is followed by a blank line.
pub(super) fn ensure_markdown_blankline(context: &mut MarkdownEncodeContext) {
    if context.content.is_empty() || context.content.ends_with("\n\n") {
        return;
    }

    if context.content.ends_with('\n') {
        context.newline();
    } else {
        context.push_str("\n\n");
    }
}
