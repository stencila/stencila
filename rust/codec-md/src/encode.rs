use std::cmp::max;

use codec::{
    common::{eyre::Result, itertools::Itertools, serde_json},
    stencila_schema::*,
    EncodeOptions,
};

use crate::utils::escape;

/// Encode a `Node` to Markdown
pub fn encode(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
    let options = options.unwrap_or_default();
    Ok(node.to_md(&options).trim().to_string())
}

/// A trait to encode a `Node` as Markdown
pub trait ToMd {
    fn to_md(&self, options: &EncodeOptions) -> String;
}

macro_rules! primitive_to_md {
    ($type:ty) => {
        impl ToMd for $type {
            fn to_md(&self, _options: &EncodeOptions) -> String {
                self.to_string()
            }
        }
    };
}

primitive_to_md!(Null);
primitive_to_md!(Boolean);
primitive_to_md!(Integer);
primitive_to_md!(Number);
primitive_to_md!(String);

impl<Type> ToMd for Option<Type>
where
    Type: ToMd,
{
    fn to_md(&self, options: &EncodeOptions) -> String {
        match self {
            Some(value) => value.to_md(options),
            None => "".to_string(),
        }
    }
}

impl<Type> ToMd for Box<Type>
where
    Type: ToMd,
{
    fn to_md(&self, options: &EncodeOptions) -> String {
        self.as_ref().to_md(options)
    }
}

impl<Type> ToMd for Vec<Type>
where
    Type: ToMd,
{
    fn to_md(&self, options: &EncodeOptions) -> String {
        self.iter()
            .map(|item| item.to_md(options))
            .collect::<Vec<String>>()
            .concat()
    }
}

macro_rules! slice_to_md {
    ($type:ty) => {
        impl ToMd for $type {
            fn to_md(&self, options: &EncodeOptions) -> String {
                self.iter()
                    .map(|item| item.to_md(options))
                    .collect::<Vec<String>>()
                    .concat()
            }
        }
    };
}
slice_to_md!([Node]);
slice_to_md!([InlineContent]);
slice_to_md!([BlockContent]);

macro_rules! delimited_inline_content_to_md {
    ($type:ty, $delimiter:expr) => {
        impl ToMd for $type {
            fn to_md(&self, options: &EncodeOptions) -> String {
                [$delimiter, &self.content.to_md(options), $delimiter].concat()
            }
        }
    };
}

delimited_inline_content_to_md!(Emphasis, "_");
delimited_inline_content_to_md!(Strikeout, "~~");
delimited_inline_content_to_md!(Strong, "**");
delimited_inline_content_to_md!(Subscript, "~");
delimited_inline_content_to_md!(Superscript, "^");

impl ToMd for Underline {
    fn to_md(&self, options: &EncodeOptions) -> String {
        ["<u>", &self.content.to_md(options), "</u>"].concat()
    }
}

impl ToMd for Quote {
    fn to_md(&self, options: &EncodeOptions) -> String {
        ["<q>", &self.content.to_md(options), "</q>"].concat()
    }
}

impl ToMd for CodeExpression {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        ["`", &self.text, "`{", &self.programming_language, " exec}"].concat()
    }
}

macro_rules! delimited_inline_text_to_md {
    ($type:ty, $delimiter:expr) => {
        impl ToMd for $type {
            fn to_md(&self, _options: &EncodeOptions) -> String {
                [$delimiter, &self.text, $delimiter].concat()
            }
        }
    };
}

delimited_inline_text_to_md!(CodeFragment, "`");

impl ToMd for MathFragment {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        match self.math_language.as_ref().map(|string| string.as_str()) {
            Some("asciimath") => ["`", &self.text, "`{asciimath}"].concat(),
            _ => ["$", &self.text, "$"].concat(),
        }
    }
}

impl ToMd for Parameter {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        let mut options = String::new();

