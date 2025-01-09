use bitflags::bitflags;
use monostate::MustBe;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use codec::common::{serde_json, serde_with::skip_serializing_none};

/// Lexical node types represented as Rust structs with support
/// for serialization/deserialization
///
/// Implements the node types in Lexical:
///
/// https://lexical.dev/docs/concepts/nodes
/// https://lexical.dev/docs/api/classes/lexical.LexicalNode
///
/// As well as those extension node type found in Koenig:
///
/// https://github.com/TryGhost/Koenig

/// Block node types
///
/// Lexical does not have an equivalent for this enumeration
/// but we use it here for enforcing which node types
/// can be children of others. Lexical seems to use the `isInline` method for that.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum BlockNode {
    Heading(HeadingNode),
    ExtendedHeading(ExtendedHeadingNode),
    Paragraph(ParagraphNode),
    List(ListNode),
    Quote(QuoteNode),
    Aside(AsideNode),
    ExtendedQuote(ExtendedQuoteNode),
    Image(ImageNode),
    Audio(AudioNode),
    Video(VideoNode),
    CodeBlock(CodeBlockNode),
    Markdown(MarkdownNode),
    Html(HtmlNode),
    HorizontalRule(HorizontalRuleNode),
    Unknown(UnknownNode),
}

/// Inline node types
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(super) enum InlineNode {
    Text(TextNode),
    ExtendedText(ExtendedTextNode),
    Link(LinkNode),
    HashTag(HashTagNode),
    Unknown(UnknownNode),
}

// Allow deserialization of JSON containing node types unknown to
// this codec
type UnknownNode = serde_json::Value;

#[derive(Serialize, Deserialize)]
pub(super) struct LexicalDoc {
    pub root: RootNode,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct RootNode {
    pub r#type: MustBe!("root"),

    pub children: Vec<BlockNode>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct HeadingNode {
    pub r#type: MustBe!("heading"),

    pub tag: HeadingTagType,

    pub children: Vec<InlineNode>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct ExtendedHeadingNode {
    pub r#type: MustBe!("extended-heading"),

    pub tag: HeadingTagType,

    pub children: Vec<InlineNode>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum HeadingTagType {
    #[default]
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct ParagraphNode {
    pub r#type: MustBe!("paragraph"),

    pub children: Vec<InlineNode>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct QuoteNode {
    pub r#type: MustBe!("quote"),

    pub children: Vec<InlineNode>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct ExtendedQuoteNode {
    pub r#type: MustBe!("extended-quote"),

    pub children: Vec<InlineNode>,
}

#[derive(SmartDefault, Serialize, Deserialize)]
pub(super) struct ListNode {
    pub r#type: MustBe!("list"),

    #[serde(rename = "listType")]
    pub list_type: ListType,

    /// The start number for numbered lists, but required even for bullet lists
    #[default = 1]
    pub start: u32,

    pub children: Vec<ListItemNode>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum ListType {
    #[default]
    Number,
    Bullet,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
pub(super) struct ListItemNode {
    pub r#type: MustBe!("listitem"),

    /// The 1-based position of the item in the list (even bullet list items)
    pub value: Option<u32>,

    pub checked: Option<bool>,

    /// Assumes that only inline nodes are expected here
    /// (whereas in Stencila, block nodes are expected)
    pub children: Vec<InlineNode>,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ImageNode {
    pub r#type: MustBe!("image"),

    pub src: String,

    pub width: Option<u32>,

    pub height: Option<u32>,

    pub title: Option<String>,

    pub alt: Option<String>,

    pub caption: Option<String>,

    pub card_width: Option<String>,

    pub href: Option<String>,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
pub(super) struct CodeBlockNode {
    pub r#type: MustBe!("codeblock"),

    pub code: String,

    pub language: Option<String>,

    pub caption: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct MarkdownNode {
    pub r#type: MustBe!("markdown"),

    pub markdown: String,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
pub(super) struct HtmlNode {
    pub r#type: MustBe!("html"),

    pub html: String,

    pub visibility: Option<HtmlVisibility>,
}

#[derive(SmartDefault, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HtmlVisibility {
    #[default = true]
    pub show_on_email: bool,

    #[default = true]
    pub show_on_web: bool,

    pub segment: String,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct HorizontalRuleNode {
    pub r#type: MustBe!("horizontalrule"),
}

#[derive(Serialize, Deserialize)]
pub(super) struct HashTagNode {
    pub r#type: MustBe!("hashtag"),

    pub format: TextFormat,

    pub text: String,
}

#[skip_serializing_none]
#[derive(Default, Serialize, Deserialize)]
pub(super) struct LinkNode {
    pub r#type: MustBe!("link"),

    pub children: Vec<InlineNode>,

    pub format: String,

    pub url: String,

    pub title: Option<String>,

    pub target: Option<String>,

    pub rel: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct TextNode {
    pub r#type: MustBe!("text"),

    pub format: TextFormat,

    pub text: String,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct ExtendedTextNode {
    pub r#type: MustBe!("extended-text"),

    pub format: TextFormat,

    pub text: String,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AudioNode {
    pub r#type: MustBe!("audio"),

    pub mime_type: Option<String>,

    pub src: String,

    pub title: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub(super) struct AsideNode {
    pub r#type: MustBe!("aside"),

    pub children: Vec<InlineNode>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VideoNode {
    pub r#type: MustBe!("video"),

    pub src: String,

    pub file_name: Option<String>,

    pub mime_type: Option<String>,

    pub width: Option<u32>,

    pub height: Option<u32>,

    pub duration: Option<u32>,

    pub thumbnail_src: Option<String>,

    pub custom_thumbnail_src: Option<String>,

    pub thumbnail_width: Option<u32>,

    pub thumbnail_height: Option<u32>,

    pub card_width: Option<String>,

    pub r#loop: Option<bool>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TextFormat: u8 {
        const NORMAL        = 0b00000000;
        const BOLD          = 0b00000001;
        const ITALIC        = 0b00000010;
        const STRIKETHROUGH = 0b00000100;
        const UNDERLINE     = 0b00001000;
        const CODE          = 0b00010000;
        const SUBSCRIPT     = 0b00100000;
        const SUPERSCRIPT   = 0b01000000;
        const HIGHLIGHT     = 0b10000000;
    }
}

impl Serialize for TextFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}

impl<'de> Deserialize<'de> for TextFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        Ok(TextFormat::from_bits_truncate(bits))
    }
}
