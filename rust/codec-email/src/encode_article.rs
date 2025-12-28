use std::collections::BTreeMap;

use serde_json::Value;
use stencila_codec::{
    Losses,
    stencila_schema::{Article, Author, Reference},
};
use stencila_codec_text_trait::TextCodec;

use crate::{
    encode_blocks, encode_inlines,
    encode_theme::{encode_theme_attributes, encode_theme_styles},
    utils::html_escape,
};

/// Encode an Article to MJML
pub(super) fn encode_article(
    article: &Article,
    theme_vars: Option<&BTreeMap<String, Value>>,
) -> (String, Losses) {
    let mut mjml = String::with_capacity(10_000);
    let mut losses = Losses::none();

    // Extract title as text
    let title_text = article
        .title
        .as_ref()
        .map(|inlines| inlines.to_text())
        .unwrap_or_default();

    // Extract abstract preview (first 150 characters, safely handling multibyte UTF-8)
    let abstract_preview = article
        .r#abstract
        .as_ref()
        .map(|blocks| {
            let text = blocks.to_text();
            let char_count = text.chars().count();
            if char_count > 150 {
                let truncated: String = text.chars().take(147).collect();
                format!("{truncated}...")
            } else {
                text
            }
        })
        .unwrap_or_default();

    // Build MJML document
    mjml.push_str("<mjml>\n");
    mjml.push_str("  <mj-head>\n");

    // Title
    mjml.push_str(&format!(
        "    <mj-title>{}</mj-title>\n",
        html_escape(&title_text)
    ));

    // Preview text
    if !abstract_preview.is_empty() {
        mjml.push_str(&format!(
            "    <mj-preview>{}</mj-preview>\n",
            html_escape(&abstract_preview)
        ));
    }

    // Theme attributes
    mjml.push_str("    <mj-attributes>\n");
    encode_theme_attributes(&mut mjml, theme_vars);
    mjml.push_str("    </mj-attributes>\n");

    // Theme styles
    mjml.push_str("    <mj-style>\n");
    encode_theme_styles(&mut mjml, theme_vars);
    mjml.push_str("    </mj-style>\n");

    mjml.push_str("  </mj-head>\n");
    mjml.push_str("  <mj-body>\n");

    // Main content section
    mjml.push_str("    <mj-section>\n");
    mjml.push_str("      <mj-column>\n");

    // Title
    if let Some(title) = &article.title {
        mjml.push_str("        <mj-text>\n");
        mjml.push_str("          <h1>");
        encode_inlines(title, &mut mjml, &mut losses);
        mjml.push_str("</h1>\n");
        mjml.push_str("        </mj-text>\n");
    }

    // Authors
    if let Some(authors) = &article.authors {
        mjml.push_str("        <mj-text>\n");
        mjml.push_str("          <p class=\"authors\">");
        encode_authors(authors, &mut mjml);
        mjml.push_str("</p>\n");
        mjml.push_str("        </mj-text>\n");
    }

    // Abstract
    if let Some(r#abstract) = &article.r#abstract {
        mjml.push_str("        <mj-text>\n");
        mjml.push_str("          <div class=\"abstract\">\n");
        mjml.push_str("            <h2>Abstract</h2>\n");
        encode_blocks(r#abstract, &mut mjml, &mut losses);
        mjml.push_str("          </div>\n");
        mjml.push_str("        </mj-text>\n");
    }

    // Content
    encode_blocks(&article.content, &mut mjml, &mut losses);

    // References
    if let Some(references) = &article.references {
        mjml.push_str("        <mj-text>\n");
        mjml.push_str("          <h2>References</h2>\n");
        mjml.push_str("          <ol class=\"references\">\n");
        encode_references(references, &mut mjml, &mut losses);
        mjml.push_str("          </ol>\n");
        mjml.push_str("        </mj-text>\n");
    }

    mjml.push_str("      </mj-column>\n");
    mjml.push_str("    </mj-section>\n");
    mjml.push_str("  </mj-body>\n");
    mjml.push_str("</mjml>");

    (mjml, losses)
}

/// Encode authors
fn encode_authors(authors: &[Author], mjml: &mut String) {
    for (index, author) in authors.iter().enumerate() {
        if index > 0 {
            mjml.push_str(", ");
        }
        mjml.push_str(&author.name());
    }
}

/// Encode a reference
fn encode_references(references: &[Reference], mjml: &mut String, losses: &mut Losses) {
    for reference in references {
        if let Some(content) = &reference.options.content {
            mjml.push_str("            <li>");
            encode_inlines(content, mjml, losses);
            mjml.push_str("</li>\n");
        }
    }
}
