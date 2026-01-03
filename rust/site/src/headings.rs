//! Headings extraction for right sidebar table of contents
//!
//! Extracts document headings from Article nodes and converts them
//! to the `HeadingItem` structure used by the layout system.

use stencila_codec::stencila_schema::{Block, Inline, List, Node};
use stencila_codec_dom::HeadingItem;
use stencila_config::RightSidebarConfig;

/// Extract headings from a document node
///
/// If the node is an Article with headings, extracts them as `HeadingItem`s
/// with optional depth filtering based on the right sidebar configuration.
///
/// # Arguments
///
/// * `node` - The document node (typically an Article)
/// * `config` - Optional right sidebar configuration for depth filtering
///
/// # Returns
///
/// Returns `Some(Vec<HeadingItem>)` if the node is an Article with headings,
/// `None` otherwise.
pub fn extract_headings_from_node(
    node: &Node,
    config: Option<&RightSidebarConfig>,
) -> Option<Vec<HeadingItem>> {
    let Node::Article(article) = node else {
        return None;
    };

    let headings_list = article.options.headings.as_ref()?;
    let max_depth = config.and_then(|c| c.depth).unwrap_or(3);

    let headings = list_to_heading_items(headings_list, 1, max_depth);

    if headings.is_empty() {
        None
    } else {
        Some(headings)
    }
}

/// Convert a List of heading items to HeadingItems
///
/// Recursively processes the heading list structure, respecting depth limits.
fn list_to_heading_items(list: &List, current_depth: u8, max_depth: u8) -> Vec<HeadingItem> {
    if current_depth > max_depth {
        return Vec::new();
    }

    list.items
        .iter()
        .filter_map(|item| list_item_to_heading_item(item, current_depth, max_depth))
        .collect()
}

/// Convert a single ListItem to a HeadingItem
fn list_item_to_heading_item(
    item: &stencila_codec::stencila_schema::ListItem,
    current_depth: u8,
    max_depth: u8,
) -> Option<HeadingItem> {
    // The first block should be a Paragraph containing a Link
    let first_block = item.content.first()?;
    let Block::Paragraph(paragraph) = first_block else {
        return None;
    };

    // Find the Link in the paragraph content
    let link = paragraph.content.iter().find_map(|inline| {
        if let Inline::Link(link) = inline {
            Some(link)
        } else {
            None
        }
    })?;

    // Extract the heading ID from the link target (e.g., "#hdg-abc123" -> "hdg-abc123")
    let id = link.target.strip_prefix('#').unwrap_or(&link.target);

    // Extract text from the link content
    let text = inlines_to_text(&link.content);

    // Look for nested children (a nested List in the content)
    let children = item
        .content
        .iter()
        .find_map(|block| {
            if let Block::List(nested_list) = block {
                Some(list_to_heading_items(
                    nested_list,
                    current_depth + 1,
                    max_depth,
                ))
            } else {
                None
            }
        })
        .unwrap_or_default();

    Some(HeadingItem {
        id: id.to_string(),
        text,
        level: current_depth,
        children,
    })
}

/// Convert inline content to plain text
fn inlines_to_text(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(inline_to_text)
        .collect::<Vec<_>>()
        .join("")
}

/// Convert a single inline to text
fn inline_to_text(inline: &Inline) -> String {
    match inline {
        Inline::Text(text) => text.value.to_string(),
        Inline::Emphasis(emphasis) => inlines_to_text(&emphasis.content),
        Inline::Strong(strong) => inlines_to_text(&strong.content),
        Inline::Subscript(subscript) => inlines_to_text(&subscript.content),
        Inline::Superscript(superscript) => inlines_to_text(&superscript.content),
        Inline::Strikeout(strikeout) => inlines_to_text(&strikeout.content),
        Inline::Underline(underline) => inlines_to_text(&underline.content),
        Inline::QuoteInline(quote) => inlines_to_text(&quote.content),
        Inline::Link(link) => inlines_to_text(&link.content),
        Inline::CodeExpression(code_expr) => code_expr
            .output
            .as_ref()
            .map(|boxed| node_to_text(boxed.as_ref()))
            .unwrap_or_default(),
        Inline::CodeInline(code) => code.code.to_string(),
        Inline::MathInline(math) => math.code.to_string(),
        // For other inline types, return empty or a placeholder
        _ => String::new(),
    }
}

