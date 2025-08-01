use std::str::FromStr;

use tl::{HTMLTag, Parser, ParserOptions, parse};

use codec::{
    DecodeInfo, DecodeOptions, Losses,
    common::{
        eyre::{Result, bail},
        once_cell::sync::Lazy,
        regex::Regex,
        tracing,
    },
    schema::{
        Article, Author, Block, CreativeWorkType, Date, Inline, IntegerOrString, Node, Periodical,
        Person, PublicationIssue, PublicationVolume, Reference, shortcuts::t,
    },
};

use super::decode::arxiv_id_to_doi;
use super::decode_html_blocks::*;
use super::decode_html_inlines::*;

/// Decode the response from an arXiv `html` URL to a Stencila [`Node`]
///
/// See https://github.com/brucemiller/LaTeXML for details on how this HTML is
/// generated.
#[tracing::instrument(skip(_options))]
pub(super) async fn decode_arxiv_html(
    arxiv_id: &str,
    html: &str,
    _options: Option<DecodeOptions>,
) -> Result<(Node, DecodeInfo)> {
    if html.trim().is_empty() {
        bail!("Retrieved HTML content is empty");
    }

    // Parse the HTML
    let dom = parse(html, ParserOptions::default())?;
    let parser = dom.parser();

    // Extract the <article> element (ignore <nav> and <footer> content)
    let Some(article) = dom
        .query_selector("article")
        .and_then(|mut nodes| nodes.next())
        .and_then(|article| article.get(parser))
        .and_then(|article| article.as_tag())
    else {
        bail!("No <article> tag in HTML")
    };

    // Extract <base> href from head for resolving relative URLs
    let base_href = dom
        .query_selector("base[href]")
        .and_then(|mut nodes| nodes.next())
        .and_then(|node| node.get(parser))
        .and_then(|node| node.as_tag())
        .and_then(|node| get_attr(node, "href"));

    // Decode article
    let mut context = ArxivDecodeContext::new(base_href);
    let mut article = decode_article(parser, article, &mut context);

    // Set doi, and other metadata
    if !arxiv_id.is_empty() {
        article.doi = Some(arxiv_id_to_doi(arxiv_id));
        article.options.repository = Some("https://arxiv.org".into());
        article.options.path = Some(["html/", arxiv_id].concat());
    }

    Ok((
        Node::Article(article),
        DecodeInfo {
            losses: context.losses,
            ..Default::default()
        },
    ))
}

pub struct ArxivDecodeContext {
    losses: Losses,
    pub appendix_started: bool,
    pub base_href: Option<String>,
}

impl ArxivDecodeContext {
    pub fn new(base_href: Option<String>) -> Self {
        Self {
            losses: Losses::none(),
            appendix_started: false,
            base_href,
        }
    }

    /// Resolve a potentially relative URL using the base href from context
    pub fn resolve_url(&self, url: &str) -> String {
        // If URL is already absolute or a data URL, return as-is
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("data:") {
            return url.to_string();
        }

        // If we have a base href, use it to resolve the relative URL
        if let Some(base_href) = &self.base_href {
            // Remove start and end slashes from base_href if present
            let base = base_href.trim_start_matches('/').trim_end_matches('/');

            // Remove leading slash from url if present (for relative URLs)
            let relative_url = url.trim_start_matches('/');

            format!("https://export.arxiv.org/{base}/{relative_url}")
        } else {
            // No base href available, return URL as-is
            url.to_string()
        }
    }

    pub fn add_loss(&mut self, tag: &HTMLTag) {
        let tag_name = tag.name().as_utf8_str();

        let class = tag
            .attributes()
            .class()
            .map(|cls| format!(" class=\"{}\"", cls.as_utf8_str()))
            .unwrap_or_default();

        self.losses.add(format!("<{tag_name}{class}>",))
    }
}

// Helper functions for common HTML attribute extraction patterns

/// Get the class attribute as a string, or empty string if not present
pub fn get_class(tag: &HTMLTag) -> String {
    tag.attributes()
        .class()
        .map(|cls| cls.as_utf8_str().to_string())
        .unwrap_or_default()
}

/// Get an attribute value as Option<String>
pub fn get_attr(tag: &HTMLTag, name: &str) -> Option<String> {
    tag.attributes()
        .get(name)
        .flatten()
        .map(|bytes| bytes.as_utf8_str().to_string())
}

