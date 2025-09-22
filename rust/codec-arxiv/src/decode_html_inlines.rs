//! Inline-level HTML element decoder for arXiv LaTeXML content.
//!
//! This module handles conversion of inline HTML elements (text formatting, links,
//! math, citations, etc.) from LaTeXML-generated HTML into Stencila Inline nodes.
//! Special attention is paid to preserving academic citation structures and
//! mathematical content with both LaTeX and MathML representations.

use tl::{HTMLTag, Parser};

use stencila_codec::stencila_schema::{
    Citation, CitationGroup, CitationMode, CodeInline, ImageObject, Inline, Link, MathInline,
    shortcuts::{em, stg, stk, sub, sup, t, u},
};

use super::decode_html::{
    ArxivDecodeContext, extract_latex_and_mathml, extract_text_from_inlines, get_attr,
    get_href_target,
};

/// Decode inline elements with careful whitespace preservation
///
/// LaTeXML generates complex nested inline structures where whitespace
/// significance can vary. This function attempts to preserve meaningful
/// whitespace while normalizing excessive spacing.
pub fn decode_inlines(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> Vec<Inline> {
    let mut inlines = Vec::new();
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            inlines.extend(decode_inline(parser, tag, context));
        } else if let Some(text) = child.as_raw() {
            let text_content = text.try_as_utf8_str().unwrap_or_default();

            // Preserve whitespace structure while normalizing excessive spacing
            if text_content.chars().any(|c| !c.is_whitespace()) {
                // Text contains non-whitespace: normalize internal spacing but preserve boundaries
                let normalized = text_content
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                // Maintain leading/trailing whitespace patterns from original
                let result = match (
                    text_content.starts_with(char::is_whitespace),
                    text_content.ends_with(char::is_whitespace),
                ) {
                    (true, true) => format!(" {normalized} "),
                    (true, false) => format!(" {normalized}"),
                    (false, true) => format!("{normalized} "),
                    (false, false) => normalized,
                };

                if !result.is_empty() {
                    inlines.push(t(&result));
                }
            } else if !text_content.is_empty() {
                // Pure whitespace content: preserve as single space
                inlines.push(t(" "));
            }
        }
    }
    inlines
}

/// Decode an inline element
pub fn decode_inline(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> Vec<Inline> {
    let mut inlines = Vec::new();

    let tag_name = tag.name().as_utf8_str();
    let _tag_class = tag
        .attributes()
        .class()
        .map(|cls| cls.as_utf8_str())
        .unwrap_or_default();

    match tag_name.as_ref() {
        "cite" => {
            if _tag_class.contains("ltx_cite") {
                inlines.push(decode_citation(parser, tag, &_tag_class, context));
            } else {
                inlines.append(&mut decode_inlines(parser, tag, context));
                context.add_loss(tag);
            }
        }
        "math" => {
            inlines.push(decode_math_inline(parser, tag, context));
        }
        "em" | "i" | "strong" | "bold" | "u" | "sub" | "sup" | "s" => {
            inlines.push(decode_mark(parser, tag, &tag_name, context))
        }
        "a" => inlines.push(decode_a(parser, tag, context)),
        "img" => inlines.push(decode_img(parser, tag, context)),
        "svg" => {
            if _tag_class.contains("ltx_picture") {
                inlines.push(decode_svg_picture_inline(parser, tag, context));
            } else {
                // Handle other SVG elements as needed
                inlines.append(&mut decode_inlines(parser, tag, context));
                context.add_loss(tag);
            }
        }
        "span" => {
            inlines.append(&mut decode_span(parser, tag, context));
        }
        _ => {
            // Unhandled tag: just decode children into inlines but record loss
            inlines.append(&mut decode_inlines(parser, tag, context));
            context.add_loss(tag);
        }
    }

    inlines
}

/// Decode an <a> element into a [`Link`] node
pub fn decode_a(parser: &Parser, tag: &HTMLTag, context: &mut ArxivDecodeContext) -> Inline {
    let target = get_href_target(tag).unwrap_or_default();
    let title = get_attr(tag, "title");
    let content = decode_inlines(parser, tag, context);

    Inline::Link(Link {
        target,
        content,
        title,
        ..Default::default()
    })
}

/// Decode LaTeX citation elements into Stencila citations
///
/// LaTeXML preserves different citation styles from LaTeX:
/// - \citep{} becomes ltx_citemacro_citep (parenthetical citations)
/// - \citet{} becomes ltx_citemacro_citet (narrative citations)
/// - \cite{} becomes ltx_citemacro_cite (basic citations)
///
/// Citations may contain multiple references, requiring careful parsing
/// to separate author names, years, and reference targets.
pub fn decode_citation(
    parser: &Parser,
    tag: &HTMLTag,
    tag_class: &str,
    _context: &mut ArxivDecodeContext,
) -> Inline {
    // Map LaTeXML citation classes to Stencila citation modes
    let mode = if tag_class.contains("ltx_citemacro_citep") {
        CitationMode::Parenthetical
    } else if tag_class.contains("ltx_citemacro_citet") || tag_class.contains("ltx_citemacro_cite")
    {
        CitationMode::Narrative
    } else {
        CitationMode::Parenthetical // Safe default
    };

    // Extract all reference links within this citation
    let mut ref_links = Vec::new();
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag()
            && child_tag.name().as_utf8_str() == "a"
        {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            // Look for ltx_ref links that point to bibliography entries
            if child_class.contains("ltx_ref")
                && let Some(href) = child_tag.attributes().get("href").flatten()
            {
                let full_url = href.as_utf8_str();
                // Extract fragment identifier (the part after #)
                let target = extract_fragment_id(&full_url);
                ref_links.push((target, child_tag));
            }
        }
    }

    // Handle multiple citations as CitationGroup, single citations as Citation
    if ref_links.len() > 1 {
        create_citation_group(parser, tag, ref_links)
    } else {
        create_single_citation(parser, tag, ref_links, mode)
    }
}

