use roxmltree::Node;

use codec::{
    schema::{shortcuts, Article, Block, Inlines},
    Loss, LossDirection, Losses,
};

/// Decode the `<body>` of an `<article>`
pub(super) fn body(node: &Node, article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let tag = child.tag_name().name();
        let block = match tag {
            "p" => p(&child, losses),
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
fn p(node: &Node, losses: &mut Losses) -> Block {
    shortcuts::p(inlines(node, losses))
}

/// Decode inline content nodes
fn inlines(node: &Node, losses: &mut Losses) -> Inlines {
    let mut inlines = Inlines::new();
    for child in node.children() {
        let inline = if child.is_text() {
            shortcuts::text(child.text().unwrap_or_default())
        } else {
            let tag = child.tag_name().name();
            {
                if child.is_element() {
                    losses.add(Loss::of_type(LossDirection::Decode, tag))
                }
                continue;
            }
        };
        inlines.push(inline);
    }
    inlines
}