/// Get href attribute, extracting hash fragment for internal links
pub fn get_href_target(tag: &HTMLTag) -> Option<String> {
    get_attr(tag, "href").map(|href| {
        // For internal links, extract only the hash part
        if let Some(hash_pos) = href.find('#') {
            href[hash_pos + 1..].to_string()
        } else if let Some(rest) = href.strip_prefix('#') {
            rest.to_string()
        } else {
            // External link, keep full URL
            href
        }
    })
}

/// Decode common HTML entities
pub fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Check if a text string is an email address or contains email pattern
fn is_email_address(text: &str) -> bool {
    let trimmed = text.trim();

    // Check for @ symbol
    if !trimmed.contains('@') {
        return false;
    }

    // Find @ position
    if let Some(at_pos) = trimmed.find('@') {
        let after_at = &trimmed[at_pos + 1..];
        // Must have a dot after @ and reasonable length
        if after_at.contains('.') && after_at.len() > 3 {
            return true;
        }
    }

    false
}

/// Check if a text string is an organization/institution name
fn is_organization_name(text: &str) -> bool {
    let trimmed = text.trim().to_lowercase();

    // Skip very short strings (likely abbreviations or names)
    if trimmed.len() < 3 {
        return false;
    }

    // Check for exact organizational patterns (more precise)
    if trimmed.starts_with("department of")
        || trimmed.starts_with("university of")
        || trimmed.starts_with("institute of")
        || trimmed.starts_with("school of")
        || trimmed.starts_with("college of")
        || trimmed.starts_with("center of")
        || trimmed.starts_with("centre of")
        || trimmed == "department"
        || trimmed == "university"
        || trimmed == "institute"
        || trimmed == "college"
        || trimmed == "school"
        || trimmed == "laboratory"
        || trimmed == "research"
        || trimmed == "foundation"
        || trimmed == "organization"
        || trimmed == "organisation"
        || trimmed == "company"
        || trimmed == "corporation"
    {
        return true;
    }

    // Check for clear organizational endings (more specific)
    if trimmed.ends_with(" university")
        || trimmed.ends_with(" institute")
        || trimmed.ends_with(" college")
        || trimmed.ends_with(" laboratory")
        || trimmed.ends_with(" foundation")
        || trimmed.ends_with(" research")
        || trimmed.ends_with(" inc.")
        || trimmed.ends_with(" llc")
        || trimmed.ends_with(" ltd.")
        || trimmed.ends_with(" corp.")
        || trimmed.ends_with(" corporation")
        || (trimmed.ends_with(" ai") && trimmed.len() > 5)
    // Avoid filtering short names like "Ai"
    {
        return true;
    }

    // Check for multi-word department/institutional patterns
    if trimmed.contains(" department")
        || trimmed.contains(" university")
        || trimmed.contains(" institute")
        || trimmed.contains(" college")
        || trimmed.contains(" laboratory")
        || trimmed.contains(" hospital")
        || trimmed.contains(" medical center")
    {
        return true;
    }

    false
}

/// Extract text content from an HTML element
pub fn get_text(parser: &Parser, tag: &HTMLTag) -> String {
    let mut text_parts = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            text_parts.push(get_text(parser, child_tag));
        } else if let Some(text) = child.as_raw() {
            if let Some(text_str) = text.try_as_utf8_str() {
                text_parts.push(decode_html_entities(text_str));
            }
        }
    }

    text_parts.join(" ").trim().to_string()
}

/// Get text content excluding superscript elements (for author names)
pub fn get_text_excluding_superscripts(parser: &Parser, tag: &HTMLTag) -> String {
    let mut text_parts = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            // Skip superscript elements
            if child_tag.name().as_utf8_str() == "sup" || child_class.contains("ltx_sup") {
                continue;
            }

            text_parts.push(get_text_excluding_superscripts(parser, child_tag));
        } else if let Some(text) = child.as_raw() {
            if let Some(text_str) = text.try_as_utf8_str() {
                text_parts.push(decode_html_entities(text_str));
            }
        }
    }

    text_parts.join(" ").trim().to_string()
}

