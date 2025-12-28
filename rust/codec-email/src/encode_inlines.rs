use stencila_codec::{
    Losses,
    stencila_schema::{
        Citation, CitationGroup, CodeExpression, ImageObject, Inline, Link, MathInline, Node,
    },
};
use stencila_codec_png::to_png_data_uri_with;
use stencila_codec_text_trait::TextCodec;
use stencila_images::ImageResizeOptions;

use crate::utils::{html_escape, process_image_url};

/// Encode a list of inlines to HTML (inside mj-text)
pub(super) fn encode_inlines(inlines: &[Inline], mjml: &mut String, losses: &mut Losses) {
    for inline in inlines {
        encode_inline(inline, mjml, losses);
    }
}

/// Encode a single inline to HTML
fn encode_inline(inline: &Inline, mjml: &mut String, losses: &mut Losses) {
    match inline {
        Inline::Text(text) => mjml.push_str(&html_escape(&text.value)),
        // Marks
        Inline::Emphasis(node) => encode_mark("em", &node.content, mjml, losses),
        Inline::QuoteInline(node) => encode_mark("q", &node.content, mjml, losses),
        Inline::Strong(node) => encode_mark("strong", &node.content, mjml, losses),
        Inline::Strikeout(node) => encode_mark("s", &node.content, mjml, losses),
        Inline::Subscript(node) => encode_mark("sub", &node.content, mjml, losses),
        Inline::Superscript(node) => encode_mark("sup", &node.content, mjml, losses),
        Inline::Underline(node) => encode_mark("u", &node.content, mjml, losses),
        // Link
        Inline::Link(link) => {
            encode_link(link, mjml, losses);
        }
        // Code
        Inline::CodeInline(code) => {
            mjml.push_str("<code>");
            mjml.push_str(&html_escape(&code.code));
            mjml.push_str("</code>");
        }
        Inline::CodeExpression(expr) => {
            encode_code_expression(expr, mjml);
        }
        // Math
        Inline::MathInline(math) => {
            encode_math_inline(math, mjml, losses);
        }
        // Media
        Inline::ImageObject(image) => {
            encode_inline_image(image, mjml);
        }
        // Citations
        Inline::Citation(citation) => {
            encode_citation(citation, mjml, losses);
        }
        Inline::CitationGroup(group) => {
            encode_citation_group(group, mjml, losses);
        }

        _ => {
            // Encode other inline types as plain text and record loss
            mjml.push_str(&inline.to_text());
            losses.add(format!("Inline::{}", inline.node_type()));
        }
    }
}

/// Encode a mark (inline with content wrapped in a tag)
fn encode_mark(tag: &str, content: &[Inline], html: &mut String, losses: &mut Losses) {
    html.push('<');
    html.push_str(tag);
    html.push('>');
    encode_inlines(content, html, losses);
    html.push_str("</");
    html.push_str(tag);
    html.push('>');
}

/// Encode a link
fn encode_link(link: &Link, mjml: &mut String, losses: &mut Losses) {
    mjml.push_str(&format!("<a href=\"{}\">", html_escape(&link.target)));
    encode_inlines(&link.content, mjml, losses);
    mjml.push_str("</a>");
}

/// Encode an inline image
///
/// Resizes data URI images for email (600px max width).
fn encode_inline_image(image: &ImageObject, mjml: &mut String) {
    let src = process_image_url(&image.content_url);
    let alt = image
        .caption
        .as_ref()
        .map(|c| c.to_text())
        .unwrap_or_default();
    mjml.push_str(&format!(
        "<img src=\"{}\" alt=\"{}\" style=\"max-width: 100%; height: auto;\"/>",
        html_escape(&src),
        html_escape(&alt)
    ));
}

/// Encode a code expression
fn encode_code_expression(expr: &CodeExpression, mjml: &mut String) {
    if let Some(output) = &expr.output {
        mjml.push_str(&output.to_text());
    } else {
        mjml.push_str(&expr.code);
    }
}

/// Encode inline math
///
/// Renders the math to a PNG image since email clients don't support MathML.
/// The image is resized for email (max 600px width).
fn encode_math_inline(math: &MathInline, mjml: &mut String, losses: &mut Losses) {
    let options = ImageResizeOptions::for_email();

    // Render math to PNG data URI with email-optimized sizing
    match to_png_data_uri_with(&Node::MathInline(math.clone()), &options) {
        Ok(data_uri) => {
            mjml.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\" style=\"vertical-align: middle; height: 1em;\"/>",
                data_uri,
                html_escape(&math.code)
            ));
        }
        Err(e) => {
            // Fallback to showing the math code
            tracing::warn!("Failed to render MathInline to PNG: {}", e);
            mjml.push_str(&format!("<code>{}</code>", html_escape(&math.code)));
            losses.add(format!("MathInline render error: {e}"));
        }
    }
}

/// Encode a citation
///
/// Uses the rendered content if available, otherwise falls back to the target.
fn encode_citation(citation: &Citation, html: &mut String, losses: &mut Losses) {
    if let Some(content) = &citation.options.content {
        encode_inlines(content, html, losses);
    } else {
        // Fallback to target in parentheses
        html.push('(');
        html.push_str(&html_escape(&citation.target));
        html.push(')');
    }
}

/// Encode a citation group
///
/// Uses the rendered content if available, otherwise encodes each citation.
fn encode_citation_group(group: &CitationGroup, html: &mut String, losses: &mut Losses) {
    if let Some(content) = &group.content {
        encode_inlines(content, html, losses);
    } else {
        // Fallback to encoding each citation
        html.push('(');
        for (index, citation) in group.items.iter().enumerate() {
            if index > 0 {
                html.push_str("; ");
            }
            if let Some(content) = &citation.options.content {
                encode_inlines(content, html, losses);
            } else {
                html.push_str(&html_escape(&citation.target));
            }
        }
        html.push(')');
    }
}