        if let Some(validator) = &self.validator {
            match validator.as_ref() {
                ValidatorTypes::BooleanValidator(..) => {
                    options += "bool";
                }
                ValidatorTypes::IntegerValidator(validator) => {
                    options += "int";
                    if let Some(min) = validator.minimum {
                        options += " min=";
                        options += &min.to_string();
                    }
                    if let Some(max) = validator.maximum {
                        options += " max=";
                        options += &max.to_string();
                    }
                    if let Some(mult) = validator.multiple_of {
                        options += " mult=";
                        options += &mult.to_string();
                    }
                }
                ValidatorTypes::NumberValidator(validator) => {
                    options += "num";
                    if let Some(min) = validator.minimum {
                        options += " min=";
                        options += &min.to_string();
                    }
                    if let Some(max) = validator.maximum {
                        options += " max=";
                        options += &max.to_string();
                    }
                    if let Some(mult) = validator.multiple_of {
                        options += " mult=";
                        options += &mult.to_string();
                    }
                }
                ValidatorTypes::StringValidator(validator) => {
                    options += "str";
                    if let Some(min) = validator.min_length {
                        options += " min=";
                        options += &min.to_string();
                    }
                    if let Some(max) = validator.max_length {
                        options += " max=";
                        options += &max.to_string();
                    }
                    if let Some(pattern) = validator.pattern.as_deref() {
                        options += &[" pattern=\"", &escape(pattern), "\""].concat();
                    }
                }
                ValidatorTypes::EnumValidator(validator) => {
                    let json = serde_json::to_string(&validator.values)
                        .unwrap_or_else(|_| "[]".to_string());
                    options += &["enum vals=", &escape(&json)].concat();
                }
                _ => {}
            };
        }

        if let Some(default) = &self.default {
            let json = serde_json::to_string(&default).unwrap_or_else(|_| "null".to_string());
            options += &[" def=", &escape(&json)].concat();
        }

        let attrs = if options.is_empty() {
            "{}".to_string()
        } else {
            ["{", &options, "}"].concat()
        };

        ["&[", &self.name, "]", &attrs].concat()
    }
}

impl ToMd for Link {
    fn to_md(&self, options: &EncodeOptions) -> String {
        ["[", &self.content.to_md(options), "](", &self.target, ")"].concat()
    }
}

macro_rules! inline_media_object_to_md {
    ($type:ty) => {
        impl ToMd for $type {
            fn to_md(&self, _options: &EncodeOptions) -> String {
                ["![", "](", &self.content_url, ")"].concat()
            }
        }
    };
}

inline_media_object_to_md!(AudioObjectSimple);
inline_media_object_to_md!(ImageObjectSimple);
inline_media_object_to_md!(VideoObjectSimple);

impl ToMd for Heading {
    fn to_md(&self, options: &EncodeOptions) -> String {
        [
            &"#".repeat(self.depth.unwrap_or(1) as usize),
            " ",
            &self.content.to_md(options),
            "\n\n",
        ]
        .concat()
    }
}

impl ToMd for Paragraph {
    fn to_md(&self, options: &EncodeOptions) -> String {
        let mut md = self.content.to_md(options);
        if let Some(width) = options.max_width {
            textwrap::fill_inplace(&mut md, width);
        };
        [&md, "\n\n"].concat()
    }
}

impl ToMd for CodeBlock {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        let lang = match &self.programming_language {
            Some(boxed) => boxed.as_str(),
            None => "",
        };

        ["```", lang, "\n", &self.text, "\n```\n\n"].concat()
    }
}

impl ToMd for CodeChunk {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        [
            "```",
            &self.programming_language,
            " exec\n",
            &self.text,
            "\n```\n\n",
        ]
        .concat()
    }
}

impl ToMd for MathBlock {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        match self.math_language.as_ref().map(|string| string.as_str()) {
            Some("asciimath") => ["```asciimath\n", &self.text, "\n```\n\n"].concat(),
            _ => ["$$\n", &self.text, "\n$$\n\n"].concat(),
        }
    }
}

