//! Stabilize node UIDs for deterministic site rendering
//!
//! This crate provides functionality to replace random node UIDs with deterministic
//! path-based UIDs. This ensures that the same source document produces identical
//! HTML and nodemap.json output on re-render, enabling effective ETag-based caching.

use std::collections::HashMap;

use stencila_codec_text::to_text;
use stencila_node_id::NodeUid;
use stencila_schema::{
    Block, Citation, Heading, IfBlockClause, Inline, ListItem, Node, NodePath, NodeProperty,
    NodeSlot, SuggestionBlock, SuggestionInline, TableCell, TableRow, VisitorMut, WalkControl,
    WalkNode, WalkthroughStep,
};

/// Stabilize all node UIDs in a document tree.
///
/// Replaces random UIDs with deterministic path-based UIDs, ensuring that
/// the same source document produces identical rendered output.
///
/// This should be called before encoding to HTML for site rendering.
///
/// # Example
///
/// ```ignore
/// use stencila_node_stabilize::stabilize;
///
/// let mut node = /* load document */;
/// stabilize(&mut node);
/// // Now render to HTML - UIDs will be deterministic
/// ```
pub fn stabilize(node: &mut Node) {
    let mut stabilizer = Stabilizer::new();
    node.walk_mut(&mut stabilizer);
}

/// A visitor that stabilizes node UIDs based on their path in the document tree.
///
/// Each node's UID is replaced with a deterministic value derived from its
/// position (properties and indices) in the document tree.
/// Headings get special treatment with content-based slugs.
#[derive(Default)]
pub struct Stabilizer {
    /// Current path including both properties and indices
    path: NodePath,
    /// Tracks heading slugs for duplicate detection
    heading_slugs: HashMap<String, usize>,
}

impl Stabilizer {
    /// Create a new stabilizer
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the current path as a deterministic UID
    ///
    /// Encodes the path as hyphen-separated property names and indices.
    /// Example: "content-0-rows-1" for content[0].rows[1]
    fn uid_from_path(&self) -> NodeUid {
        let encoded = self
            .path
            .iter()
            .map(|slot| match slot {
                NodeSlot::Property(prop) => prop.to_string(),
                NodeSlot::Index(idx) => idx.to_string(),
            })
            .collect::<Vec<_>>()
            .join("-");
        NodeUid::from(encoded.into_bytes())
    }

    /// Generate a unique slug for a heading based on its content
    ///
    /// Converts heading text to a URL-friendly slug and handles duplicates
    /// by appending -2, -3, etc.
    fn heading_slug(&mut self, heading: &Heading) -> NodeUid {
        // Extract plain text from heading content
        let text = to_text(&heading.content);

        // Convert to slug: lowercase, replace non-alphanumeric with hyphens
        // Use is_alphanumeric() to support Unicode letters/numbers (e.g., 日本語)
        let slug: String = text
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();

        // Collapse consecutive hyphens and trim
        let mut slug = slug
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        // Handle empty slugs
        if slug.is_empty() {
            slug = "heading".to_string();
        }

        // Handle duplicates
        let count = self.heading_slugs.entry(slug.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            slug = format!("{}-{}", slug, count);
        }

        NodeUid::from(slug.into_bytes())
    }
}

impl VisitorMut for Stabilizer {
    fn enter_property(&mut self, property: NodeProperty) -> WalkControl {
        self.path.push_back(NodeSlot::Property(property));
        WalkControl::Continue
    }

    fn exit_property(&mut self) {
        self.path.pop_back();
    }

    fn enter_index(&mut self, index: usize) -> WalkControl {
        self.path.push_back(NodeSlot::Index(index));
        WalkControl::Continue
    }

    fn exit_index(&mut self) {
        self.path.pop_back();
    }

