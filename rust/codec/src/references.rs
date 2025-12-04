//! Helper functions for injecting references sections into articles

use stencila_schema::{
    Article, Block, Heading, Inline, Node, Paragraph, Reference, Section, SectionType, Text,
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

/// Convert references to blocks (paragraphs with pre-rendered content)
pub fn references_to_blocks(references: &[Reference]) -> Vec<Block> {
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

/// Inject a references section into an [Article] if needed
///
/// Returns a new Node with references section appended if:
/// - The node is an [Article]
/// - The article has non-empty references
/// - The article content doesn't already have a SectionType::References section
///
/// Otherwise returns a clone of the original node.
pub fn inject_references_section(node: &Node) -> Node {
    if let Node::Article(article) = node
        && let Some(references) = &article.references
        && !references.is_empty()
        && !has_references_section(&article.content)
    {
        // Create references section with heading and reference paragraphs
        let mut section_content = vec![Block::Heading(Heading::new(
            1,
            vec![Inline::Text(Text::from("References"))],
        ))];
        section_content.extend(references_to_blocks(references));

        let references_section = Block::Section(Section {
            section_type: Some(SectionType::References),
            content: section_content,
            ..Default::default()
        });

        // Clone article with appended references section
        let mut new_content = article.content.clone();
        new_content.push(references_section);

        Node::Article(Article {
            content: new_content,
            ..article.clone()
        })
    } else {
        node.clone()
    }
}