/// Decode the root <article> element into aa Stencila [`Article`]
fn decode_article(parser: &Parser, article: &HTMLTag, context: &mut ArxivDecodeContext) -> Article {
    let mut title = None;
    let mut authors = Vec::new();
    let mut abstract_ = None;
    let mut references = Vec::new();
    let mut content = Vec::new();

    for child in article
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();
            let class = get_class(tag);

            match tag_name.as_ref() {
                "h1" if class.contains("ltx_title_document") => {
                    title = Some(decode_inlines(parser, tag, context));
                }
                "div" if class.contains("ltx_authors") => {
                    authors = decode_authors(parser, tag);
                }
                "div" if class.contains("ltx_abstract") => {
                    abstract_ = Some(decode_abstract(parser, tag, context));
                }
                "section" if class.contains("ltx_bibliography") => {
                    references = decode_bibliography(parser, tag, context);
                }
                _ => {
                    content.append(&mut decode_blocks(parser, tag, context));
                }
            }
        }
    }

    if references.is_empty() {
        if let Some(tag) = article
            .query_selector(parser, ".ltx_bibliography")
            .and_then(|mut query| query.next())
            .and_then(|node_handle| node_handle.get(parser))
            .and_then(|node| node.as_tag())
        {
            references = decode_bibliography(parser, tag, context);
        }
    }

    Article {
        title,
        authors: (!authors.is_empty()).then_some(authors),
        r#abstract: abstract_,
        references: (!references.is_empty()).then_some(references),
        content,
        ..Default::default()
    }
}

/// Extract plain text from a vector of inlines
pub fn extract_text_from_inlines(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .filter_map(|inline| match inline {
            Inline::Text(text) => Some(text.value.to_string()),
            _ => None,
        })
        .collect::<Vec<String>>()
        .join("")
}

/// Decode author information from div.ltx_authors
fn decode_authors(parser: &Parser, tag: &HTMLTag) -> Vec<Author> {
    let mut authors = Vec::new();

    // Look for ltx_creator elements within the authors div
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_class.contains("ltx_creator") {
                authors.append(&mut decode_author_from_creator(parser, child_tag));
            }
        }
    }

    // If no individual creators found, fall back to extracting all text and parsing
    if authors.is_empty() {
        let full_text = get_text_excluding_superscripts(parser, tag);
        authors = decode_authors_from_text(&full_text);
    }

    authors
}

/// Decode a ltx_personname element that may contain authors and affiliations separated by <br> tags
fn decode_personname_element(parser: &Parser, tag: &HTMLTag) -> Vec<Author> {
    let mut authors = Vec::new();
    let mut current_author_parts = Vec::new();

    // Iterate through child nodes to separate author names from affiliations
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let tag_name = child_tag.name().as_utf8_str().to_lowercase();

            if tag_name == "br" {
                // <br> tag indicates separation between author name and affiliation,
                // or between different affiliation lines
                if !current_author_parts.is_empty() {
                    // We have author name parts, process them as potentially multiple authors
                    let author_text = current_author_parts.join(" ");
                    let mut parsed_authors = decode_authors_from_text(&author_text);
                    authors.append(&mut parsed_authors);
                    current_author_parts.clear();
                }
                // After <br>, following content is likely affiliation, skip it for now
                // TODO: In future, associate affiliations with authors
            } else {
                // Extract text from other elements (like <a> tags for author names)
                // Use the function that excludes superscripts for cleaner author names
                let element_text = get_text_excluding_superscripts(parser, child_tag);
                if !element_text.trim().is_empty() {
                    current_author_parts.push(element_text);
                }
            }
        } else if let Some(text) = child.as_raw() {
            if let Some(text_str) = text.try_as_utf8_str() {
                let decoded_text = decode_html_entities(text_str).trim().to_string();
                if !decoded_text.is_empty() && decoded_text != "&" {
                    current_author_parts.push(decoded_text);
                }
            }
        }
    }

    // Process any remaining author parts
    if !current_author_parts.is_empty() {
        let author_text = current_author_parts.join(" ");
        let mut parsed_authors = decode_authors_from_text(&author_text);
        authors.append(&mut parsed_authors);
    }

    // If no authors found through structure parsing, fall back to text-based parsing
    if authors.is_empty() {
        let full_text = get_text_excluding_superscripts(parser, tag);
        authors = decode_authors_from_text(&full_text);
    }

    authors
}

