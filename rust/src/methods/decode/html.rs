use super::md;
use crate::traits::ToVecInlineContent;
use defaults::Defaults;
use eyre::Result;
use kuchiki::{traits::*, NodeRef};
use markup5ever::local_name;
use stencila_schema::{
    Article, BlockContent, Delete, Emphasis, ImageObjectSimple, InlineContent, Node,
    NontextualAnnotation, Paragraph, Strong, Subscript, Superscript,
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
            // TODO: CodeBlock
            // TODO: CodeChunk
            // TODO: Collection
            // TODO: Figure
            // TODO: Heading
            // TODO: List
            // TODO: MathBlock
            local_name!("p") => {
                vec![BlockContent::Paragraph(Paragraph {
                    content: decode_inlines(node, context),
                    ..Default::default()
                })]
            }
            // TODO: QuoteBlock
            // TODO: Table
            // TODO: ThematicBreak

            // Recurse into HTML block elems thereby ignoring them but
            // not their children
            local_name!("html")
            | local_name!("body")
            | local_name!("main")
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

/// Decode a HTML node into a zero or more `InlineContent` nodes
fn decode_inline(node: &NodeRef, context: &Context) -> Vec<InlineContent> {
    if let Some(element) = node.as_element() {
        // Decode a HTML element
        //
        // The following are ordered alphabetically by the output node type
        // with placeholder comments for types not implemented yet
        match element.name.local {
            // TODO: AudioObject
            // TODO: Cite
            // TODO: CiteGroup
            // TODO: CodeExpression
            // TODO: CodeFragment
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
            // TODO: Link
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
            // TODO: VideoObject

            // Recurse into all other elements thereby ignoring them but
            // not their inline children
            _ => decode_inlines(node, context),
        }
    } else if let Some(text) = node.as_text() {
        // Decode HTML text by optionally parsing it as a Markdown fragment
        // and unwrapping from `Vec<BlockContent>` to `Vec<InlineContent>`.
        if context.options.decode_markdown {
            md::decode_fragment(&text.borrow()).to_vec_inline_content()
        } else {
            vec![InlineContent::String(text.borrow().clone())]
        }
    } else {
        // Skip everything else
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;

    #[test]
    fn fragments() {
        snapshot_content("fragments/html/*.html", |content| {
            assert_json_snapshot!(decode_fragment(&content, Options::default()));
        });
    }
}
