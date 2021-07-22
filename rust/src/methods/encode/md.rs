use eyre::Result;
use itertools::Itertools;
use std::cmp::max;
use stencila_schema::*;

/// Encode a `Node` to Markdown
pub fn encode(node: &Node) -> Result<String> {
    Ok(node.to_md().trim().to_string())
}

/// A trait to encode a `Node` as Markdown
pub trait ToMd {
    fn to_md(&self) -> String;
}

macro_rules! slice_to_md {
    ($type:ty) => {
        impl ToMd for $type {
            fn to_md(&self) -> String {
                self.iter()
                    .map(|item| item.to_md())
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
            fn to_md(&self) -> String {
                [$delimiter, &self.content.to_md(), $delimiter].concat()
            }
        }
    };
}

delimited_inline_content_to_md!(Delete, "~~");
delimited_inline_content_to_md!(Emphasis, "_");
delimited_inline_content_to_md!(Strong, "**");
delimited_inline_content_to_md!(Subscript, "~");
delimited_inline_content_to_md!(Superscript, "^");

impl ToMd for NontextualAnnotation {
    fn to_md(&self) -> String {
        ["<u>", &self.content.to_md(), "</u>"].concat()
    }
}

impl ToMd for Quote {
    fn to_md(&self) -> String {
        ["<q>", &self.content.to_md(), "</q>"].concat()
    }
}

macro_rules! delimited_inline_text_to_md {
    ($type:ty, $delimiter:expr) => {
        impl ToMd for $type {
            fn to_md(&self) -> String {
                [$delimiter, &self.text, $delimiter].concat()
            }
        }
    };
}

delimited_inline_text_to_md!(CodeFragment, "`");
delimited_inline_text_to_md!(MathFragment, "$");

impl ToMd for Link {
    fn to_md(&self) -> String {
        ["[", &self.content.to_md(), "](", &self.target, ")"].concat()
    }
}

macro_rules! inline_media_object_to_md {
    ($type:ty) => {
        impl ToMd for $type {
            fn to_md(&self) -> String {
                ["![", "](", &self.content_url, ")"].concat()
            }
        }
    };
}

inline_media_object_to_md!(AudioObjectSimple);
inline_media_object_to_md!(ImageObjectSimple);
inline_media_object_to_md!(VideoObjectSimple);

impl ToMd for Heading {
    fn to_md(&self) -> String {
        [
            &"#".repeat(self.depth.unwrap_or(1) as usize),
            " ",
            &self.content.to_md(),
            "\n\n",
        ]
        .concat()
    }
}

impl ToMd for Paragraph {
    fn to_md(&self) -> String {
        [&self.content.to_md(), "\n\n"].concat()
    }
}

impl ToMd for CodeBlock {
    fn to_md(&self) -> String {
        let lang = match &self.programming_language {
            Some(boxed) => boxed.as_str(),
            None => "",
        };

        ["```", lang, "\n", &self.text, "\n```\n\n"].concat()
    }
}

impl ToMd for List {
    fn to_md(&self) -> String {
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
                item.to_md()
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
    fn to_md(&self) -> String {
        let checkbox = self.is_checked.map(|is_checked| match is_checked {
            true => InlineContent::String("[x] ".to_string()),
            false => InlineContent::String("[ ] ".to_string()),
        });
        match &self.content {
            Some(content) => match content {
                ListItemContent::VecInlineContent(inlines) => match checkbox {
                    Some(checkbox) => [vec![checkbox], inlines.clone()].concat().to_md(),
                    None => inlines.to_md(),
                },
                ListItemContent::VecBlockContent(blocks) => match checkbox {
                    Some(checkbox) => {
                        // Check box is only added is the first block is a paragraph
                        if let Some(BlockContent::Paragraph(paragraph)) = blocks.first() {
                            let mut paragraph = paragraph.clone();
                            paragraph.content.insert(0, checkbox);
                            [paragraph.to_md(), blocks[1..].to_md()].concat()
                        } else {
                            blocks.to_md()
                        }
                    }
                    None => blocks.to_md(),
                },
            },
            None => "".to_string(),
        }
    }
}

impl ToMd for QuoteBlock {
    fn to_md(&self) -> String {
        let content: Vec<String> = self
            .content
            .iter()
            .map(|block| {
                block
                    .to_md()
                    .split('\n')
                    .map(|line| ["> ", line].concat())
                    .join("\n")
            })
            .collect();
        [content.join("\n"), "\n\n".to_string()].concat()
    }
}

impl ToMd for TableSimple {
    fn to_md(&self) -> String {
        let mut column_widths: Vec<usize> = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();
        for row in &self.rows {
            let mut cells: Vec<String> = Vec::new();
            for (column, cell) in row.cells.iter().enumerate() {
                let content = match &cell.content {
                    None => "".to_string(),
                    Some(content) => match content {
                        TableCellContent::VecInlineContent(inlines) => inlines.to_md(),
                        TableCellContent::VecBlockContent(blocks) => blocks.to_md(),
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
                    format!("{:width$}", content, width = column_widths[column])
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
    fn to_md(&self) -> String {
        "---\n\n".to_string()
    }
}

impl ToMd for Article {
    fn to_md(&self) -> String {
        match &self.content {
            Some(content) => content.to_md(),
            None => "".to_string(),
        }
    }
}

/// Encode a `Node` to plain text
impl ToMd for Node {
    fn to_md(&self) -> String {
        match self {
            Node::Article(node) => node.to_md(),
            Node::Boolean(node) => node.to_string(),
            //Node::Cite(node) => node.to_md(),
            Node::CodeBlock(node) => node.to_md(),
            Node::CodeFragment(node) => node.to_md(),
            Node::Delete(node) => node.to_md(),
            Node::Emphasis(node) => node.to_md(),
            Node::Heading(node) => node.to_md(),
            Node::Integer(node) => node.to_string(),
            Node::Link(node) => node.to_md(),
            Node::List(node) => node.to_md(),
            Node::NontextualAnnotation(node) => node.to_md(),
            //Node::Note(node) => node.to_md(),
            Node::Null => "null".to_string(),
            Node::Number(node) => node.to_string(),
            Node::Paragraph(node) => node.to_md(),
            Node::Quote(node) => node.to_md(),
            Node::QuoteBlock(node) => node.to_md(),
            Node::String(node) => node.to_string(),
            Node::Strong(node) => node.to_md(),
            Node::Subscript(node) => node.to_md(),
            Node::Superscript(node) => node.to_md(),
            _ => "".to_string(),
        }
    }
}

impl ToMd for InlineContent {
    fn to_md(&self) -> String {
        match self {
            InlineContent::AudioObject(node) => node.to_md(),
            InlineContent::Boolean(node) => node.to_string(),
            //InlineContent::Cite(node) => node.to_md(),
            InlineContent::CodeFragment(node) => node.to_md(),
            InlineContent::Delete(node) => node.to_md(),
            InlineContent::Emphasis(node) => node.to_md(),
            InlineContent::ImageObject(node) => node.to_md(),
            InlineContent::Integer(node) => node.to_string(),
            InlineContent::Link(node) => node.to_md(),
            InlineContent::NontextualAnnotation(node) => node.to_md(),
            //InlineContent::Note(node) => node.to_md(),
            InlineContent::Null => "null".to_string(),
            InlineContent::Number(node) => node.to_string(),
            InlineContent::MathFragment(node) => node.to_md(),
            InlineContent::Quote(node) => node.to_md(),
            InlineContent::String(node) => node.to_string(),
            InlineContent::Strong(node) => node.to_md(),
            InlineContent::Subscript(node) => node.to_md(),
            InlineContent::Superscript(node) => node.to_md(),
            InlineContent::VideoObject(node) => node.to_md(),
            _ => "".to_string(),
        }
    }
}

impl ToMd for BlockContent {
    fn to_md(&self) -> String {
        match self {
            //BlockContent::Claim(node) => node.to_md(),
            BlockContent::CodeBlock(node) => node.to_md(),
            BlockContent::Heading(node) => node.to_md(),
            BlockContent::List(node) => node.to_md(),
            //BlockContent::MathBlock(node) => node.to_md(),
            BlockContent::Paragraph(node) => node.to_md(),
            BlockContent::QuoteBlock(node) => node.to_md(),
            BlockContent::Table(node) => node.to_md(),
            BlockContent::ThematicBreak(node) => node.to_md(),
            _ => "".to_string(),
        }
    }
}

impl ToMd for ThingDescription {
    fn to_md(&self) -> String {
        match self {
            ThingDescription::String(string) => string.to_string(),
            ThingDescription::VecInlineContent(inlines) => inlines.to_md(),
            ThingDescription::VecBlockContent(blocks) => blocks.to_md(),
        }
    }
}