/// Decode authors from a span.ltx_creator element
fn decode_author_from_creator(parser: &Parser, tag: &HTMLTag) -> Vec<Author> {
    // Look for ltx_personname within the creator
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_class.contains("ltx_personname") {
                // Parse authors and affiliations from the personname element
                return decode_personname_element(parser, child_tag);
            }
        }
    }

    // Fallback: extract all text from creator
    let creator_text = get_text_excluding_superscripts(parser, tag);
    decode_authors_from_text(&creator_text)
}

/// Parse multiple authors from a text string using Person::from_str for each
pub fn decode_authors_from_text(text: &str) -> Vec<Author> {
    let text = text.trim().trim_end_matches(['.', ',', ';']);

    static SPLIT_BY: Lazy<Regex> =
        Lazy::new(|| Regex::new(r",|&|\band\b|\n").expect("invalid regex"));
    let standard_parts: Vec<&str> = SPLIT_BY.split(text).collect();

    // If standard separators worked, use them
    if standard_parts.len() > 1 {
        let authors: Vec<String> = standard_parts
            .into_iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter(|s| !is_email_address(s)) // Filter out email addresses
            .filter(|s| !is_organization_name(s)) // Filter out organization names
            .map(|s| s.to_string())
            .collect();

        return authors
            .into_iter()
            .map(|author| Person::from_str(&author).unwrap_or_default())
            .map(Author::Person)
            .collect();
    }

    // If no standard separators, try to detect superscript-separated names
    // Pattern: "FirstName LastName 1  NextFirst NextLast 1  ..."
    if let Some(sup_authors) = parse_superscript_separated_authors(text) {
        if sup_authors.len() > 1 {
            return sup_authors;
        }
    }

    // Fallback: treat as single author
    let trimmed = text.trim();
    if !trimmed.is_empty() && !is_email_address(trimmed) && !is_organization_name(trimmed) {
        if let Ok(person) = Person::from_str(trimmed) {
            return vec![Author::Person(person)];
        }
    }

    vec![]
}

/// Try to parse authors separated by superscript numbers
fn parse_superscript_separated_authors(text: &str) -> Option<Vec<Author>> {
    // Pattern to detect: "Name1 Number  Name2 Number  ..."
    // Look for digit followed by 2+ spaces pattern
    static SUP_PATTERN: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\d+\s{2,}").expect("invalid regex"));

    // Check if text contains the pattern
    if !SUP_PATTERN.is_match(text) {
        return None;
    }

    // Split by "digit + multiple spaces"
    let parts: Vec<&str> = SUP_PATTERN.split(text).collect();

    // Clean up each part and try to parse as author
    let mut authors = Vec::new();
    for part in parts {
        let cleaned = part
            .trim()
            .trim_end_matches(char::is_numeric) // Remove trailing numbers
            .trim();

        if !cleaned.is_empty() && !is_email_address(cleaned) && !is_organization_name(cleaned) {
            if let Ok(person) = Person::from_str(cleaned) {
                authors.push(Author::Person(person));
            }
        }
    }

    if authors.len() > 1 {
        Some(authors)
    } else {
        None
    }
}

/// Decode abstract from div.ltx_abstract
fn decode_abstract(parser: &Parser, tag: &HTMLTag, context: &mut ArxivDecodeContext) -> Vec<Block> {
    decode_blocks(parser, tag, context)
}

/// Extract label and other inlines from a tag
pub fn extract_label_and_inlines(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> (Option<String>, Vec<Inline>) {
    let mut label = None;
    let mut content_parts = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_tag.name().as_utf8_str() == "span" && child_class.contains("ltx_tag") {
                let label_text = get_text(parser, child_tag).trim().to_string();
                if !label_text.is_empty() {
                    label = Some(label_text);
                }
            } else {
                content_parts.extend(decode_inlines(parser, child_tag, context));
            }
        } else if let Some(text) = child.as_raw() {
            let text_content = text.try_as_utf8_str().unwrap_or_default().trim();
            if !text_content.is_empty() {
                content_parts.push(t(text_content));
            }
        }
    }

    (label, content_parts)
}

