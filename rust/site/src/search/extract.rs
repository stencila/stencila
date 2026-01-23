//! Content extraction for search indexing
//!
//! This module walks through document nodes and extracts searchable content
//! with appropriate weights based on the node type.

use std::collections::HashSet;

use stencila_codec::stencila_schema::{
    Admonition, Article, Block, CodeChunk, Datatable, DatatableColumn, Figure, Heading, List,
    ListItem, Node, Paragraph, QuoteBlock, Section, StyledBlock, Table,
};
use stencila_codec_text::to_text;
use stencila_config::SearchConfig;

use super::entry::{DatatableMetadata, SearchEntry, weights};

/// Maximum text length to index per entry (Unicode code points)
const MAX_TEXT_LENGTH: usize = 500;

/// Extract searchable entries from a document node
///
/// Walks through the document tree and creates search entries for
/// indexable content like headings, paragraphs, datatables, etc.
pub fn extract_entries(node: &Node, route: &str) -> Vec<SearchEntry> {
    let mut entries = Vec::new();
    extract_from_node(node, route, 0, MAX_TEXT_LENGTH, &None, &mut entries);
    entries
}

/// Extract searchable entries from a document node with configuration
///
/// Same as `extract_entries` but respects the search configuration for
/// filtering node types and text truncation.
pub fn extract_entries_with_config(
    node: &Node,
    route: &str,
    config: &SearchConfig,
) -> Vec<SearchEntry> {
    let mut entries = Vec::new();

    // Convert include_types to a HashSet for efficient lookup
    let include_types: HashSet<String> = config.include_types().into_iter().collect();

    extract_from_node(
        node,
        route,
        0,
        config.max_text_length(),
        &Some(include_types),
        &mut entries,
    );
    entries
}

/// Type alias for the optional include types filter
type IncludeTypes = Option<HashSet<String>>;

/// Check if a node type should be included based on the filter
fn should_include(node_type: &str, include_types: &IncludeTypes) -> bool {
    match include_types {
        Some(types) => types.contains(node_type),
        None => true, // No filter means include all
    }
}

/// Recursively extract entries from a node
fn extract_from_node(
    node: &Node,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    match node {
        Node::Article(article) => extract_from_article(
            article,
            route,
            depth,
            max_text_length,
            include_types,
            entries,
        ),
        Node::Section(section) => extract_from_section(
            section,
            route,
            depth,
            max_text_length,
            include_types,
            entries,
        ),
        Node::Heading(heading) if should_include("Heading", include_types) => {
            extract_from_heading(heading, route, depth, max_text_length, entries)
        }
        Node::Paragraph(paragraph) if should_include("Paragraph", include_types) => {
            extract_from_paragraph(paragraph, route, depth, max_text_length, entries)
        }
        Node::CodeChunk(code_chunk) if should_include("CodeChunk", include_types) => {
            extract_from_code_chunk(code_chunk, route, depth, max_text_length, entries)
        }
        Node::Datatable(datatable) if should_include("Datatable", include_types) => {
            extract_from_datatable(datatable, route, depth, max_text_length, entries)
        }
        Node::Figure(figure) if should_include("Figure", include_types) => {
            extract_from_figure(figure, route, depth, max_text_length, entries)
        }
        Node::Table(table) if should_include("Table", include_types) => {
            extract_from_table(table, route, depth, max_text_length, entries)
        }
        // Container nodes - always recurse into their content
        Node::QuoteBlock(quote) => {
            extract_from_quote_block(quote, route, depth, max_text_length, include_types, entries)
        }
        Node::List(list) => {
            extract_from_list(list, route, depth, max_text_length, include_types, entries)
        }
        Node::Admonition(admonition) => extract_from_admonition(
            admonition,
            route,
            depth,
            max_text_length,
            include_types,
            entries,
        ),
        Node::StyledBlock(styled) => extract_from_styled_block(
            styled,
            route,
            depth,
            max_text_length,
            include_types,
            entries,
        ),
        // Other node types are not directly indexed (e.g., primitives, inlines)
        _ => {}
    }
}

/// Extract entries from an Article
fn extract_from_article(
    article: &Article,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    // Index the title if Article type is included
    if should_include("Article", include_types)
        && let Some(title) = &article.title
    {
        let text = truncate_text_with_limit(&to_text(title), max_text_length);
        if !text.is_empty() {
            entries.push(SearchEntry::new(
                article.node_id().to_string(),
                "Article",
                route.to_string(),
                text,
                weights::TITLE,
                depth,
            ));
        }
    }

    // Recurse into content
    for block in &article.content {
        extract_from_block(
            block,
            route,
            depth + 1,
            max_text_length,
            include_types,
            entries,
        );
    }
}