    fn visit_node(&mut self, node: &mut Node) -> WalkControl {
        let uid = self.uid_from_path();

        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match node {
                    $(Node::$variant(node) => node.uid = uid,)*

                    Node::Null(..) |
                    Node::Boolean(..) |
                    Node::Integer(..) |
                    Node::UnsignedInteger(..) |
                    Node::Number(..) |
                    Node::String(..) |
                    Node::Cord(..) |
                    Node::Array(..) |
                    Node::Object(..) => {},
                }
            };
        }

        variants!(
            Admonition,
            Agent,
            Annotation,
            AppendixBreak,
            ArrayHint,
            ArrayValidator,
            Article,
            AudioObject,
            AuthorRole,
            Bibliography,
            BooleanValidator,
            Brand,
            Button,
            CallArgument,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Citation,
            CitationGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            CodeExpression,
            CodeInline,
            CodeLocation,
            Collection,
            Comment,
            CompilationDigest,
            CompilationMessage,
            ConstantValidator,
            ContactPoint,
            CreativeWork,
            Datatable,
            DatatableColumn,
            DatatableColumnHint,
            DatatableHint,
            Date,
            DateTime,
            DateTimeValidator,
            DateValidator,
            DefinedTerm,
            Directory,
            Duration,
            DurationValidator,
            Emphasis,
            Enumeration,
            EnumValidator,
            Excerpt,
            ExecutionDependant,
            ExecutionDependency,
            ExecutionMessage,
            ExecutionTag,
            Figure,
            File,
            ForBlock,
            Form,
            Function,
            Grant,
            Heading,
            IfBlock,
            IfBlockClause,
            ImageObject,
            IncludeBlock,
            InlinesBlock,
            InstructionBlock,
            InstructionInline,
            InstructionMessage,
            IntegerValidator,
            Island,
            Link,
            List,
            ListItem,
            MathBlock,
            MathInline,
            MediaObject,
            ModelParameters,
            MonetaryGrant,
            Note,
            NumberValidator,
            ObjectHint,
            Organization,
            Page,
            Paragraph,
            Parameter,
            Periodical,
            Person,
            PostalAddress,
            Product,
            Prompt,
            PromptBlock,
            PropertyValue,
            ProvenanceCount,
            PublicationIssue,
            PublicationVolume,
            QuoteBlock,
            QuoteInline,
            RawBlock,
            Reference,
            Review,
            Section,
            Sentence,
            Skill,
            SoftwareApplication,
            SoftwareSourceCode,
            Strikeout,
            StringHint,
            StringValidator,
            Strong,
            StyledBlock,
            StyledInline,
            Subscript,
            SuggestionBlock,
            SuggestionInline,
            Superscript,
            Supplement,
            Table,
            TableCell,
            TableRow,
            Text,
            ThematicBreak,
            Thing,
            Time,
            Timestamp,
            TimestampValidator,
            TimeValidator,
            TupleValidator,
            Underline,
            Unknown,
            Variable,
            VideoObject,
            Walkthrough,
            WalkthroughStep,
            Workflow
        );

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        // Handle headings specially with content-based slugs
        if let Block::Heading(heading) = block {
            heading.uid = self.heading_slug(heading);
            return WalkControl::Continue;
        }

        let uid = self.uid_from_path();

        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match block {
                    $(Block::$variant(node) => node.uid = uid,)*
                    Block::Heading(..) => unreachable!(),
                }
            };
        }

        variants!(
            Admonition,
            AppendixBreak,
            AudioObject,
            CallBlock,
            Chat,
            ChatMessage,
            ChatMessageGroup,
            Claim,
            CodeBlock,
            CodeChunk,
            Datatable,
            Excerpt,
            Figure,
            File,
            ForBlock,
            Form,
            IfBlock,
            ImageObject,
            IncludeBlock,
            InlinesBlock,
            InstructionBlock,
            Island,
            List,
            MathBlock,
            Page,
            Paragraph,
            PromptBlock,
            QuoteBlock,
            RawBlock,
            Section,
            StyledBlock,
            SuggestionBlock,
            Supplement,
            Table,
            ThematicBreak,
            VideoObject,
            Walkthrough
        );

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        let uid = self.uid_from_path();

        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match inline {
                    $(Inline::$variant(node) => node.uid = uid,)*

                    Inline::Null(..) |
                    Inline::Boolean(..) |
                    Inline::Integer(..) |
                    Inline::UnsignedInteger(..) |
                    Inline::Number(..) => {},
                }
            };
        }

        variants!(
            Annotation,
            AudioObject,
            Button,
            Citation,
            CitationGroup,
            CodeExpression,
            CodeInline,
            Date,
            DateTime,
            Duration,
            Emphasis,
            ImageObject,
            InstructionInline,
            Link,
            MathInline,
            MediaObject,
            Note,
            Parameter,
            QuoteInline,
            Sentence,
            Strikeout,
            Strong,
            StyledInline,
            Subscript,
            SuggestionInline,
            Superscript,
            Text,
            Time,
            Timestamp,
            Underline,
            VideoObject
        );

        WalkControl::Continue
    }

    fn visit_list_item(&mut self, list_item: &mut ListItem) -> WalkControl {
        list_item.uid = self.uid_from_path();
        WalkControl::Continue
    }

    fn visit_table_row(&mut self, table_row: &mut TableRow) -> WalkControl {
        table_row.uid = self.uid_from_path();
        WalkControl::Continue
    }

    fn visit_table_cell(&mut self, table_cell: &mut TableCell) -> WalkControl {
        table_cell.uid = self.uid_from_path();
        WalkControl::Continue
    }

    fn visit_if_block_clause(&mut self, clause: &mut IfBlockClause) -> WalkControl {
        clause.uid = self.uid_from_path();
        WalkControl::Continue
    }

    fn visit_walkthrough_step(&mut self, step: &mut WalkthroughStep) -> WalkControl {
        step.uid = self.uid_from_path();
        WalkControl::Continue
    }

    fn visit_citation(&mut self, citation: &mut Citation) -> WalkControl {
        citation.uid = self.uid_from_path();
        WalkControl::Continue
    }

    fn visit_suggestion_block(&mut self, block: &mut SuggestionBlock) -> WalkControl {
        block.uid = self.uid_from_path();
        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, inline: &mut SuggestionInline) -> WalkControl {
        inline.uid = self.uid_from_path();
        WalkControl::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_schema::{Article, Heading, Paragraph, Text};

    #[test]
    fn test_stabilize_simple_document() {
        // Create a simple document
        let article = Article {
            content: vec![
                Block::Paragraph(Paragraph {
                    content: vec![Inline::Text(Text {
                        value: "Hello".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
                Block::Paragraph(Paragraph {
                    content: vec![Inline::Text(Text {
                        value: "World".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };

        // Stabilize
        let mut node = Node::Article(article.clone());
        stabilize(&mut node);

        // Extract the stabilized article
        let Node::Article(stabilized) = node else {
            panic!("Expected Article");
        };

        // Root article should have empty path UID
        assert_eq!(stabilized.node_id().uid_str(), "");

        // The paragraphs should have deterministic UIDs based on their path
        // Path includes property name and index
        let Block::Paragraph(para1) = &stabilized.content[0] else {
            panic!("Expected Paragraph");
        };
        let Block::Paragraph(para2) = &stabilized.content[1] else {
            panic!("Expected Paragraph");
        };

        // First paragraph: content property, index 0 (kebab-case)
        assert_eq!(para1.node_id().to_string(), "pgh_content-0");
        // Second paragraph: content property, index 1
        assert_eq!(para2.node_id().to_string(), "pgh_content-1");

        // The text nodes inside should have their own paths
        let Inline::Text(text1) = &para1.content[0] else {
            panic!("Expected Text");
        };
        let Inline::Text(text2) = &para2.content[0] else {
            panic!("Expected Text");
        };

        // First text: content[0].content[0] (kebab-case)
        assert_eq!(text1.node_id().to_string(), "txt_content-0-content-0");
        // Second text: content[1].content[0]
        assert_eq!(text2.node_id().to_string(), "txt_content-1-content-0");
    }

    #[test]
    fn test_stabilize_is_deterministic() {
        // Create a document
        let create_doc = || Article {
            content: vec![Block::Paragraph(Paragraph {
                content: vec![Inline::Text(Text {
                    value: "Test".into(),
                    ..Default::default()
                })],
                ..Default::default()
            })],
            ..Default::default()
        };

        // Stabilize twice and compare
        let mut node1 = Node::Article(create_doc());
        let mut node2 = Node::Article(create_doc());

        stabilize(&mut node1);
        stabilize(&mut node2);

        let Node::Article(article1) = node1 else {
            panic!("Expected Article");
        };
        let Node::Article(article2) = node2 else {
            panic!("Expected Article");
        };

        let Block::Paragraph(para1) = &article1.content[0] else {
            panic!("Expected Paragraph");
        };
        let Block::Paragraph(para2) = &article2.content[0] else {
            panic!("Expected Paragraph");
        };

        // UIDs should be identical
        assert_eq!(para1.node_id().to_string(), para2.node_id().to_string());
    }

    #[test]
    fn test_heading_slugs() {
        // Create a document with headings
        let article = Article {
            content: vec![
                Block::Heading(Heading {
                    content: vec![Inline::Text(Text {
                        value: "Introduction".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
                Block::Heading(Heading {
                    content: vec![Inline::Text(Text {
                        value: "Hello World!".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
                Block::Heading(Heading {
                    content: vec![Inline::Text(Text {
                        value: "Introduction".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };

        let mut node = Node::Article(article);
        stabilize(&mut node);

        let Node::Article(stabilized) = node else {
            panic!("Expected Article");
        };

        // First heading: "Introduction" -> "introduction"
        let Block::Heading(heading1) = &stabilized.content[0] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading1.node_id().to_string(), "hea_introduction");

        // Second heading: "Hello World!" -> "hello-world"
        let Block::Heading(heading2) = &stabilized.content[1] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading2.node_id().to_string(), "hea_hello-world");

        // Third heading: duplicate "Introduction" -> "introduction-2"
        let Block::Heading(heading3) = &stabilized.content[2] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading3.node_id().to_string(), "hea_introduction-2");
    }

    #[test]
    fn test_heading_slugs_unicode() {
        // Test non-ASCII heading content (Unicode support)
        let article = Article {
            content: vec![
                Block::Heading(Heading {
                    content: vec![Inline::Text(Text {
                        value: "日本語タイトル".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
                Block::Heading(Heading {
                    content: vec![Inline::Text(Text {
                        value: "Ελληνικά".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
                Block::Heading(Heading {
                    content: vec![Inline::Text(Text {
                        value: "日本語タイトル".into(),
                        ..Default::default()
                    })],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };

        let mut node = Node::Article(article);
        stabilize(&mut node);

        let Node::Article(stabilized) = node else {
            panic!("Expected Article");
        };

        // Japanese heading preserved (lowercased where applicable)
        let Block::Heading(heading1) = &stabilized.content[0] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading1.node_id().uid_str(), "日本語タイトル");

        // Greek heading lowercased
        let Block::Heading(heading2) = &stabilized.content[1] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading2.node_id().uid_str(), "ελληνικά");

        // Duplicate Japanese heading gets suffix
        let Block::Heading(heading3) = &stabilized.content[2] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading3.node_id().uid_str(), "日本語タイトル-2");
    }

    #[test]
    fn test_heading_slugs_empty_content() {
        // Test empty heading content falls back to "heading"
        let article = Article {
            content: vec![
                Block::Heading(Heading {
                    content: vec![],
                    ..Default::default()
                }),
                Block::Heading(Heading {
                    content: vec![],
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };

        let mut node = Node::Article(article);
        stabilize(&mut node);

        let Node::Article(stabilized) = node else {
            panic!("Expected Article");
        };

        // Empty heading falls back to "heading"
        let Block::Heading(heading1) = &stabilized.content[0] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading1.node_id().uid_str(), "heading");

        // Second empty heading gets suffix
        let Block::Heading(heading2) = &stabilized.content[1] else {
            panic!("Expected Heading");
        };
        assert_eq!(heading2.node_id().uid_str(), "heading-2");
    }
}
