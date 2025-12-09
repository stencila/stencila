//! Helper functions for handling references sections in articles

use stencila_schema::{
    Article, Block, Heading, Inline, Paragraph, Reference, Section, SectionType, Text,
};

/// Check if blocks contain a section with [`SectionType::References`]
pub fn has_references_section(blocks: &[Block]) -> bool {
    blocks.iter().any(|block| match block {
        Block::Section(section) => {
            matches!(section.section_type, Some(SectionType::References))
                || has_references_section(&section.content)
        }
        _ => false,
    })
}

/// Append a references section to an article's content
pub fn append_references_section(article: &mut Article, references: &[Reference]) {
    let references_section = Block::Section(Section {
        section_type: Some(SectionType::References),
        content: create_section_content(references),
        ..Default::default()
    });
    article.content.push(references_section);
}

/// Populate an existing [`SectionType::References`] section with references
///
/// Assumes the caller has already checked that a references section exists
/// using [`has_references_section`].
pub fn populate_references_section(article: &mut Article, references: &[Reference]) {
    populate_references_in_blocks(&mut article.content, references);
}

/// Recursively search for and populate a References section in blocks
fn populate_references_in_blocks(blocks: &mut [Block], references: &[Reference]) -> bool {
    for block in blocks.iter_mut() {
        if let Block::Section(section) = block {
            if matches!(section.section_type, Some(SectionType::References)) {
                section.content = create_section_content(references);
                return true;
            }
            // Recurse into nested sections
            if populate_references_in_blocks(&mut section.content, references) {
                return true;
            }
        }
    }
    false
}

/// Create section content with heading and reference paragraphs
fn create_section_content(references: &[Reference]) -> Vec<Block> {
    let mut content = vec![Block::Heading(Heading::new(
        1,
        vec![Inline::Text(Text::from("References"))],
    ))];
    content.extend(references_to_blocks(references));
    content
}

/// Convert references to blocks (paragraphs with pre-rendered content)
///
/// The content of references is rendered in the `node-execute` crate
/// when an article is compiled.
fn references_to_blocks(references: &[Reference]) -> Vec<Block> {
    references
        .iter()
        .filter_map(|reference| {
            // Use the pre-rendered content (compiled from citation style)
            reference
                .options
                .content
                .as_ref()
                .map(|content| Block::Paragraph(Paragraph::new(content.clone())))
        })
        .collect()
}