/// Extract entries from a Section
fn extract_from_section(
    section: &Section,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    for block in &section.content {
        extract_from_block(
            block,
            route,
            depth + 1,
            max_text_length,
            include_types,
            entries,
        );
    }
}

/// Extract entries from a Block
fn extract_from_block(
    block: &Block,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    match block {
        Block::Section(section) => extract_from_section(
            section,
            route,
            depth,
            max_text_length,
            include_types,
            entries,
        ),
        Block::Heading(heading) if should_include("Heading", include_types) => {
            extract_from_heading(heading, route, depth, max_text_length, entries)
        }
        Block::Paragraph(paragraph) if should_include("Paragraph", include_types) => {
            extract_from_paragraph(paragraph, route, depth, max_text_length, entries)
        }
        Block::CodeChunk(code_chunk) if should_include("CodeChunk", include_types) => {
            extract_from_code_chunk(code_chunk, route, depth, max_text_length, entries)
        }
        Block::Datatable(datatable) if should_include("Datatable", include_types) => {
            extract_from_datatable(datatable, route, depth, max_text_length, entries)
        }
        Block::Figure(figure) if should_include("Figure", include_types) => {
            extract_from_figure(figure, route, depth, max_text_length, entries)
        }
        Block::Table(table) if should_include("Table", include_types) => {
            extract_from_table(table, route, depth, max_text_length, entries)
        }
        // Container blocks - always recurse into their content
        Block::QuoteBlock(quote) => {
            extract_from_quote_block(quote, route, depth, max_text_length, include_types, entries)
        }
        Block::List(list) => {
            extract_from_list(list, route, depth, max_text_length, include_types, entries)
        }
        Block::Admonition(admonition) => extract_from_admonition(
            admonition,
            route,
            depth,
            max_text_length,
            include_types,
            entries,
        ),
        Block::StyledBlock(styled) => extract_from_styled_block(
            styled,
            route,
            depth,
            max_text_length,
            include_types,
            entries,
        ),
        // Other block types are not indexed (e.g., ThematicBreak, RawBlock)
        _ => {}
    }
}

/// Extract entries from a Heading
fn extract_from_heading(
    heading: &Heading,
    route: &str,
    depth: u8,
    max_text_length: usize,
    entries: &mut Vec<SearchEntry>,
) {
    let text = truncate_text_with_limit(&to_text(&heading.content), max_text_length);
    if text.is_empty() {
        return;
    }

    // Weight based on heading level (1-6)
    let level = heading.level.clamp(1, 6) as u8;
    let weight = match level {
        1 => weights::HEADING_1,
        2 => weights::HEADING_2,
        3 => weights::HEADING_3,
        4 => weights::HEADING_4,
        5 => weights::HEADING_5,
        _ => weights::HEADING_6,
    };

    entries.push(SearchEntry::new(
        heading.node_id().to_string(),
        "Heading",
        route.to_string(),
        text,
        weight,
        depth,
    ));
}

/// Extract entries from a Paragraph
fn extract_from_paragraph(
    paragraph: &Paragraph,
    route: &str,
    depth: u8,
    max_text_length: usize,
    entries: &mut Vec<SearchEntry>,
) {
    let text = truncate_text_with_limit(&to_text(&paragraph.content), max_text_length);
    if text.is_empty() {
        return;
    }

    entries.push(SearchEntry::new(
        paragraph.node_id().to_string(),
        "Paragraph",
        route.to_string(),
        text,
        weights::PARAGRAPH,
        depth,
    ));
}

/// Extract entries from a CodeChunk
fn extract_from_code_chunk(
    code_chunk: &CodeChunk,
    route: &str,
    depth: u8,
    max_text_length: usize,
    entries: &mut Vec<SearchEntry>,
) {
    let text = truncate_text_with_limit(&code_chunk.code.to_string(), max_text_length);
    if text.is_empty() {
        return;
    }

    entries.push(SearchEntry::new(
        code_chunk.node_id().to_string(),
        "CodeChunk",
        route.to_string(),
        text,
        weights::CODE,
        depth,
    ));
}

