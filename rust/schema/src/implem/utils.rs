use codec_dom_trait::{DomCodec, DomEncodeContext};

use crate::Block;

/// Encode the `caption` of a `Figure`, `Table` of `CodeChunk` to DOM HTML
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
        // No caption so just render label type and label. Render within a pseudo-<stencila-paragraph>
        // element for styling consistency with when there is a caption (the usual case)

        context
            .enter_elem("stencila-paragraph")
            .enter_elem_attrs("p", [("slot", "content")])
            .enter_elem_attrs("span", [("class", class)])
            .push_text(kind);

        if let Some(label) = label {
            context.push_text(" ").push_text(label);
        }

        context.exit_elem().exit_elem().exit_elem();

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
                .enter_elem_attrs("span", [("class", class)])
                .push_text(kind);
            if let Some(label) = label {
                context.push_text(" ").push_text(label);
            }
            context.exit_elem().push_text(" ");

            paragraph.content.to_dom(context);
            context.exit_elem().exit_node();
        } else {
            block.to_dom(context);
        }
    }
}