/// Extract fragment identifier from URL (part after #)
fn extract_fragment_id(url: &str) -> String {
    if let Some(hash_pos) = url.find('#') {
        url[hash_pos + 1..].to_string()
    } else if let Some(rest) = url.strip_prefix('#') {
        rest.to_string()
    } else {
        // External URL, keep as-is
        url.to_string()
    }
}

/// Create a citation group for multiple references
///
/// Does not set citation mode on citation items because they are part of the
/// group which is by definition, parenthetical.
fn create_citation_group(
    parser: &Parser,
    tag: &HTMLTag,
    ref_links: Vec<(String, &HTMLTag)>,
) -> Inline {
    let mut items = Vec::new();
    let total_count = ref_links.len();

    // Parse the citation content by extracting text around each link
    for (i, (target, link_tag)) in ref_links.into_iter().enumerate() {
        let citation_text = extract_citation_text_for_link(parser, tag, link_tag);
        let cleaned_text = clean_citation_text_in_group(&citation_text, i, total_count);

        let mut citation = Citation {
            target,
            ..Default::default()
        };

        if !cleaned_text.is_empty() {
            citation.options.content = Some(vec![t(&cleaned_text)]);
        }

        items.push(citation);
    }

    Inline::CitationGroup(CitationGroup {
        items,
        ..Default::default()
    })
}

/// Create a single citation
fn create_single_citation(
    parser: &Parser,
    tag: &HTMLTag,
    ref_links: Vec<(String, &HTMLTag)>,
    mode: CitationMode,
) -> Inline {
    // Single citation - use original logic but simplified
    let target = ref_links
        .first()
        .map(|(target, _)| target.clone())
        .unwrap_or_default();

    let full_text = get_text_content(parser, tag);
    let cleaned_text = clean_citation_text(&full_text);

    let mut citation = Citation {
        target: if target.is_empty() && !cleaned_text.is_empty() {
            cleaned_text.clone()
        } else {
            target
        },
        citation_mode: Some(mode),
        ..Default::default()
    };

    if !cleaned_text.is_empty() {
        citation.options.content = Some(vec![t(&cleaned_text)]);
    }

    Inline::Citation(citation)
}

/// Clean citation text by removing outer parentheses/brackets and deduplicating commas
fn clean_citation_text(text: &str) -> String {
    let mut cleaned = text.trim().to_string();

    // Remove outer parentheses or square brackets
    if (cleaned.starts_with('(') && cleaned.ends_with(')'))
        || (cleaned.starts_with('[') && cleaned.ends_with(']'))
    {
        cleaned = cleaned[1..cleaned.len() - 1].to_string();
    }

    // Deduplicate commas and clean up whitespace
    let parts: Vec<&str> = cleaned
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect();

    parts.join(", ")
}