/// Extract both LaTeX code and MathML content from a math element
pub fn extract_latex_and_mathml(parser: &Parser, tag: &HTMLTag) -> (String, String) {
    let mathml = if tag.name().as_utf8_str() == "math" {
        tag.outer_html(parser)
    } else {
        // For equation tables, look for math elements within
        let found_math = find_math_elements(parser, tag);

        if found_math.is_empty() {
            // If no MathML found, wrap content in math tags
            let inner_html = tag.inner_html(parser);
            if inner_html.trim().is_empty() {
                String::new()
            } else {
                format!("<math>{inner_html}</math>")
            }
        } else {
            found_math
        }
    };

    let latex = get_attr(tag, "alttext")
        .filter(|s| !s.is_empty())
        .or_else(|| {
            let annotations_latex = extract_latex_from_annotations(parser, tag);
            if annotations_latex.is_empty() {
                None
            } else {
                Some(annotations_latex)
            }
        })
        .unwrap_or_else(|| {
            if mathml.is_empty() {
                String::new()
            } else {
                "\\text{Math content}".to_string()
            }
        });

    (latex, mathml)
}

/// Find math elements within a container
fn find_math_elements(parser: &Parser, tag: &HTMLTag) -> String {
    tag.children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
        .filter_map(|child| child.as_tag())
        .filter_map(|child_tag| {
            if child_tag.name().as_utf8_str() == "math" {
                Some(child_tag.outer_html(parser))
            } else {
                // Recursively search for math elements
                let nested_math = find_math_elements(parser, child_tag);
                if nested_math.is_empty() {
                    None
                } else {
                    Some(nested_math)
                }
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Extract LaTeX code from annotation elements within MathML
fn extract_latex_from_annotations(parser: &Parser, tag: &HTMLTag) -> String {
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            // Look for annotation elements with LaTeX
            if child_tag.name().as_utf8_str() == "annotation" {
                if let Some(encoding) = child_tag.attributes().get("encoding").flatten() {
                    if encoding.as_utf8_str().contains("tex") {
                        return get_text(parser, child_tag);
                    }
                }
            }

            // Recursively search in child elements
            let nested_latex = extract_latex_from_annotations(parser, child_tag);
            if !nested_latex.is_empty() {
                return nested_latex;
            }
        }
    }

    String::new()
}

/// Decode bibliography section and extract Reference objects
fn decode_bibliography(
    parser: &Parser,
    tag: &HTMLTag,
    _context: &mut ArxivDecodeContext,
) -> Vec<Reference> {
    let mut references = Vec::new();

    // Look for ul.ltx_biblist within the bibliography section
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_tag.name().as_utf8_str() == "ul" && child_class.contains("ltx_biblist") {
                // Process each li.ltx_bibitem as a reference
                for bibitem in child_tag
                    .children()
                    .top()
                    .iter()
                    .flat_map(|h| h.get(parser))
                {
                    if let Some(item_tag) = bibitem.as_tag() {
                        let item_class = item_tag
                            .attributes()
                            .class()
                            .map(|cls| cls.as_utf8_str())
                            .unwrap_or_default();

                        if item_tag.name().as_utf8_str() == "li"
                            && item_class.contains("ltx_bibitem")
                        {
                            if let Some(reference) = decode_reference(parser, item_tag) {
                                references.push(reference);
                            }
                        }
                    }
                }
            }
        }
    }

    references
}

/// Decode a single bibliography item into a Reference
fn decode_reference(parser: &Parser, tag: &HTMLTag) -> Option<Reference> {
    // Extract id from the li element
    let id = tag
        .attributes()
        .get("id")
        .flatten()
        .map(|bytes| bytes.as_utf8_str().to_string());

    let mut authors = Vec::new();
    let mut date = None;
    let mut title = None;
    let mut publication_info = None;
    let mut page_start = None;
    let mut page_end = None;
    let mut pagination = None;

    // Find all ltx_bibblock spans
    let mut bibblocks = Vec::new();
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            if child_tag.name().as_utf8_str() == "span" {
                let class = get_class(child_tag);
                if class.contains("ltx_bibblock") {
                    let text = get_text(parser, child_tag).trim().to_string();
                    if !text.is_empty() {
                        bibblocks.push(text);
                    }
                }
            }
        }
    }

    // Parse bibblocks in order
    for (i, block) in bibblocks.iter().enumerate() {
        match i {
            0 => {
                // First block: author details and year
                let (parsed_authors, parsed_date) = parse_author_block(block);
                authors = parsed_authors;
                date = parsed_date;
            }
            1 => {
                // Second block: title
                title = Some(vec![t(block.trim_end_matches('.'))]);
            }
            2 => {
                // Third block: publication details
                if let Some((pub_info, start_page, end_page, page_info)) =
                    parse_publication_block(block)
                {
                    publication_info = Some(pub_info);
                    page_start = start_page;
                    page_end = end_page;
                    pagination = page_info;
                }
            }
            _ => {
                // Additional blocks - could be more publication details
                if publication_info.is_none() {
                    if let Some((pub_info, start_page, end_page, page_info)) =
                        parse_publication_block(block)
                    {
                        publication_info = Some(pub_info);
                        page_start = start_page;
                        page_end = end_page;
                        pagination = page_info;
                    }
                }
            }
        }
    }

    // If we have at least some content, create the reference
    if !bibblocks.is_empty() {
        Some(Reference {
            id,
            authors: if authors.is_empty() {
                None
            } else {
                Some(authors)
            },
            date,
            title,
            is_part_of: publication_info,
            page_start,
            page_end,
            pagination,
            ..Default::default()
        })
    } else {
        None
    }
}

