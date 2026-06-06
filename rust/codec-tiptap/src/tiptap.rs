//! Internal Tiptap and ProseMirror JSON data structures.
//!
//! These structs model only the native Tiptap nodes currently handled by this
//! codec plus the custom Stencila extension nodes used for opaque preservation.
//! Unknown native nodes and marks are held as raw JSON values so callers can
//! record conversion losses with their original Tiptap type names.

use monostate::MustBe;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The root Tiptap document node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TiptapDoc {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("doc"),

    /// The top-level block content in the document.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<BlockNode>,
}

/// A block-level Tiptap node supported by this codec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum BlockNode {
    /// A native Tiptap heading node.
    Heading(HeadingNode),
    /// A native Tiptap paragraph node.
    Paragraph(ParagraphNode),
    /// A custom opaque Stencila block node.
    StencilaBlock(StencilaBlockNode),
    /// Any unsupported native block node.
    Unknown(Value),
}

/// A native Tiptap paragraph node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct ParagraphNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("paragraph"),

    /// Inline content contained by the paragraph.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<InlineNode>,
}

/// A native Tiptap heading node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct HeadingNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("heading"),

    /// Heading attributes.
    pub attrs: HeadingAttrs,

    /// Inline content contained by the heading.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<InlineNode>,
}

/// Attributes for a native Tiptap heading node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct HeadingAttrs {
    /// The heading level, expected to be between one and six.
    pub level: u8,
}

/// A custom block node containing an opaque Stencila block payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct StencilaBlockNode {
    /// The fixed custom Tiptap node type.
    pub r#type: MustBe!("stencilaBlock"),

    /// Attributes used to preserve the Stencila block.
    pub attrs: StencilaAttrs,
}

/// A custom inline node containing an opaque Stencila inline payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct StencilaInlineNode {
    /// The fixed custom Tiptap node type.
    pub r#type: MustBe!("stencilaInline"),

    /// Attributes used to preserve the Stencila inline.
    pub attrs: StencilaAttrs,
}

/// Attributes shared by custom opaque Stencila nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct StencilaAttrs {
    /// The expected Stencila node type for the opaque payload.
    #[serde(rename = "nodeType")]
    pub node_type: String,

    /// The serialized Stencila node payload.
    pub node: Value,
}

/// An inline-level Tiptap node supported by this codec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum InlineNode {
    /// A native Tiptap text node.
    Text(TextNode),
    /// A custom opaque Stencila inline node.
    StencilaInline(StencilaInlineNode),
    /// Any unsupported native inline node.
    Unknown(Value),
}

/// A native Tiptap text node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TextNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("text"),

    /// Marks applied to the text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Mark>,

    /// The text content.
    pub text: String,
}

/// A Tiptap mark supported by this codec or held as raw JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum Mark {
    /// A known native Tiptap mark.
    Known(KnownMark),
    /// Any unsupported native mark.
    Unknown(Value),
}

/// A known native Tiptap mark.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct KnownMark {
    /// The mark type.
    pub r#type: MarkType,

    /// Mark attributes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attrs: Option<MarkAttrs>,
}

/// Attributes for native Tiptap marks.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MarkAttrs {
    /// Link destination URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,

    /// Advisory link title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Relationship between the document and target.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rel: Option<String>,

    /// Whether a link should show only the internal target label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_only: Option<bool>,

    /// Programming language for inline code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programming_language: Option<String>,

    /// Unsupported mark attributes captured so conversion can report losses.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    pub extra: Map<String, Value>,
}

impl MarkAttrs {
    /// Whether this mark has no attributes to serialize.
    pub fn is_empty(&self) -> bool {
        self.href.is_none()
            && self.title.is_none()
            && self.rel.is_none()
            && self.label_only.is_none()
            && self.programming_language.is_none()
            && self.extra.is_empty()
    }
}

/// Native Tiptap mark types supported by this codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum MarkType {
    /// Bold text.
    Bold,
    /// Inline code.
    Code,
    /// Italic text.
    Italic,
    /// Linked text.
    Link,
    /// Struck out text.
    #[serde(rename = "strike")]
    Strikeout,
    /// Subscripted text.
    Subscript,
    /// Superscripted text.
    Superscript,
    /// Underlined text.
    Underline,
}

/// Get the Tiptap `type` string for a raw JSON node or mark.
pub(super) fn value_type(value: &Value) -> &str {
    value
        .get("type")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown")
}
