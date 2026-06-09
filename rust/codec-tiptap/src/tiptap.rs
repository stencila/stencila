//! Internal Tiptap and ProseMirror JSON data structures.
//!
//! These structs model only the native Tiptap nodes currently handled by this
//! codec plus the custom Stencila extension nodes used for opaque preservation.
//! Unknown native nodes and marks are held as raw JSON values so callers can
//! record conversion losses with their original Tiptap type names.

use monostate::MustBe;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stencila_codec::stencila_schema::{Block, CompilationMessage, ImageObject};

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
    /// A native Tiptap blockquote node.
    Blockquote(BlockquoteNode),
    /// A native Tiptap bullet list node.
    BulletList(BulletListNode),
    /// A native Tiptap code block node.
    CodeBlock(CodeBlockNode),
    /// A native Tiptap heading node.
    Heading(HeadingNode),
    /// A native Tiptap horizontal rule node.
    HorizontalRule(HorizontalRuleNode),
    /// A native Stencila math block node.
    MathBlock(MathBlockNode),
    /// A native Tiptap ordered list node.
    OrderedList(OrderedListNode),
    /// A native Tiptap paragraph node.
    Paragraph(ParagraphNode),
    /// A native Tiptap table node.
    Table(TableNode),
    /// A native Tiptap task list node.
    TaskList(TaskListNode),
    /// A custom opaque Stencila block node.
    StencilaBlock(StencilaBlockNode),
    /// Any unsupported native block node.
    Unknown(Value),
}

/// A native Tiptap blockquote node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct BlockquoteNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("blockquote"),

    /// Block content contained by the quote.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<BlockNode>,
}

/// A native Tiptap bullet list node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct BulletListNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("bulletList"),

    /// List items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ListItemNode>,
}

/// A native Tiptap ordered list node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct OrderedListNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("orderedList"),

    /// Ordered list attributes.
    #[serde(default)]
    pub attrs: OrderedListAttrs,

    /// List items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ListItemNode>,
}

/// Attributes for a native Tiptap ordered list node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct OrderedListAttrs {
    /// The first list item number.
    #[serde(default = "one")]
    pub start: u64,

    /// Ordered list numbering marker type.
    pub r#type: Option<String>,

    /// Unsupported ordered list attributes captured for loss reporting.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    pub extra: Map<String, Value>,
}

impl Default for OrderedListAttrs {
    fn default() -> Self {
        Self {
            start: 1,
            r#type: None,
            extra: Map::new(),
        }
    }
}

/// A native Tiptap list item node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct ListItemNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("listItem"),

    /// Block content contained by the list item.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<BlockNode>,
}

/// A native Tiptap task list node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TaskListNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("taskList"),

    /// Task items.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<TaskItemNode>,
}

/// A native Tiptap task item node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TaskItemNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("taskItem"),

    /// Task item attributes.
    #[serde(default)]
    pub attrs: TaskItemAttrs,

    /// Block content contained by the task item.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<BlockNode>,
}

/// Attributes for a native Tiptap task item node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TaskItemAttrs {
    /// Whether the task item is checked.
    #[serde(default)]
    pub checked: bool,
}

/// A native Tiptap code block node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct CodeBlockNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("codeBlock"),

    /// Code block attributes.
    #[serde(default)]
    pub attrs: CodeBlockAttrs,

    /// Plain text content contained by the code block.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<InlineNode>,
}

/// Attributes for a native Tiptap code block node.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CodeBlockAttrs {
    /// Programming language for the code.
    pub language: Option<String>,

    /// Stencila node identifier for the code block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Whether the code block is a demo that should also be rendered.
    #[serde(
        default,
        alias = "is_demo",
        alias = "is-demo",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_demo: Option<bool>,
}

/// A native Stencila math block node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct MathBlockNode {
    /// The fixed custom Tiptap node type.
    pub r#type: MustBe!("mathBlock"),

    /// Math block attributes.
    #[serde(default)]
    pub attrs: MathBlockAttrs,
}

/// Attributes for a native Stencila math block node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MathBlockAttrs {
    /// Fields shared by block and inline math nodes.
    #[serde(flatten)]
    pub math: MathAttrs,

    /// Whether the Stencila node id should be automatically updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_automatically: Option<bool>,

    /// Short label for the math block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Whether the math block label should be automatically updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_automatically: Option<bool>,
}

/// Attributes shared by native Stencila math block and inline nodes.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct MathAttrs {
    /// Stencila node identifier for the math.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Source code for the math.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// Source language for the math.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub math_language: Option<String>,

    /// Messages generated while compiling the math.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// MathML generated from the source code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mathml: Option<String>,

    /// Images generated for the math.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ImageObject>>,
}

/// A native Tiptap horizontal rule node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct HorizontalRuleNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("horizontalRule"),
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

/// A native Tiptap table node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TableNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("table"),

    /// Table attributes.
    #[serde(default)]
    pub attrs: TableAttrs,

    /// Table rows.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<TableRowNode>,
}

/// Attributes for a native Tiptap table node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TableAttrs {
    /// Stencila node identifier for the table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Short label for the table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Whether the label should be automatically updated.
    #[serde(
        default,
        alias = "label_automatically",
        alias = "label-automatically",
        skip_serializing_if = "Option::is_none"
    )]
    pub label_automatically: Option<bool>,

    /// Stencila block caption for the table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<Vec<Block>>,

    /// Stencila block notes for the table.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<Block>>,
}

/// A native Tiptap table row node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TableRowNode {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("tableRow"),

    /// Table cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<TableCellNode>,
}

/// A table cell-level Tiptap node supported by this codec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum TableCellNode {
    /// A native Tiptap table data cell.
    TableCell(TableCell),
    /// A native Tiptap table header cell.
    TableHeader(TableHeader),
    /// Any unsupported native table cell node.
    Unknown(Value),
}

/// A native Tiptap table data cell node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TableCell {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("tableCell"),

    /// Table cell attributes.
    #[serde(default)]
    pub attrs: TableCellAttrs,

    /// Block content contained by the cell.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<BlockNode>,
}

/// A native Tiptap table header cell node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TableHeader {
    /// The fixed Tiptap node type.
    pub r#type: MustBe!("tableHeader"),

    /// Table cell attributes.
    #[serde(default)]
    pub attrs: TableCellAttrs,

    /// Block content contained by the cell.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<BlockNode>,
}

/// Attributes for a native Tiptap table cell node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct TableCellAttrs {
    /// How many columns the cell spans.
    #[serde(default = "one")]
    pub colspan: u64,

    /// How many rows the cell spans.
    #[serde(default = "one")]
    pub rowspan: u64,

    /// Unsupported ProseMirror table column widths.
    pub colwidth: Option<Vec<Option<u64>>>,

    /// Horizontal cell alignment.
    pub align: Option<String>,

    /// Unsupported table cell attributes captured for loss reporting.
    #[serde(default, flatten, skip_serializing_if = "Map::is_empty")]
    pub extra: Map<String, Value>,
}

fn one() -> u64 {
    1
}

impl Default for TableCellAttrs {
    fn default() -> Self {
        Self {
            colspan: 1,
            rowspan: 1,
            colwidth: None,
            align: None,
            extra: Map::new(),
        }
    }
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
    /// A native Stencila math inline node.
    MathInline(MathInlineNode),
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

/// A native Stencila math inline node.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct MathInlineNode {
    /// The fixed custom Tiptap node type.
    pub r#type: MustBe!("mathInline"),

    /// Math inline attributes.
    #[serde(default)]
    pub attrs: MathAttrs,
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