/// Parse author block text to extract authors and date
fn parse_author_block(text: &str) -> (Vec<Author>, Option<Date>) {
    // Extract year from the end - simple string parsing instead of regex
    let mut date = None;
    let mut author_text = text;
    let author_text_string: String;

    // Look for year pattern like "(2023)" at the end
    if let Some(open_paren) = text.rfind('(') {
        if let Some(close_paren) = text[open_paren..].find(')') {
            let year_part = &text[open_paren + 1..open_paren + close_paren];
            if year_part.len() == 4 && year_part.chars().all(|c| c.is_ascii_digit()) {
                date = Some(Date {
                    value: year_part.to_string(),
                    ..Default::default()
                });
                author_text = &text[..open_paren];
            }
        }
    }

    // If no parentheses, look for 4-digit year embedded in the text
    if date.is_none() {
        // Split by commas first to handle cases like "Author1, Author2, 2010, Journal"
        let parts: Vec<&str> = text.split(',').map(|s| s.trim()).collect();

        // Find the first part that's a 4-digit year
        let mut year_index = None;
        for (i, part) in parts.iter().enumerate() {
            let clean_part = part.trim_end_matches('.');
            if clean_part.len() == 4 && clean_part.chars().all(|c| c.is_ascii_digit()) {
                date = Some(Date {
                    value: clean_part.to_string(),
                    ..Default::default()
                });
                year_index = Some(i);
                break;
            }
        }

        // If we found a year, only take the parts before it as authors
        if let Some(idx) = year_index {
            author_text_string = parts[..idx].join(", ");
            author_text = &author_text_string;
        }
    }

    // Parse authors
    let authors = decode_authors_from_text(author_text.trim().trim_end_matches(','));

    (authors, date)
}

/// Parse page range like "1221–1244." or "1125-1161" into start and end pages
fn parse_page_range(page_info: &str) -> (Option<IntegerOrString>, Option<IntegerOrString>) {
    let trimmed = page_info.trim().trim_end_matches('.');

    // Try splitting by different dash characters
    let parts: Vec<&str> = if trimmed.contains('–') {
        trimmed.split('–').collect()
    } else if trimmed.contains('—') {
        trimmed.split('—').collect()
    } else if trimmed.contains('-') {
        trimmed.split('-').collect()
    } else {
        vec![trimmed]
    };

    if parts.len() >= 2 {
        // Page range found
        let start_part = parts[0].trim();
        let end_part = parts[1].trim();

        let page_start = parse_single_page(start_part);
        let page_end = parse_single_page(end_part);

        (page_start, page_end)
    } else {
        // Single page number
        let page_num = parse_single_page(trimmed);
        (page_num.clone(), page_num)
    }
}

/// Parse a single page number, handling both numeric and text formats
fn parse_single_page(page_text: &str) -> Option<IntegerOrString> {
    let cleaned = page_text.trim().trim_end_matches('.');

    if cleaned.is_empty() {
        return None;
    }

    // Try to parse as integer
    if cleaned.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(page_num) = cleaned.parse::<i64>() {
            return Some(IntegerOrString::Integer(page_num));
        }
    }

    // Fall back to string
    Some(IntegerOrString::String(cleaned.to_string()))
}

