use eyre::Result;
use kuchiki::{traits::*, NodeData, NodeRef};
use markup5ever::local_name;
use stencila_schema::{
    Article, BlockContent, Delete, Emphasis, ImageObjectSimple, InlineContent, List, Node,
    NontextualAnnotation, Paragraph, Strong, Subscript, Superscript,
};

/// Decode a HTML document to a `Node`
///
/// Intended for decoding an entire document into an `Article`.
pub fn decode(html: &str) -> Result<Node> {
    let content = decode_fragment(html);

    let article = Article {
        content: Some(content),
        ..Default::default()
    };

    Ok(Node::Article(article))
}

/// Decode a HTML fragment to a vector of `BlockContent`
///
/// Intended for decoding a fragment of HTML (e.g. some HTML in a
/// Markdown document) and inserting it into a larger document.
pub fn decode_fragment(html: &str) -> Vec<BlockContent> {
    let document = kuchiki::parse_html().one(html);
    decode_blocks(document)
}

/// Decode the children of a HTML node into a vector of `BlockContent`
pub fn decode_blocks(node: NodeRef) -> Vec<BlockContent> {
    node.children().map(decode_block).flatten().collect()
}

/// Decode a HTML node into a zero or more `BlockContent` nodes
pub fn decode_block(node: NodeRef) -> Vec<BlockContent> {
    if let Some(element) = node.as_element() {
        match element.name.local {
            local_name!("p") => {
                return vec![BlockContent::Paragraph(Paragraph {
                    content: decode_inlines(node),
                    ..Default::default()
                })]
            }
            _ => (),
        }
    }
    return decode_blocks(node);
}

/// Decode the children of a HTML node into a vector of `InlineContent`
pub fn decode_inlines(node: NodeRef) -> Vec<InlineContent> {
    node.children().map(decode_inline).flatten().collect()
}

/// Decode a HTML node into a zero or more `InlineContent` nodes
pub fn decode_inline(node: NodeRef) -> Vec<InlineContent> {
    if let Some(element) = node.as_element() {
        match element.name.local {
            // These are ordered alphabetically by the output node type
            // with placeholder comments for types not implemented yet

            // AudioObject
            // Cite
            // CiteGroup
            // CodeExpression
            // CodeFragment
            local_name!("del") => {
                return vec![InlineContent::Delete(Delete {
                    content: decode_inlines(node),
                    ..Default::default()
                })]
            }
            local_name!("em") => {
                return vec![InlineContent::Emphasis(Emphasis {
                    content: decode_inlines(node),
                    ..Default::default()
                })]
            }
            local_name!("img") => {
                let attrs = element.attributes.borrow();
                let content_url = attrs.get(local_name!("src")).unwrap_or("").to_string();
                let caption = attrs
                    .get(local_name!("title"))
                    .map(|value| Box::new(value.to_string()));
                return vec![InlineContent::ImageObject(ImageObjectSimple {
                    content_url,
                    caption,
                    ..Default::default()
                })];
            }
            // Link
            // MathFragment
            local_name!("u") => {
                return vec![InlineContent::NontextualAnnotation(NontextualAnnotation {
                    content: decode_inlines(node),
                    ..Default::default()
                })]
            }
            // Note
            // Quote
            local_name!("strong") => {
                return vec![InlineContent::Strong(Strong {
                    content: decode_inlines(node),
                    ..Default::default()
                })]
            }
            local_name!("sub") => {
                return vec![InlineContent::Subscript(Subscript {
                    content: decode_inlines(node),
                    ..Default::default()
                })]
            }
            local_name!("sup") => {
                return vec![InlineContent::Superscript(Superscript {
                    content: decode_inlines(node),
                    ..Default::default()
                })]
            }
            // VideoObject
            _ => (),
        }
    } else if let Some(text) = node.as_text() {
        return vec![InlineContent::String(text.borrow().clone())];
    }
    return decode_inlines(node);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn fragments() {
        snapshot_content("fragments/html/*.html", |content| {
            assert_json_snapshot!(decode_fragment(&content));
        });
    }
}