/// Extract entries from a Datatable
fn extract_from_datatable(
    datatable: &Datatable,
    route: &str,
    depth: u8,
    max_text_length: usize,
    entries: &mut Vec<SearchEntry>,
) {
    // Index column names
    let columns: Vec<String> = datatable.columns.iter().map(column_name).collect();

    if columns.is_empty() {
        return;
    }

    let text = truncate_text_with_limit(&columns.join(" "), max_text_length);
    // Use max of all column lengths to handle potentially ragged columns
    let row_count = datatable
        .columns
        .iter()
        .map(column_value_count)
        .max()
        .unwrap_or(0);

    let metadata = DatatableMetadata {
        columns: columns.clone(),
        description: datatable.options.name.clone().map(|s| s.to_string()),
        row_count: if row_count > 0 { Some(row_count) } else { None },
    };

    entries.push(
        SearchEntry::new(
            datatable.node_id().to_string(),
            "Datatable",
            route.to_string(),
            text,
            weights::DATATABLE,
            depth,
        )
        .with_metadata(metadata),
    );
}

/// Get the name from a DatatableColumn
fn column_name(column: &DatatableColumn) -> String {
    column.name.to_string()
}

/// Get the number of values in a DatatableColumn
fn column_value_count(column: &DatatableColumn) -> usize {
    column.values.len()
}

/// Extract entries from a Figure (index the caption)
fn extract_from_figure(
    figure: &Figure,
    route: &str,
    depth: u8,
    max_text_length: usize,
    entries: &mut Vec<SearchEntry>,
) {
    if let Some(caption) = &figure.caption {
        let text = truncate_text_with_limit(&to_text(caption), max_text_length);
        if !text.is_empty() {
            entries.push(SearchEntry::new(
                figure.node_id().to_string(),
                "Figure",
                route.to_string(),
                text,
                weights::CAPTION,
                depth,
            ));
        }
    }
}

/// Extract entries from a Table (index the caption)
fn extract_from_table(
    table: &Table,
    route: &str,
    depth: u8,
    max_text_length: usize,
    entries: &mut Vec<SearchEntry>,
) {
    if let Some(caption) = &table.caption {
        let text = truncate_text_with_limit(&to_text(caption), max_text_length);
        if !text.is_empty() {
            entries.push(SearchEntry::new(
                table.node_id().to_string(),
                "Table",
                route.to_string(),
                text,
                weights::CAPTION,
                depth,
            ));
        }
    }
}

/// Extract entries from a QuoteBlock (recurse into content)
fn extract_from_quote_block(
    quote: &QuoteBlock,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    for block in &quote.content {
        extract_from_block(
            block,
            route,
            depth + 1,
            max_text_length,
            include_types,
            entries,
        );
    }
}

/// Extract entries from a List (recurse into list items)
fn extract_from_list(
    list: &List,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    for item in &list.items {
        extract_from_list_item(item, route, depth, max_text_length, include_types, entries);
    }
}

/// Extract entries from a ListItem (recurse into content)
fn extract_from_list_item(
    item: &ListItem,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    for block in &item.content {
        extract_from_block(
            block,
            route,
            depth + 1,
            max_text_length,
            include_types,
            entries,
        );
    }
}

/// Extract entries from an Admonition (recurse into content)
fn extract_from_admonition(
    admonition: &Admonition,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    for block in &admonition.content {
        extract_from_block(
            block,
            route,
            depth + 1,
            max_text_length,
            include_types,
            entries,
        );
    }
}

/// Extract entries from a StyledBlock (recurse into content)
fn extract_from_styled_block(
    styled: &StyledBlock,
    route: &str,
    depth: u8,
    max_text_length: usize,
    include_types: &IncludeTypes,
    entries: &mut Vec<SearchEntry>,
) {
    for block in &styled.content {
        extract_from_block(
            block,
            route,
            depth + 1,
            max_text_length,
            include_types,
            entries,
        );
    }
}

/// Truncate text to a specific limit (Unicode code points), preserving word boundaries
fn truncate_text_with_limit(text: &str, max_length: usize) -> String {
    let text = text.trim();

    // Count Unicode code points, not bytes
    let char_count = text.chars().count();
    if char_count <= max_length {
        return text.to_string();
    }

    // Find the byte index corresponding to max_length characters
    let byte_index = text
        .char_indices()
        .nth(max_length)
        .map(|(i, _)| i)
        .unwrap_or(text.len());

    // Find a word boundary (space) near the limit
    let truncated = &text[..byte_index];
    if let Some(last_space) = truncated.rfind(' ') {
        truncated[..last_space].to_string()
    } else {
        truncated.to_string()
    }
}

