use codec::stencila_schema::*;

/// A trait to encode a `Node` as plain text
///
/// Not implemented for all node types - feel free to add them!
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
                    .concat()
            }
        }
    };
}
slice_to_txt!([Node]);
slice_to_txt!([InlineContent]);
slice_to_txt!([BlockContent]);

macro_rules! primitive_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                json5::to_string(self).expect("Should always convert to JSON5")
            }
        }
    };
}
primitive_to_txt!(Null);
primitive_to_txt!(Boolean);
primitive_to_txt!(Integer);
primitive_to_txt!(Number);
primitive_to_txt!(Array);
primitive_to_txt!(Object);

impl ToTxt for String {
    fn to_txt(&self) -> String {
        self.to_string()
    }
}

macro_rules! inline_content_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                self.content.to_txt()
            }
        }
    };
}

inline_content_to_txt!(Delete);
inline_content_to_txt!(Emphasis);
inline_content_to_txt!(Link);
inline_content_to_txt!(NontextualAnnotation);
inline_content_to_txt!(Note);
inline_content_to_txt!(Quote);
inline_content_to_txt!(Strong);
inline_content_to_txt!(Subscript);
inline_content_to_txt!(Superscript);

macro_rules! block_content_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                [&self.content.to_txt(), "\n\n"].concat()
            }
        }
    };
}

block_content_to_txt!(ClaimSimple);
block_content_to_txt!(Paragraph);
block_content_to_txt!(Heading);
block_content_to_txt!(QuoteBlock);

macro_rules! inline_text_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                self.text.to_string()
            }
        }
    };
}

inline_text_to_txt!(CodeFragment);
inline_text_to_txt!(MathFragment);

macro_rules! block_text_to_txt {
    ($type:ty) => {
        impl ToTxt for $type {
            fn to_txt(&self) -> String {
                [&self.text.to_string(), "\n\n"].concat()
            }
        }
    };
}

block_text_to_txt!(CodeBlock);
block_text_to_txt!(MathBlock);

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

/// Encode a `Node` to plain text
impl ToTxt for Node {
    fn to_txt(&self) -> String {
        match self {
            Node::Array(node) => node.to_txt(),
            Node::Article(node) => node.to_txt(),
            Node::Boolean(node) => node.to_txt(),
            Node::Cite(node) => node.to_txt(),
            Node::CodeBlock(node) => node.to_txt(),
            Node::CodeFragment(node) => node.to_txt(),
            Node::Delete(node) => node.to_txt(),
            Node::Emphasis(node) => node.to_txt(),
            Node::Heading(node) => node.to_txt(),
            Node::Integer(node) => node.to_txt(),
            Node::Link(node) => node.to_txt(),
            Node::NontextualAnnotation(node) => node.to_txt(),
            Node::Note(node) => node.to_txt(),
            Node::Null(node) => node.to_txt(),
            Node::Number(node) => node.to_txt(),
            Node::Object(node) => node.to_txt(),
            Node::Paragraph(node) => node.to_txt(),
            Node::Quote(node) => node.to_txt(),
            Node::QuoteBlock(node) => node.to_txt(),
            Node::String(node) => node.to_txt(),
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
            InlineContent::Boolean(node) => node.to_txt(),
            InlineContent::Cite(node) => node.to_txt(),
            InlineContent::CodeFragment(node) => node.to_txt(),
            InlineContent::Delete(node) => node.to_txt(),
            InlineContent::Emphasis(node) => node.to_txt(),
            InlineContent::Integer(node) => node.to_txt(),
            InlineContent::Link(node) => node.to_txt(),
            InlineContent::NontextualAnnotation(node) => node.to_txt(),
            InlineContent::Note(node) => node.to_txt(),
            InlineContent::Null(node) => node.to_txt(),
            InlineContent::Number(node) => node.to_txt(),
            InlineContent::MathFragment(node) => node.to_txt(),
            InlineContent::Quote(node) => node.to_txt(),
            InlineContent::String(node) => node.to_txt(),
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