impl ToMd for List {
    fn to_md(&self, options: &EncodeOptions) -> String {
        let ordered = matches!(&self.order, Some(ListOrder::Ascending));
        let items: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let bullet = if ordered {
                    (index + 1).to_string() + ". "
                } else {
                    "- ".to_string()
                };
                item.to_md(options)
                    .split('\n')
                    .enumerate()
                    .map(|(index, line)| {
                        if index == 0 {
                            [bullet.clone(), line.to_string()].concat()
                        } else if line.trim().is_empty() {
                            String::new()
                        } else {
                            ["  ", line].concat()
                        }
                    })
                    .join("\n")
            })
            .collect();

        // Keep lists tight if no items have internal newlines
        let mut tight = true;
        for item in &items {
            if item.trim().contains('\n') {
                tight = false;
                break;
            }
        }
        let items = items
            .iter()
            .map(|item| item.trim())
            .join(if tight { "\n" } else { "\n\n" });

        [items, "\n\n".to_string()].concat()
    }
}

impl ToMd for ListItem {
    fn to_md(&self, options: &EncodeOptions) -> String {
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => InlineContent::String("[x] ".to_string()),
            false => InlineContent::String("[ ] ".to_string()),
        });
        match &self.content {
            Some(content) => match content {
                ListItemContent::VecInlineContent(inlines) => match checkbox {
                    Some(checkbox) => [vec![checkbox], inlines.clone()].concat().to_md(options),
                    None => inlines.to_md(options),
                },
                ListItemContent::VecBlockContent(blocks) => match checkbox {
                    Some(checkbox) => {
                        // Check box is only added is the first block is a paragraph
                        if let Some(BlockContent::Paragraph(paragraph)) = blocks.first() {
                            let mut paragraph = paragraph.clone();
                            paragraph.content.insert(0, checkbox);
                            [paragraph.to_md(options), blocks[1..].to_md(options)].concat()
                        } else {
                            blocks.to_md(options)
                        }
                    }
                    None => blocks.to_md(options),
                },
            },
            None => "".to_string(),
        }
    }
}

impl ToMd for QuoteBlock {
    fn to_md(&self, options: &EncodeOptions) -> String {
        let content: Vec<String> = self
            .content
            .iter()
            .map(|block| {
                block
                    .to_md(options)
                    .trim()
                    .lines()
                    .map(|line| ["> ", line].concat())
                    .join("\n")
            })
            .collect();
        [content.join("\n"), "\n\n".to_string()].concat()
    }
}

impl ToMd for TableSimple {
    fn to_md(&self, options: &EncodeOptions) -> String {
        let mut column_widths: Vec<usize> = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();
        for row in &self.rows {
            let mut cells: Vec<String> = Vec::new();
            for (column, cell) in row.cells.iter().enumerate() {
                let content = match &cell.content {
                    None => "".to_string(),
                    Some(content) => match content {
                        TableCellContent::VecInlineContent(inlines) => inlines.to_md(options),
                        TableCellContent::VecBlockContent(blocks) => blocks.to_md(options),
                    },
                };
                let width = content.len();
                match column_widths.get_mut(column) {
                    Some(column_width) => {
                        if width > *column_width {
                            *column_width = width
                        }
                    }
                    None => column_widths.push(max(3, width)),
                }
                cells.push(content);
            }
            rows.push(cells);
        }

        let row_to_md = |cells: &[String]| -> String {
            cells
                .iter()
                .enumerate()
                .map(|(column, content)| {
                    format!(
                        "{:width$}",
                        // Ensure cell has no newlines which will break table
                        content.replace("\r\n", " ").replace('\n', " "),
                        width = column_widths[column]
                    )
                })
                .join(" | ")
        };

        let (first, rest) = if rows.len() == 1 {
            (
                row_to_md(&vec!["".to_string(); column_widths.len()]),
                row_to_md(&rows[0]),
            )
        } else {
            (
                row_to_md(&rows[0]),
                rows[1..].iter().map(|row| row_to_md(row)).join(" |\n| "),
            )
        };

        let dashes = column_widths
            .iter()
            .map(|width| "-".repeat(*width))
            .join(" | ");

        [
            "| ", &first, " |\n", "| ", &dashes, " |\n", "| ", &rest, " |\n\n",
        ]
        .concat()
    }
}