#[cfg(test)]
mod tests {
    use stencila_codec::stencila_schema::{Cord, Inline, Primitive, QuoteBlock, Text};

    use super::*;

    #[test]
    fn test_truncate_text() {
        // Short text unchanged
        assert_eq!(
            truncate_text_with_limit("hello world", MAX_TEXT_LENGTH),
            "hello world"
        );

        // Whitespace trimmed
        assert_eq!(
            truncate_text_with_limit("  hello  ", MAX_TEXT_LENGTH),
            "hello"
        );

        // Long text truncated at word boundary
        let long_text = "a ".repeat(300);
        let truncated = truncate_text_with_limit(&long_text, MAX_TEXT_LENGTH);
        assert!(truncated.chars().count() <= MAX_TEXT_LENGTH);
        assert!(!truncated.ends_with(' '));
    }

    #[test]
    fn test_truncate_text_multibyte() {
        // Test with multi-byte UTF-8 characters (Chinese characters are 3 bytes each)
        // Create text with 600 Chinese characters (1800 bytes)
        let chinese_text = "中".repeat(600);
        let truncated = truncate_text_with_limit(&chinese_text, MAX_TEXT_LENGTH);

        // Should truncate at character boundary, not byte boundary
        assert_eq!(truncated.chars().count(), MAX_TEXT_LENGTH);
        // Should not panic and result should be valid UTF-8
        assert!(truncated.is_char_boundary(truncated.len()));

        // Test with mixed content
        let mixed = format!("{} {}", "café".repeat(100), "日本語".repeat(100));
        let truncated_mixed = truncate_text_with_limit(&mixed, MAX_TEXT_LENGTH);
        assert!(truncated_mixed.chars().count() <= MAX_TEXT_LENGTH);
    }

    /// Helper to create a simple text inline
    fn text_inline(s: &str) -> Inline {
        Inline::Text(Text {
            value: Cord::from(s),
            ..Default::default()
        })
    }

    /// Helper to create a paragraph with text
    fn paragraph(s: &str) -> Paragraph {
        Paragraph {
            content: vec![text_inline(s)],
            ..Default::default()
        }
    }

    /// Helper to create a heading with text at a given level
    fn heading(s: &str, level: i64) -> Heading {
        Heading {
            level,
            content: vec![text_inline(s)],
            ..Default::default()
        }
    }

    #[test]
    fn test_extract_article_title() {
        let article = Article {
            title: Some(vec![text_inline("My Article Title")]),
            content: vec![Block::Paragraph(paragraph("Some content"))],
            ..Default::default()
        };

        let entries = extract_entries(&Node::Article(article), "/test");

        // Should have 2 entries: title and paragraph
        assert_eq!(entries.len(), 2);

        // First entry should be the article title
        assert_eq!(entries[0].node_type, "Article");
        assert_eq!(entries[0].text, "My Article Title");
        assert_eq!(entries[0].weight, weights::TITLE);
        assert_eq!(entries[0].depth, 0);

        // Second entry should be the paragraph
        assert_eq!(entries[1].node_type, "Paragraph");
        assert_eq!(entries[1].text, "Some content");
        assert_eq!(entries[1].depth, 1);
    }

