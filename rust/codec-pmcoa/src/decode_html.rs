//! HTML decoder for PMC article pages.
//!
//! This module handles the conversion of HTML from PubMed Central article pages
//! into Stencila document structures. PMC HTML pages contain semantic markup
//! that can be parsed to extract article metadata and content.

use std::str::FromStr;

use tl::HTMLTag;

use stencila_codec::{
    DecodeInfo, DecodeOptions, Losses,
    eyre::Result,
    stencila_schema::{Article, Author, Node, Person, shortcuts::{t, p}},
};

/// Decode PMC HTML content to a Stencila [`Node`]
///
/// This function parses HTML content from PMC article pages and extracts
/// the article structure, metadata, and content into Stencila's schema.
pub async fn decode_html(
    html: &str,
    _options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    // Parse the HTML
    let dom = tl::parse(html, tl::ParserOptions::default())?;
    let parser = dom.parser();

    let mut article = Article::default();
    let mut losses = Losses::none();

    // Extract title
    if let Some(title_element) = dom.get_element_by_id("article-title-1") {
        let title_text = get_text_from_element(&title_element, parser);
        if !title_text.is_empty() {
            article.title = Some(vec![t(&title_text)]);
        }
    }

    // Extract authors - PMC typically puts authors in various elements
    // This is a basic implementation that can be enhanced
    let authors = extract_authors(&dom, parser);
    if !authors.is_empty() {
        article.authors = Some(authors);
    }

    // Extract abstract
    if let Some(abstract_element) = dom.get_element_by_id("abstract-1") {
        let abstract_text = get_text_from_element(&abstract_element, parser);
        if !abstract_text.is_empty() {
            // For now, just store as a simple paragraph - this could be enhanced to parse structure
            article.r#abstract = Some(vec![p([t(&abstract_text)])]);
        }
    }

    // Add a note about HTML parsing being basic
    losses.add("PMC HTML parsing is basic and may not capture all content structure");

    let info = DecodeInfo {
        losses,
        ..Default::default()
    };

    Ok((Node::Article(article), info))
}

/// Extract author information from PMC HTML
fn extract_authors(dom: &tl::VDom, parser: &tl::Parser) -> Vec<Author> {
    let mut authors = Vec::new();

    // PMC often uses various patterns for author information
    // Look for common author container patterns
    let author_selectors = [
        "div.contrib-group",
        "div.authors",
        ".contrib-author",
        ".author",
    ];

    for selector in &author_selectors {
        if let Some(mut iter) = dom.query_selector(selector) {
            while let Some(element_handle) = iter.next() {
                if let Some(element) = element_handle.get(parser) {
                    if let Some(tag) = element.as_tag() {
                        let author_text = get_text_from_tag(tag, parser);

                        // Basic name parsing using Stencila's Person::from_str
                        if !author_text.trim().is_empty() {
                            if let Ok(person) = Person::from_str(&author_text) {
                                authors.push(Author::Person(person));
                            }
                        }
                    }
                }
            }
        }
    }

    authors
}

/// Extract text from a DOM element (NodeHandle)
fn get_text_from_element(element: &tl::NodeHandle, parser: &tl::Parser) -> String {
    if let Some(node) = element.get(parser) {
        if let Some(tag) = node.as_tag() {
            return get_text_from_tag(tag, parser);
        }
    }
    String::new()
}

/// Extract text from an HTML tag, recursively processing child elements
fn get_text_from_tag(tag: &tl::HTMLTag, parser: &tl::Parser) -> String {
    let mut text_parts = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            text_parts.push(get_text_from_tag(child_tag, parser));
        } else if let Some(text) = child.as_raw()
            && let Some(text_str) = text.try_as_utf8_str()
        {
            // Decode HTML entities
            let decoded_text = decode_html_entities(text_str);
            text_parts.push(decoded_text);
        }
    }

    text_parts.join(" ").trim().to_string()
}

/// Decode HTML entities to their corresponding characters
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}