impl ToMd for ThematicBreak {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        "---\n\n".to_string()
    }
}

impl ToMd for Include {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        let mut options = Vec::new();

        if let Some(media_type) = self.media_type.as_deref() {
            options.push(["format=", media_type].concat())
        }

        if let Some(select) = self.select.as_deref() {
            options.push(["select=", select].concat())
        }

        let attrs = if options.is_empty() {
            "".to_string()
        } else {
            ["{", &options.join(" "), "}"].concat()
        };

        ["/", &self.source, &attrs, "\n\n"].concat()
    }
}

impl ToMd for Call {
    fn to_md(&self, _options: &EncodeOptions) -> String {
        let args = self
            .arguments
            .iter()
            .flatten()
            .map(|arg| {
                [
                    &arg.name,
                    "=",
                    &serde_json::to_string(&arg.value).unwrap_or_default(),
                ]
                .concat()
            })
            .join(" ");

        let mut options = Vec::new();

        if let Some(media_type) = self.media_type.as_deref() {
            options.push(["format=", media_type].concat())
        }

        if let Some(select) = self.select.as_deref() {
            options.push(["select=", select].concat())
        }

        let attrs = if options.is_empty() {
            "".to_string()
        } else {
            ["{", &options.join(" "), "}"].concat()
        };

        ["/", &self.source, "(", &args, ")", &attrs, "\n\n"].concat()
    }
}

macro_rules! content_to_md {
    ($type:ty) => {
        impl ToMd for $type {
            fn to_md(&self, options: &EncodeOptions) -> String {
                self.content.to_md(options)
            }
        }
    };
}

content_to_md!(Article);
content_to_md!(CreativeWork);

impl ToMd for CreativeWorkContent {
    fn to_md(&self, options: &EncodeOptions) -> String {
        match self {
            CreativeWorkContent::String(node) => node.to_md(options),
            CreativeWorkContent::VecNode(nodes) => nodes.to_md(options),
        }
    }
}

impl ToMd for Node {
    fn to_md(&self, options: &EncodeOptions) -> String {
        match self {
            Node::Article(node) => node.to_md(options),
            Node::Boolean(node) => node.to_md(options),
            Node::CodeBlock(node) => node.to_md(options),
            Node::CodeFragment(node) => node.to_md(options),
            Node::CreativeWork(node) => node.to_md(options),
            Node::Emphasis(node) => node.to_md(options),
            Node::Heading(node) => node.to_md(options),
            Node::Integer(node) => node.to_md(options),
            Node::Link(node) => node.to_md(options),
            Node::List(node) => node.to_md(options),
            Node::Null(node) => node.to_md(options),
            Node::Number(node) => node.to_md(options),
            Node::Paragraph(node) => node.to_md(options),
            Node::Parameter(node) => node.to_md(options),
            Node::Quote(node) => node.to_md(options),
            Node::QuoteBlock(node) => node.to_md(options),
            Node::Strikeout(node) => node.to_md(options),
            Node::String(node) => node.to_md(options),
            Node::Strong(node) => node.to_md(options),
            Node::Subscript(node) => node.to_md(options),
            Node::Superscript(node) => node.to_md(options),
            Node::Underline(node) => node.to_md(options),
            _ => format!(
                "<!-- Markdown encoding for Node::{} is not yet supported -->\n\n",
                self.as_ref()
            ),
        }
    }
}