/// Convert a node to text (for code expression outputs)
fn node_to_text(node: &Node) -> String {
    match node {
        Node::Text(text) => text.value.to_string(),
        Node::String(s) => s.to_string(),
        Node::Integer(i) => i.to_string(),
        Node::Number(n) => n.to_string(),
        Node::Boolean(b) => b.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_codec::stencila_schema::{
        Article, ArticleOptions, Link, ListItem, ListOrder, Paragraph, Text,
    };

    /// Create a simple heading list item with a link
    fn make_heading_item(id: &str, text: &str, children: Option<List>) -> ListItem {
        let mut content = vec![Block::Paragraph(Paragraph::new(vec![Inline::Link(
            Link::new(vec![Inline::Text(Text::from(text))], format!("#{id}")),
        )]))];

        if let Some(nested) = children {
            content.push(Block::List(nested));
        }

        ListItem::new(content)
    }

    #[test]
    fn test_extract_headings_from_article() {
        // Create a simple article with headings
        let headings = List::new(
            vec![
                make_heading_item("hdg-1", "Introduction", None),
                make_heading_item(
                    "hdg-2",
                    "Methods",
                    Some(List::new(
                        vec![
                            make_heading_item("hdg-2-1", "Data Collection", None),
                            make_heading_item("hdg-2-2", "Analysis", None),
                        ],
                        ListOrder::Ascending,
                    )),
                ),
                make_heading_item("hdg-3", "Results", None),
            ],
            ListOrder::Ascending,
        );

        let article = Article {
            options: Box::new(ArticleOptions {
                headings: Some(headings),
                ..Default::default()
            }),
            ..Default::default()
        };

        let node = Node::Article(article);
        let result = extract_headings_from_node(&node, None);

        assert!(result.is_some());
        let items = result.unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].text, "Introduction");
        assert_eq!(items[0].id, "hdg-1");
        assert_eq!(items[0].level, 1);
        assert!(items[0].children.is_empty());

        assert_eq!(items[1].text, "Methods");
        assert_eq!(items[1].children.len(), 2);
        assert_eq!(items[1].children[0].text, "Data Collection");
        assert_eq!(items[1].children[0].level, 2);
    }

    #[test]
    fn test_depth_filtering() {
        let headings = List::new(
            vec![make_heading_item(
                "hdg-1",
                "Top Level",
                Some(List::new(
                    vec![make_heading_item(
                        "hdg-1-1",
                        "Second Level",
                        Some(List::new(
                            vec![make_heading_item("hdg-1-1-1", "Third Level", None)],
                            ListOrder::Ascending,
                        )),
                    )],
                    ListOrder::Ascending,
                )),
            )],
            ListOrder::Ascending,
        );

        let article = Article {
            options: Box::new(ArticleOptions {
                headings: Some(headings),
                ..Default::default()
            }),
            ..Default::default()
        };

        let node = Node::Article(article);

        // With max depth 2, should only have 2 levels
        let config = RightSidebarConfig {
            depth: Some(2),
            ..Default::default()
        };
        let result = extract_headings_from_node(&node, Some(&config));

        assert!(result.is_some());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].children.len(), 1);
        // Third level should be filtered out
        assert!(items[0].children[0].children.is_empty());
    }

    #[test]
    fn test_non_article_node() {
        let node = Node::Integer(42);
        let result = extract_headings_from_node(&node, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_article_without_headings() {
        let article = Article::default();
        let node = Node::Article(article);
        let result = extract_headings_from_node(&node, None);
        assert!(result.is_none());
    }
}
