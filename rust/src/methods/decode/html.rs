use super::md;
use crate::traits::ToVecInlineContent;
use defaults::Defaults;
use eyre::Result;
use kuchiki::{traits::*, ElementData, NodeRef};
use markup5ever::local_name;
use std::cmp::max;
use stencila_schema::{
    Article, AudioObjectSimple, BlockContent, CodeBlock, CodeFragment, Delete, Emphasis, Heading,
    ImageObjectSimple, InlineContent, Link, List, ListItem, ListItemContent, ListOrder, Node,
    NontextualAnnotation, Paragraph, Strong, Subscript, Superscript, ThematicBreak,
    VideoObjectSimple,
};

// Public API structs and functions...

/// Decoding options for the `decode` and `decode_fragment` functions
#[derive(Defaults)]
pub struct Options {
    /// Attempt to decode text content as Markdown
    #[def = "false"]
    pub decode_markdown: bool,
}

/// Decode a HTML document to a `Node`
///
/// Intended for decoding an entire document into an `Article`.
pub fn decode(html: &str, options: Options) -> Result<Node> {
    let content = decode_fragment(html, options);

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
///
/// If any block content is present in the fragment then that will be returned.
/// Otherwise, if the fragment only consists of inline content a vector with
/// a single paragraph containing that content will be returned.
pub fn decode_fragment(html: &str, options: Options) -> Vec<BlockContent> {
    if html.is_empty() {
        return vec![];
    }

    let context = Context { options };
    let document = kuchiki::parse_html().one(html);

    let content = decode_blocks(&document, &context);
    if !content.is_empty() {
        return content;
    }

    let content = decode_inlines(&document, &context);
    if !content.is_empty() {
        return vec![BlockContent::Paragraph(Paragraph {
            content,
            ..Default::default()
        })];
    }

    vec![]
}

// Private implementation structs and functions...

/// Decoding context
struct Context {
    options: Options,
}

/// Decode the children of a HTML node into a vector of `BlockContent`
fn decode_blocks(node: &NodeRef, context: &Context) -> Vec<BlockContent> {
    node.children()
        .flat_map(|child| decode_block(&child, context))
        .collect()
}

/// Decode a HTML node into a zero or more `BlockContent` nodes
///
/// Will ignore elements that are dealt with by `decode_inline`
fn decode_block(node: &NodeRef, context: &Context) -> Vec<BlockContent> {
    if let Some(_document) = node.as_document() {
        // Recurse into document
        decode_blocks(node, context)
    } else if let Some(element) = node.as_element() {
        // Decode a HTML element
        //
        // The following are ordered alphabetically by the output node type
        // with placeholder comments for types not implemented yet
        match element.name.local {
            // TODO: Claim
            local_name!("pre") => {
                // Follows the recommendation of [HTML5 spec](https://html.spec.whatwg.org/#the-code-element)
                // to "use the class attribute, e.g. by adding a class prefixed with "language-" to the element."
                let programming_language = if let Ok(code) = node.select_first("code") {
                    code.attributes
                        .borrow()
                        .get(local_name!("class"))
                        .map(|value| Box::new(value.replace("language-", "")))
                } else {
                    None
                };
                let text = collect_text(node);

                vec![BlockContent::CodeBlock(CodeBlock {
                    programming_language,
                    text,
                    ..Default::default()
                })]
            }
            // TODO: CodeChunk
            // TODO: Collection
            // TODO: Figure
            local_name!("h1")
            | local_name!("h2")
            | local_name!("h3")
            | local_name!("h4")
            | local_name!("h5")
            | local_name!("h6") => {
                let id = get_id(element);
                let depth = element.name.local.strip_prefix("h").map(|depth| {
                    // See the `Heading.to_html` for the rationale for
                    // subtracting one from the depth
                    let depth = depth.parse().unwrap_or(1);
                    max(1, depth - 1)
                });
                let content = decode_inlines(node, context);
                vec![BlockContent::Heading(Heading {
                    id,
                    depth,
                    content,
                    ..Default::default()
                })]
            }
            local_name!("ul") | local_name!("ol") => {
                let order = Some(match element.name.local {
                    local_name!("ol") => ListOrder::Ascending,
                    _ => ListOrder::Unordered,
                });
                let items = decode_list_items(node, context);

                vec![BlockContent::List(List {
                    order,
                    items,
                    ..Default::default()
                })]
            }
            // TODO: MathBlock
            local_name!("p") => {
                vec![BlockContent::Paragraph(Paragraph {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            // TODO: QuoteBlock
            // TODO: Table
            local_name!("hr") => {
                vec![BlockContent::ThematicBreak(ThematicBreak::default())]
            }

            // Recurse into HTML block elems thereby ignoring them but
            // not their children
            local_name!("html")
            | local_name!("body")
            | local_name!("article")
            | local_name!("main")
            | local_name!("aside")
            | local_name!("div")
            | local_name!("section") => decode_blocks(node, context),

            // All other elements (e.g. inlines) are skipped
            _ => vec![],
        }
    } else if let Some(text) = node.as_text() {
        // Decode HTML non-whitespace text by optionally parsing it as a
        // Markdown fragment
        if !text.borrow().trim().is_empty() {
            if context.options.decode_markdown {
                md::decode_fragment(&text.borrow())
            } else {
                vec![BlockContent::Paragraph(Paragraph {
                    content: vec![InlineContent::String(text.borrow().clone())],
                    ..Default::default()
                })]
            }
        } else {
            vec![]
        }
    } else {
        // Skip everything else
        vec![]
    }
}

/// Decode the children of a HTML node into a vector of `InlineContent`
fn decode_inlines(node: &NodeRef, context: &Context) -> Vec<InlineContent> {
    node.children()
        .flat_map(|child| decode_inline(&child, context))
        .collect()
}

/// Decode a HTML node into a zero or more `InlineContent` nodes.
///
/// This function should handle most of the HTML "Phrasing content"
/// [elements](https://developer.mozilla.org/en-US/docs/Web/Guide/HTML/Content_categories#phrasing_content)
fn decode_inline(node: &NodeRef, context: &Context) -> Vec<InlineContent> {
    if let Some(element) = node.as_element() {
        // Decode a HTML element
        //
        // The following are ordered alphabetically by the output node type
        // with placeholder comments for types not implemented yet
        match element.name.local {
            local_name!("audio") => {
                let attrs = element.attributes.borrow();
                let content_url = attrs.get(local_name!("src")).unwrap_or("").to_string();

                vec![InlineContent::AudioObject(AudioObjectSimple {
                    content_url,
                    ..Default::default()
                })]
            }
            // TODO: Cite
            // TODO: CiteGroup
            // TODO: CodeExpression
            local_name!("code") => {
                // See note for `CodeBlock` on choice of attribute for decodiong `programming_language`
                let programming_language = element
                    .attributes
                    .borrow()
                    .get(local_name!("class"))
                    .map(|value| Box::new(value.replace("language-", "")));
                let text = collect_text(node);
                vec![InlineContent::CodeFragment(CodeFragment {
                    programming_language,
                    text,
                    ..Default::default()
                })]
            }
            local_name!("del") => {
                vec![InlineContent::Delete(Delete {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            local_name!("em") => {
                vec![InlineContent::Emphasis(Emphasis {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            local_name!("img") => {
                let attrs = element.attributes.borrow();
                let content_url = attrs.get(local_name!("src")).unwrap_or("").to_string();
                let caption = attrs
                    .get(local_name!("title"))
                    .map(|value| Box::new(value.to_string()));

                vec![InlineContent::ImageObject(ImageObjectSimple {
                    content_url,
                    caption,
                    ..Default::default()
                })]
            }
            local_name!("a") => {
                let attrs = element.attributes.borrow();
                let target = attrs.get(local_name!("href")).unwrap_or("").to_string();
                let title = attrs
                    .get(local_name!("title"))
                    .map(|value| Box::new(value.to_string()));
                let content = decode_inlines(node, context);

                vec![InlineContent::Link(Link {
                    target,
                    title,
                    content,
                    ..Default::default()
                })]
            }
            // TODO: MathFragment
            local_name!("u") => {
                vec![InlineContent::NontextualAnnotation(NontextualAnnotation {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            // TODO: Note
            // TODO: Quote
            local_name!("strong") => {
                vec![InlineContent::Strong(Strong {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            local_name!("sub") => {
                vec![InlineContent::Subscript(Subscript {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            local_name!("sup") => {
                vec![InlineContent::Superscript(Superscript {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            local_name!("video") => {
                let (content_url, media_type) = if let Ok(source) = node.select_first("source") {
                    let attrs = source.attributes.borrow();
                    let content_url = attrs.get(local_name!("src")).unwrap_or("").to_string();
                    let media_type = attrs
                        .get(local_name!("type"))
                        .map(|value| Box::new(value.to_string()));
                    (content_url, media_type)
                } else {
                    let attrs = element.attributes.borrow();
                    let content_url = attrs.get(local_name!("src")).unwrap_or("").to_string();
                    (content_url, None)
                };

                vec![InlineContent::VideoObject(VideoObjectSimple {
                    content_url,
                    media_type,
                    ..Default::default()
                })]
            }

            // Recurse into all other elements thereby ignoring them but
            // not their inline children
            _ => decode_inlines(node, context),
        }
    } else if let Some(text) = node.as_text() {
        // Decode HTML text by optionally parsing it as a Markdown fragment
        // and unwrapping from `Vec<BlockContent>` to `Vec<InlineContent>`.
        if !text.borrow().is_empty() {
            if context.options.decode_markdown {
                md::decode_fragment(&text.borrow()).to_vec_inline_content()
            } else {
                vec![InlineContent::String(text.borrow().clone())]
            }
        } else {
            vec![]
        }
    } else {
        // Skip everything else
        vec![]
    }
}

/// Decode list items from a `<ul>` or `<ol>`.
///
/// Only `<li>` children (and their descendants) are returned.
fn decode_list_items(node: &NodeRef, context: &Context) -> Vec<ListItem> {
    node.children()
        .filter_map(|child| {
            if let Some(element) = child.as_element() {
                if matches!(element.name.local, local_name!("li")) {
                    let blocks = decode_blocks(&child, context);
                    let content = if !blocks.is_empty() {
                        Some(ListItemContent::VecBlockContent(blocks))
                    } else {
                        let inlines = decode_inlines(&child, context);
                        if !inlines.is_empty() {
                            Some(ListItemContent::VecInlineContent(inlines))
                        } else {
                            None
                        }
                    };

                    return Some(ListItem {
                        content,
                        ..Default::default()
                    });
                }
            }
            None
        })
        .collect()
}

/// Get the `id` attribute of an element (if any)
fn get_id(element: &ElementData) -> Option<Box<String>> {
    element
        .attributes
        .borrow()
        .get(local_name!("id"))
        .map(|id| Box::new(id.to_string()))
}

/// Accumulate all the text within a node, including text within descendant elements.
fn collect_text(node: &NodeRef) -> String {
    if let Some(text) = node.as_text() {
        text.borrow().to_string()
    } else {
        node.children().fold(String::new(), |acc, child| {
            [acc, collect_text(&child)].concat()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn html_articles() {
        snapshot_content("articles/*.html", |content| {
            assert_json_snapshot!(
                decode(&content, Options::default()).expect("Unable to decode HTML")
            );
        });
    }

    #[test]
    fn html_fragments() {
        snapshot_content("fragments/html/*.html", |content| {
            assert_json_snapshot!(decode_fragment(&content, Options::default()));
        });
    }
}