impl ToMd for InlineContent {
    fn to_md(&self, options: &EncodeOptions) -> String {
        match self {
            InlineContent::AudioObject(node) => node.to_md(options),
            InlineContent::Boolean(node) => node.to_md(options),
            InlineContent::CodeExpression(node) => node.to_md(options),
            InlineContent::CodeFragment(node) => node.to_md(options),
            InlineContent::Emphasis(node) => node.to_md(options),
            InlineContent::ImageObject(node) => node.to_md(options),
            InlineContent::Integer(node) => node.to_md(options),
            InlineContent::Link(node) => node.to_md(options),
            InlineContent::Null(node) => node.to_md(options),
            InlineContent::Number(node) => node.to_md(options),
            InlineContent::MathFragment(node) => node.to_md(options),
            InlineContent::Parameter(node) => node.to_md(options),
            InlineContent::Quote(node) => node.to_md(options),
            InlineContent::Strikeout(node) => node.to_md(options),
            InlineContent::String(node) => node.to_md(options),
            InlineContent::Strong(node) => node.to_md(options),
            InlineContent::Subscript(node) => node.to_md(options),
            InlineContent::Superscript(node) => node.to_md(options),
            InlineContent::Underline(node) => node.to_md(options),
            InlineContent::VideoObject(node) => node.to_md(options),
            _ => format!(
                "<!-- Markdown encoding for InlineContent::{} is not yet supported -->\n\n",
                self.as_ref()
            ),
        }
    }
}

impl ToMd for BlockContent {
    fn to_md(&self, options: &EncodeOptions) -> String {
        match self {
            BlockContent::Call(node) => node.to_md(options),
            BlockContent::CodeBlock(node) => node.to_md(options),
            BlockContent::CodeChunk(node) => node.to_md(options),
            BlockContent::Heading(node) => node.to_md(options),
            BlockContent::Include(node) => node.to_md(options),
            BlockContent::List(node) => node.to_md(options),
            BlockContent::MathBlock(node) => node.to_md(options),
            BlockContent::Paragraph(node) => node.to_md(options),
            BlockContent::QuoteBlock(node) => node.to_md(options),
            BlockContent::Table(node) => node.to_md(options),
            BlockContent::ThematicBreak(node) => node.to_md(options),
            _ => format!(
                "<!-- Markdown encoding for BlockContent::{} is not yet supported -->\n\n",
                self.as_ref()
            ),
        }
    }
}

impl ToMd for ThingDescription {
    fn to_md(&self, options: &EncodeOptions) -> String {
        match self {
            ThingDescription::String(string) => string.to_string(),
            ThingDescription::VecInlineContent(inlines) => inlines.to_md(options),
            ThingDescription::VecBlockContent(blocks) => blocks.to_md(options),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::pretty_assertions::assert_eq;

    /// Test wrapping of long paragraphs
    #[test]
    fn encode_paragraph_long() {
        let md = encode(
            &Node::Paragraph(Paragraph {
                content: vec![InlineContent::String(
                    "This should be on first and this on second, and yep, this on third."
                        .to_string(),
                )],
                ..Default::default()
            }),
            Some(EncodeOptions {
                max_width: Some(24),
                ..Default::default()
            }),
        )
        .unwrap();
        assert_eq!(
            md,
            "This should be on first\nand this on second, and\nyep, this on third."
        )
    }

    /// A regression test that quote blocks do not have unnecessary lines starting with >
    #[test]
    fn encode_quote_block() {
        let md = encode(
            &Node::QuoteBlock(QuoteBlock {
                content: vec![BlockContent::Paragraph(Paragraph {
                    content: vec![InlineContent::String("Hello world.".to_string())],
                    ..Default::default()
                })],
                ..Default::default()
            }),
            None,
        )
        .unwrap();
        assert_eq!(md, "> Hello world.")
    }
}