/// Parse volume and issue information, creating proper nested structure
fn parse_volume_and_issue_info(volume_info: &str, periodical: Periodical) -> Box<CreativeWorkType> {
    // Check if there's an issue number in parentheses like "14(4)"
    if let Some(open_paren) = volume_info.find('(') {
        if let Some(close_paren) = volume_info[open_paren..].find(')') {
            let volume_part = volume_info[..open_paren].trim();
            let issue_part = &volume_info[open_paren + 1..open_paren + close_paren];

            if !volume_part.is_empty() && !issue_part.is_empty() {
                // Create nested structure: PublicationIssue -> PublicationVolume -> Periodical
                let publication_volume = PublicationVolume {
                    is_part_of: Some(Box::new(CreativeWorkType::Periodical(periodical))),
                    volume_number: Some(IntegerOrString::String(volume_part.to_string())),
                    ..Default::default()
                };

                let publication_issue = PublicationIssue {
                    is_part_of: Some(Box::new(CreativeWorkType::PublicationVolume(
                        publication_volume,
                    ))),
                    issue_number: Some(IntegerOrString::String(issue_part.to_string())),
                    ..Default::default()
                };

                return Box::new(CreativeWorkType::PublicationIssue(publication_issue));
            }
        }
    }

    // No issue found, just volume
    let publication_volume = PublicationVolume {
        is_part_of: Some(Box::new(CreativeWorkType::Periodical(periodical))),
        volume_number: Some(IntegerOrString::String(volume_info.to_string())),
        ..Default::default()
    };

    Box::new(CreativeWorkType::PublicationVolume(publication_volume))
}

type PublicationInfo = (
    Box<CreativeWorkType>,
    Option<IntegerOrString>,
    Option<IntegerOrString>,
    Option<String>,
);

/// Parse publication block to extract venue information and page range
fn parse_publication_block(text: &str) -> Option<PublicationInfo> {
    // Check for arXiv preprint pattern - simple string parsing
    if text.contains("arXiv") {
        // Find arXiv: pattern
        if let Some(start) = text.find("arXiv:") {
            let after_arxiv = &text[start + 6..];
            // Extract the arXiv number (digits.digits format)
            let mut end = 0;
            for (i, ch) in after_arxiv.char_indices() {
                if ch.is_ascii_digit() || ch == '.' {
                    end = i + 1;
                } else {
                    break;
                }
            }

            if end > 0 {
                let arxiv_id = &after_arxiv[..end];
                let periodical = Periodical {
                    name: Some("arXiv".to_string()),
                    ..Default::default()
                };

                return Some((
                    Box::new(CreativeWorkType::PublicationVolume(PublicationVolume {
                        is_part_of: Some(Box::new(CreativeWorkType::Periodical(periodical))),
                        volume_number: Some(IntegerOrString::String(arxiv_id.to_string())),
                        ..Default::default()
                    })),
                    None,
                    None,
                    None,
                ));
            }
        }
    }

    // Check for journal pattern (italic text followed by volume/pages)
    if let Some(comma_pos) = text.find(',') {
        let journal_part = text[..comma_pos].trim();
        let details_part = text[comma_pos + 1..].trim();

        // If journal_part is not empty, create periodical
        if !journal_part.is_empty() {
            let periodical = Periodical {
                name: Some(journal_part.to_string()),
                ..Default::default()
            };

            // Parse volume/issue and page information
            let (volume_info, page_info) = if let Some(colon_pos) = details_part.find(':') {
                let vol_part = details_part[..colon_pos].trim();
                let page_part = details_part[colon_pos + 1..].trim();
                (vol_part, Some(page_part))
            } else {
                (details_part.trim(), None)
            };

            let creative_work = parse_volume_and_issue_info(volume_info, periodical);

            // Parse page range if present
            let (page_start, page_end, pagination) = if let Some(pages) = page_info {
                let (start, end) = parse_page_range(pages);
                // Check if we got a clean range (both start and end are integers)
                let is_clean_range = matches!(
                    (&start, &end),
                    (
                        Some(IntegerOrString::Integer(_)),
                        Some(IntegerOrString::Integer(_))
                    )
                );

                if is_clean_range {
                    // Clean numeric range, use page_start and page_end
                    (start, end, None)
                } else if start.is_some() || end.is_some() {
                    // Partial or string-based parsing, prefer pagination with structured pages as fallback
                    (start, end, Some(pages.trim().to_string()))
                } else {
                    // Parsing failed completely, use pagination only
                    (None, None, Some(pages.trim().to_string()))
                }
            } else {
                (None, None, None)
            };

            return Some((creative_work, page_start, page_end, pagination));
        }
    }

    None
}
