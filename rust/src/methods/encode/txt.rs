use eyre::Result;
use stencila_schema::*;

/// Encode a `Node` to plain, unstructured text
///
/// This is an an intentionally lossy encoding for when a
/// plain text encoding of a node is needed. It just dumps
/// the content of a node as one, possibly long, string.
pub fn encode(node: &Node) -> Result<String> {
    Ok(node.to_txt())
}

/// A trait to encode a `Node` as plain text
///
/// Made public for other modules in this crate.
/// Not implemented for all node types. Feel free
/// to add them!
pub trait ToTxt {
    fn to_txt(&self) -> String;
}

macro_rules! slice_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                self.iter()
                    .map(|item| item.to_txt())
                    .collect::<Vec<String>>()
                    .join("")
            }
        }
    };
}
slice_to_txt!([Node]);
slice_to_txt!([InlineContent]);
slice_to_txt!([BlockContent]);

macro_rules! content_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                self.content.to_txt()
            }
        }
    };
}

content_to_txt!(Delete);
content_to_txt!(Emphasis);
content_to_txt!(Link);
content_to_txt!(NontextualAnnotation);
content_to_txt!(Note);
content_to_txt!(Quote);
content_to_txt!(Strong);
content_to_txt!(Subscript);
content_to_txt!(Superscript);

content_to_txt!(ClaimSimple);
content_to_txt!(Paragraph);
content_to_txt!(Heading);
content_to_txt!(QuoteBlock);

macro_rules! optional_content_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                match &self.content {
                    Some(content) => content.to_txt(),
                    None => "".to_string(),
                }
            }
        }
    };
}

optional_content_to_txt!(Article);
optional_content_to_txt!(Cite);

macro_rules! text_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                self.text.to_string()
            }
        }
    };
}

text_to_txt!(CodeBlock);
text_to_txt!(CodeFragment);
text_to_txt!(MathBlock);
text_to_txt!(MathFragment);

/// Encode a `Node` to plain text
impl ToTxt for Node {
    fn to_txt(&self) -> String {
        match self {
            Node::Article(node) => node.to_txt(),
            Node::Boolean(node) => node.to_string(),
            Node::Cite(node) => node.to_txt(),
            Node::CodeBlock(node) => node.to_txt(),
            Node::CodeFragment(node) => node.to_txt(),
            Node::Delete(node) => node.to_txt(),
            Node::Emphasis(node) => node.to_txt(),
            Node::Heading(node) => node.to_txt(),
            Node::Integer(node) => node.to_string(),
            Node::Link(node) => node.to_txt(),
            Node::NontextualAnnotation(node) => node.to_txt(),
            Node::Note(node) => node.to_txt(),
            Node::Null => "null".to_string(),
            Node::Number(node) => node.to_string(),
            Node::Paragraph(node) => node.to_txt(),
            Node::Quote(node) => node.to_txt(),
            Node::QuoteBlock(node) => node.to_txt(),
            Node::String(node) => node.to_string(),
            Node::Strong(node) => node.to_txt(),
            Node::Subscript(node) => node.to_txt(),
            Node::Superscript(node) => node.to_txt(),
            _ => "".to_string(),
        }
    }
}

impl ToTxt for InlineContent {
    fn to_txt(&self) -> String {
        match self {
            InlineContent::Boolean(node) => node.to_string(),
            InlineContent::Cite(node) => node.to_txt(),
            InlineContent::CodeFragment(node) => node.to_txt(),
            InlineContent::Delete(node) => node.to_txt(),
            InlineContent::Emphasis(node) => node.to_txt(),
            InlineContent::Integer(node) => node.to_string(),
            InlineContent::Link(node) => node.to_txt(),
            InlineContent::NontextualAnnotation(node) => node.to_txt(),
            InlineContent::Note(node) => node.to_txt(),
            InlineContent::Null => "null".to_string(),
            InlineContent::Number(node) => node.to_string(),
            InlineContent::MathFragment(node) => node.to_txt(),
            InlineContent::Quote(node) => node.to_txt(),
            InlineContent::String(node) => node.to_string(),
            InlineContent::Strong(node) => node.to_txt(),
            InlineContent::Subscript(node) => node.to_txt(),
            InlineContent::Superscript(node) => node.to_txt(),
            _ => "".to_string(),
        }
    }
}

impl ToTxt for BlockContent {
    fn to_txt(&self) -> String {
        match self {
            BlockContent::Claim(node) => node.to_txt(),
            BlockContent::CodeBlock(node) => node.to_txt(),
            BlockContent::Heading(node) => node.to_txt(),
            BlockContent::MathBlock(node) => node.to_txt(),
            BlockContent::Paragraph(node) => node.to_txt(),
            BlockContent::QuoteBlock(node) => node.to_txt(),
            _ => "".to_string(),
        }
    }
}

impl ToTxt for ThingDescription {
    fn to_txt(&self) -> String {
        match self {
            ThingDescription::String(string) => string.to_string(),
            ThingDescription::VecInlineContent(inlines) => inlines.to_txt(),
            ThingDescription::VecBlockContent(blocks) => blocks.to_txt(),
        }
    }
}