    #[test]
    fn test_extract_with_include_types_filter() {
        let article = Article {
            title: Some(vec![text_inline("Title")]),
            content: vec![
                Block::Heading(heading("Heading 1", 1)),
                Block::Paragraph(paragraph("Para 1")),
                Block::Heading(heading("Heading 2", 2)),
                Block::Paragraph(paragraph("Para 2")),
            ],
            ..Default::default()
        };

        // Filter to only include Headings
        let config = SearchConfig {
            enabled: Some(true),
            include_types: Some(vec!["Heading".to_string()]),
            ..Default::default()
        };

        let entries = extract_entries_with_config(&Node::Article(article), "/test", &config);

        // Should only have headings (no Article title, no Paragraphs)
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.node_type == "Heading"));
        assert_eq!(entries[0].text, "Heading 1");
        assert_eq!(entries[1].text, "Heading 2");
    }

    #[test]
    fn test_extract_respects_max_text_length() {
        let long_text = "word ".repeat(200); // 1000 chars
        let article = Article {
            content: vec![Block::Paragraph(paragraph(&long_text))],
            ..Default::default()
        };

        let config = SearchConfig {
            enabled: Some(true),
            max_text_length: Some(50),
            ..Default::default()
        };

        let entries = extract_entries_with_config(&Node::Article(article), "/test", &config);

        assert_eq!(entries.len(), 1);
        // Text should be truncated to around 50 chars (at word boundary)
        assert!(entries[0].text.chars().count() <= 50);
    }

    #[test]
    fn test_datatable_respects_max_text_length() {
        // Create a datatable with many long column names
        let columns: Vec<DatatableColumn> = (0..20)
            .map(|i| DatatableColumn {
                name: format!("very_long_column_name_number_{i}"),
                values: vec![Primitive::Integer(1)],
                ..Default::default()
            })
            .collect();

        let datatable = Datatable {
            columns,
            ..Default::default()
        };

        let config = SearchConfig {
            enabled: Some(true),
            max_text_length: Some(100),
            ..Default::default()
        };

        let entries = extract_entries_with_config(&Node::Datatable(datatable), "/test", &config);

        assert_eq!(entries.len(), 1);
        // Text should be truncated to around 100 chars
        assert!(
            entries[0].text.chars().count() <= 100,
            "Datatable text should be truncated, got {} chars",
            entries[0].text.chars().count()
        );
    }

    #[test]
    fn test_depth_handling_nested_structures() {
        // Create a nested structure: Article > QuoteBlock > Paragraph
        let quote = QuoteBlock {
            content: vec![Block::Paragraph(paragraph("Quoted text"))],
            ..Default::default()
        };

        let article = Article {
            content: vec![
                Block::Paragraph(paragraph("Top level")),
                Block::QuoteBlock(quote),
            ],
            ..Default::default()
        };

        let entries = extract_entries(&Node::Article(article), "/test");

        // Find the paragraphs
        let top_level = entries.iter().find(|e| e.text == "Top level");
        let quoted = entries.iter().find(|e| e.text == "Quoted text");

        // Top level paragraph should be at depth 1 (inside Article)
        assert_eq!(top_level.expect("top level paragraph not found").depth, 1);
        // Quoted paragraph should be at depth 2 (inside Article > QuoteBlock)
        assert_eq!(quoted.expect("quoted paragraph not found").depth, 2);
    }

    #[test]
    fn test_depth_handling_list_items() {
        // Create a list with paragraphs inside list items
        let list = List {
            items: vec![ListItem {
                content: vec![Block::Paragraph(paragraph("List item text"))],
                ..Default::default()
            }],
            ..Default::default()
        };

        let article = Article {
            content: vec![Block::Paragraph(paragraph("Top level")), Block::List(list)],
            ..Default::default()
        };

        let entries = extract_entries(&Node::Article(article), "/test");

        let top_level = entries.iter().find(|e| e.text == "Top level");
        let list_para = entries.iter().find(|e| e.text == "List item text");

        // Top level paragraph: depth 1 (inside Article)
        assert_eq!(top_level.expect("top level paragraph not found").depth, 1);
        // List item paragraph: depth 2 (inside Article > List > ListItem)
        // This should now be consistent with QuoteBlock nesting
        assert_eq!(list_para.expect("list item paragraph not found").depth, 2);
    }

    #[test]
    fn test_heading_weights_by_level() {
        let article = Article {
            content: vec![
                Block::Heading(heading("H1", 1)),
                Block::Heading(heading("H2", 2)),
                Block::Heading(heading("H3", 3)),
                Block::Heading(heading("H6", 6)),
            ],
            ..Default::default()
        };

        let entries = extract_entries(&Node::Article(article), "/test");

        let h1 = entries
            .iter()
            .find(|e| e.text == "H1")
            .expect("H1 not found");
        let h2 = entries
            .iter()
            .find(|e| e.text == "H2")
            .expect("H2 not found");
        let h3 = entries
            .iter()
            .find(|e| e.text == "H3")
            .expect("H3 not found");
        let h6 = entries
            .iter()
            .find(|e| e.text == "H6")
            .expect("H6 not found");

        // Higher level headings should have higher weights
        assert!(h1.weight > h2.weight);
        assert!(h2.weight > h3.weight);
        assert!(h3.weight > h6.weight);
    }

    #[test]
    fn test_empty_content_not_indexed() {
        let article = Article {
            title: Some(vec![text_inline("   ")]), // whitespace only
            content: vec![Block::Paragraph(paragraph(""))], // empty paragraph
            ..Default::default()
        };

        let entries = extract_entries(&Node::Article(article), "/test");

        // Neither empty title nor empty paragraph should be indexed
        assert!(entries.is_empty());
    }
}
