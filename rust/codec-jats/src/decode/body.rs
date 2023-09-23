use roxmltree::Node;

use codec::{
    schema::{
        shortcuts::{em, p, s, strong, sub, sup, text, u},
        Article, Block, Inlines,
    },
    Loss, LossDirection, Losses,
};

/// Decode the `<body>` of an `<article>`
pub(super) fn decode_body(node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let block = match tag {
            "p" => decode_p(&child, losses),
            _ => {
                if child.is_element() {
                    losses.add(Loss::of_type(LossDirection::Decode, tag))
                }
                continue;
            }
        };
        article.content.push(block)
    }
}

/// Decode a `<p>` in the `<body>`
fn decode_p(node: &Node, losses: &mut Losses) -> Block {
    p(decode_inlines(node, losses))
}

/// Decode inline content nodes
fn decode_inlines(node: &Node, losses: &mut Losses) -> Inlines {
    let mut inlines = Inlines::new();
    for child in node.children() {
        let inline = if child.is_text() {
            text(child.text().unwrap_or_default())
        } else {
            let tag = child.tag_name().name();
            match tag {
                "bold" => strong(decode_inlines(&child, losses)),
                "italic" => em(decode_inlines(&child, losses)),
                "strike" => s(decode_inlines(&child, losses)),
                "sub" => sub(decode_inlines(&child, losses)),
                "sup" => sup(decode_inlines(&child, losses)),
                "underline" => u(decode_inlines(&child, losses)),
                _ => {
                    if child.is_element() {
                        losses.add(Loss::of_type(LossDirection::Decode, tag))
                    }
                    continue;
                }
            }
        };
        inlines.push(inline);
    }
    inlines
}