/// Clean citation text for citations within a group, handling distributed parentheses/brackets
fn clean_citation_text_in_group(text: &str, index: usize, total_count: usize) -> String {
    let mut cleaned = text.trim().to_string();

    // For the first citation, remove leading parenthesis or bracket
    if index == 0 && (cleaned.starts_with('(') || cleaned.starts_with('[')) {
        cleaned = cleaned[1..].to_string();
    }

    // For the last citation, remove trailing parenthesis or bracket
    if index == total_count - 1 && (cleaned.ends_with(')') || cleaned.ends_with(']')) {
        cleaned = cleaned[..cleaned.len() - 1].to_string();
    }

    // Deduplicate commas and clean up whitespace
    let parts: Vec<&str> = cleaned
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .collect();

    parts.join(", ")
}

/// Get text content from an HTML tag
fn get_text_content(parser: &Parser, tag: &HTMLTag) -> String {
    use crate::decode_html::decode_html_entities;

    tag.children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
        .filter_map(|node| {
            if let Some(text) = node.as_raw() {
                Some(decode_html_entities(
                    text.try_as_utf8_str().unwrap_or_default(),
                ))
            } else {
                node.as_tag()
                    .map(|child_tag| get_text_content(parser, child_tag))
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Extract citation text for a specific link within a citation element
fn extract_citation_text_for_link(
    parser: &Parser,
    citation_tag: &HTMLTag,
    target_link: &HTMLTag,
) -> String {
    // Get all text and link nodes in order
    let mut nodes = Vec::new();
    collect_citation_nodes(parser, citation_tag, &mut nodes);

    // Find the target link in the nodes
    let mut target_index = None;
    for (i, node) in nodes.iter().enumerate() {
        if let CitationNode::Link(link_tag, _) = node
            && std::ptr::eq(*link_tag as *const _, target_link as *const _)
        {
            target_index = Some(i);
            break;
        }
    }

    let Some(target_idx) = target_index else {
        return String::new();
    };

    // Get the year from the target link
    let year = if let CitationNode::Link(_, year) = &nodes[target_idx] {
        year.clone()
    } else {
        String::new()
    };

    // Extract author text: look for text nodes that come before this link
    // and after the previous link (if any)
    let mut start_idx = 0;

    // Find the previous link to establish the boundary
    for i in (0..target_idx).rev() {
        if let CitationNode::Link(_, _) = &nodes[i] {
            start_idx = i + 1;
            break;
        }
    }

    // Collect all text from start_idx to target_idx
    let mut text_parts = Vec::new();
    for node in nodes.iter().take(target_idx).skip(start_idx) {
        if let CitationNode::Text(text) = node {
            text_parts.push(text.clone());
        }
    }

    // Join and clean the text
    let combined_text = text_parts.join("");

    // Clean up the author text - remove trailing commas and extra whitespace
    let author_text = combined_text
        .trim()
        .trim_start_matches('[') // Remove leading bracket
        .trim_start_matches('(') // Remove leading parenthesis
        .trim_start_matches(';') // Remove leading semicolon
        .trim_start_matches(',') // Remove leading comma
        .trim_end_matches(',') // Remove trailing comma
        .trim()
        .to_string();

    // If we have both author and year, combine them
    if author_text.is_empty() {
        year
    } else if year.is_empty() {
        author_text
    } else {
        format!("{author_text}, {year}")
    }
}

#[derive(Debug)]
enum CitationNode<'a> {
    Text(String),
    Link(&'a HTMLTag<'a>, String), // (tag, year_text)
}

/// Collect all text and link nodes from a citation in order
fn collect_citation_nodes<'a>(
    parser: &'a Parser,
    tag: &'a HTMLTag,
    nodes: &mut Vec<CitationNode<'a>>,
) {
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(text) = child.as_raw() {
            let text_content = text.try_as_utf8_str().unwrap_or_default().to_string();
            if !text_content.trim().is_empty() {
                nodes.push(CitationNode::Text(text_content));
            }
        } else if let Some(child_tag) = child.as_tag() {
            if child_tag.name().as_utf8_str() == "a"
                && child_tag
                    .attributes()
                    .class()
                    .map(|cls| cls.as_utf8_str())
                    .unwrap_or_default()
                    .contains("ltx_ref")
            {
                let link_text = get_text_content(parser, child_tag);
                nodes.push(CitationNode::Link(child_tag, link_text));
            } else {
                // Recursively collect from other tags
                collect_citation_nodes(parser, child_tag, nodes);
            }
        }
    }
}

/// Decode a <img> element into a [`ImageObject`] node
pub fn decode_img(_parser: &Parser, tag: &HTMLTag, context: &mut ArxivDecodeContext) -> Inline {
    let raw_url = get_attr(tag, "src").unwrap_or_default();
    let content_url = context.resolve_url(&raw_url);
    let title = get_attr(tag, "title").map(|title_text| vec![t(title_text)]);

    Inline::ImageObject(ImageObject {
        content_url,
        title,
        ..Default::default()
    })
}

/// Decode a simple inline "mark" element
pub fn decode_mark(
    parser: &Parser,
    tag: &HTMLTag,
    name: &str,
    context: &mut ArxivDecodeContext,
) -> Inline {
    let content = decode_inlines(parser, tag, context);
    match name {
        "em" | "i" => em(content),
        "strong" | "bold" => stg(content),
        "u" => u(content),
        "sup" => sup(content),
        "sub" => sub(content),
        "s" => stk(content),
        _ => em(content),
    }
}

/// Decode an inline math element to MathInline
pub fn decode_math_inline(
    parser: &Parser,
    tag: &HTMLTag,
    _context: &mut ArxivDecodeContext,
) -> Inline {
    let (latex_code, mathml_content) = extract_latex_and_mathml(parser, tag);

    let mut math_inline = MathInline {
        code: latex_code.into(),
        math_language: Some("latex".to_string()),
        ..Default::default()
    };

    // Set MathML in options if available
    if !mathml_content.is_empty() {
        math_inline.options.mathml = Some(mathml_content);
    }

    Inline::MathInline(math_inline)
}

/// Decode span elements based on LaTeXML semantic classes
///
/// LaTeXML uses spans extensively for semantic markup. Common patterns:
/// - Font styling: ltx_font_bold, ltx_font_italic, ltx_font_typewriter
/// - Structural: ltx_text, ltx_ref, ltx_note (pass-through content)
/// - Tags/labels: ltx_tag_* (special handling for different contexts)
/// - Errors: ltx_ERROR (ignored completely)
pub fn decode_span(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> Vec<Inline> {
    let class = tag
        .attributes()
        .class()
        .map(|cls| cls.as_utf8_str())
        .unwrap_or_default();

    // Skip error spans entirely - these indicate LaTeXML processing issues
    if class.contains("ltx_ERROR") {
        return Vec::new();
    }

    let content = decode_inlines(parser, tag, context);

    // Font styling spans - convert to appropriate Stencila formatting
    match class {
        c if c.contains("ltx_font_bold") => return vec![stg(content)],
        c if c.contains("ltx_font_italic") => return vec![em(content)],
        c if c.contains("ltx_font_typewriter") => {
            // Typewriter font usually indicates code
            return vec![Inline::CodeInline(CodeInline {
                code: extract_text_from_inlines(&content).into(),
                ..Default::default()
            })];
        }
        _ => {}
    }

    // Handle tag spans (labels, references, etc.)
    if class.contains("ltx_tag") {
        if class.contains("ltx_tag_item") {
            // List item tags like "(iii)" are decorative, ignore them
            return Vec::new();
        } else {
            // Other ltx_tag spans contain important labeling info
            return content;
        }
    }

    // Structural spans that should not be recorded as parsing losses
    if [
        "ltx_text",
        "ltx_ref",
        "ltx_note",
        "ltx_bibblock",
        "ltx_inline-enumerate",
        "ltx_inline-item",
        "ltx_transformed_inner",
    ]
    .iter()
    .any(|&pattern| class.contains(pattern))
    {
        return content;
    }

    // Unknown span type - record as parsing loss but preserve content
    context.add_loss(tag);
    content
}

/// Decode an SVG with class ltx_picture as an ImageObject with base64 data URI (inline level)
pub fn decode_svg_picture_inline(
    parser: &Parser,
    tag: &HTMLTag,
    _context: &mut ArxivDecodeContext,
) -> Inline {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // Get the full SVG element as HTML string
    let svg_html = tag.outer_html(parser);

    // Create a base64 encoded data URI
    let encoded = STANDARD.encode(svg_html.as_bytes());
    let data_uri = format!("data:image/svg+xml;base64,{encoded}");

    // Create an ImageObject
    let image_object = ImageObject {
        content_url: data_uri,
        ..Default::default()
    };

    Inline::ImageObject(image_object)
}
