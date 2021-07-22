use crate::{binaries, methods::decode::pandoc::PANDOC_SEMVER, traits::ToVecBlockContent};
use eyre::Result;
use pandoc_types::definition as pandoc;
use std::{collections::HashMap, io::Write, process::Stdio};
use stencila_schema::*;

/// Encode a `Node` to a document via Pandoc
pub async fn encode(node: &Node, output: &str, format: &str, args: &[String]) -> Result<String> {
    let pandoc = encode_node(node)?;
    encode_pandoc(pandoc, output, format, args).await
}

/// Encode a `Node` to a Pandoc document
pub fn encode_node(node: &Node) -> Result<pandoc::Pandoc> {
    Ok(node.to_pandoc())
}

/// Encode a Pandoc document to desired format.
///
/// Calls Pandoc binary to convert the Pandoc JSON to the desired format.
async fn encode_pandoc(
    doc: pandoc::Pandoc,
    output: &str,
    format: &str,
    args: &[String],
) -> Result<String> {
    let json = serde_json::to_string(&doc)?;

    if format == "pandoc" {
        Ok(json)
    } else {
        let binary = binaries::require("pandoc", PANDOC_SEMVER).await?;

        let mut command = binary.command();
        command.args(["--from", "json", "--to", format]);
        command.args(args);
        if let Some(path) = output.strip_prefix("file://") {
            command.args(["--output", path]);
        }

        let mut child = command
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(json.as_ref())?;
        }

        let result = child.wait_with_output()?;
        let stdout = std::str::from_utf8(result.stdout.as_ref())?.to_string();

        if output.starts_with("file://") {
            Ok(output.into())
        } else {
            Ok(stdout)
        }
    }
}

/// A trait to encode a `Node` as a Pandoc element
pub trait ToPandoc {
    fn to_pandoc(&self) -> pandoc::Pandoc {
        pandoc::Pandoc(pandoc::Meta(HashMap::new()), Vec::new())
    }

    fn to_pandoc_inline(&self) -> pandoc::Inline {
        pandoc::Inline::Str("".to_string())
    }

    fn to_pandoc_block(&self) -> pandoc::Block {
        pandoc::Block::HorizontalRule
    }

    fn to_pandoc_inlines(&self) -> Vec<pandoc::Inline> {
        Vec::new()
    }

    fn to_pandoc_blocks(&self) -> Vec<pandoc::Block> {
        Vec::new()
    }
}

/// Create an empty Pandoc `Attr` tuple
fn attrs_empty() -> pandoc::Attr {
    pandoc::Attr("".to_string(), Vec::new(), Vec::new())
}

macro_rules! unimplemented_to_pandoc {
    ($type:ty) => {
        impl ToPandoc for $type {}
    };
}

fn null_to_pandoc_inline() -> pandoc::Inline {
    pandoc::Inline::Str("null".to_string())
}

macro_rules! inline_primitive_to_pandoc_str {
    ($type:ty) => {
        impl ToPandoc for $type {
            fn to_pandoc_inline(&self) -> pandoc::Inline {
                pandoc::Inline::Str(self.to_string())
            }
        }
    };
}

inline_primitive_to_pandoc_str!(Boolean);
inline_primitive_to_pandoc_str!(Integer);
inline_primitive_to_pandoc_str!(Number);
inline_primitive_to_pandoc_str!(String);

macro_rules! inline_content_to_pandoc_inline {
    ($type:ty, $pandoc:expr) => {
        impl ToPandoc for $type {
            fn to_pandoc_inline(&self) -> pandoc::Inline {
                $pandoc(self.content.to_pandoc_inlines())
            }
        }
    };
}

inline_content_to_pandoc_inline!(Delete, pandoc::Inline::Strikeout);
inline_content_to_pandoc_inline!(Emphasis, pandoc::Inline::Emph);
inline_content_to_pandoc_inline!(NontextualAnnotation, pandoc::Inline::Underline);
inline_content_to_pandoc_inline!(Strong, pandoc::Inline::Strong);
inline_content_to_pandoc_inline!(Subscript, pandoc::Inline::Subscript);
inline_content_to_pandoc_inline!(Superscript, pandoc::Inline::Superscript);

macro_rules! inline_media_to_pandoc_image {
    ($type:ty) => {
        impl ToPandoc for $type {
            fn to_pandoc_inline(&self) -> pandoc::Inline {
                pandoc::Inline::Image(
                    attrs_empty(),
                    Vec::new(), // TODO: content or caption here
                    pandoc::Target(self.content_url.clone(), "".to_string()),
                )
            }
        }
    };
}

inline_media_to_pandoc_image!(AudioObjectSimple);
inline_media_to_pandoc_image!(ImageObjectSimple);
inline_media_to_pandoc_image!(VideoObjectSimple);

unimplemented_to_pandoc!(Cite);

unimplemented_to_pandoc!(CiteGroup);

unimplemented_to_pandoc!(CodeExpression);

impl ToPandoc for CodeFragment {
    fn to_pandoc_inline(&self) -> pandoc::Inline {
        pandoc::Inline::Code(attrs_empty(), self.text.clone())
    }
}

impl ToPandoc for Link {
    fn to_pandoc_inline(&self) -> pandoc::Inline {
        pandoc::Inline::Link(
            attrs_empty(),
            self.content.to_pandoc_inlines(),
            pandoc::Target(
                self.target.clone(),
                self.title
                    .as_ref()
                    .map_or("".to_string(), |title| title.to_string()),
            ),
        )
    }
}

impl ToPandoc for MathFragment {
    fn to_pandoc_inline(&self) -> pandoc::Inline {
        pandoc::Inline::Math(pandoc::MathType::InlineMath, self.text.clone())
    }
}

unimplemented_to_pandoc!(Note);

impl ToPandoc for Quote {
    fn to_pandoc_inline(&self) -> pandoc::Inline {
        pandoc::Inline::Quoted(
            pandoc::QuoteType::DoubleQuote,
            self.content.to_pandoc_inlines(),
        )
    }
}

impl ToPandoc for InlineContent {
    fn to_pandoc_inline(&self) -> pandoc::Inline {
        match self {
            InlineContent::AudioObject(node) => node.to_pandoc_inline(),
            InlineContent::Boolean(node) => node.to_pandoc_inline(),
            InlineContent::Cite(node) => node.to_pandoc_inline(),
            InlineContent::CiteGroup(node) => node.to_pandoc_inline(),
            InlineContent::CodeExpression(node) => node.to_pandoc_inline(),
            InlineContent::CodeFragment(node) => node.to_pandoc_inline(),
            InlineContent::Delete(node) => node.to_pandoc_inline(),
            InlineContent::Emphasis(node) => node.to_pandoc_inline(),
            InlineContent::ImageObject(node) => node.to_pandoc_inline(),
            InlineContent::Integer(node) => node.to_pandoc_inline(),
            InlineContent::Link(node) => node.to_pandoc_inline(),
            InlineContent::MathFragment(node) => node.to_pandoc_inline(),
            InlineContent::NontextualAnnotation(node) => node.to_pandoc_inline(),
            InlineContent::Note(node) => node.to_pandoc_inline(),
            InlineContent::Null => null_to_pandoc_inline(),
            InlineContent::Number(node) => node.to_pandoc_inline(),
            InlineContent::Quote(node) => node.to_pandoc_inline(),
            InlineContent::String(node) => node.to_pandoc_inline(),
            InlineContent::Strong(node) => node.to_pandoc_inline(),
            InlineContent::Subscript(node) => node.to_pandoc_inline(),
            InlineContent::Superscript(node) => node.to_pandoc_inline(),
            InlineContent::VideoObject(node) => node.to_pandoc_inline(),
        }
    }
}

impl ToPandoc for [InlineContent] {
    fn to_pandoc_inlines(&self) -> Vec<pandoc::Inline> {
        self.iter().map(|item| item.to_pandoc_inline()).collect()
    }
}

unimplemented_to_pandoc!(ClaimSimple);

impl ToPandoc for CodeBlock {
    fn to_pandoc_block(&self) -> pandoc::Block {
        let id = self.id.as_ref().map_or("".to_string(), |id| *id.clone());
        let classes = self
            .programming_language
            .as_ref()
            .map_or(vec![], |lang| vec![*lang.clone()]);
        let attrs = pandoc::Attr(id, classes, vec![]);
        pandoc::Block::CodeBlock(attrs, self.text.clone())
    }
}

unimplemented_to_pandoc!(CodeChunk);

unimplemented_to_pandoc!(CollectionSimple);

unimplemented_to_pandoc!(FigureSimple);

impl ToPandoc for Heading {
    fn to_pandoc_block(&self) -> pandoc::Block {
        pandoc::Block::Header(
            self.depth.unwrap_or(1) as i32,
            attrs_empty(),
            self.content.to_pandoc_inlines(),
        )
    }
}

impl ToPandoc for List {
    fn to_pandoc_block(&self) -> pandoc::Block {
        let items = self
            .items
            .iter()
            .map(|item| match &item.content {
                Some(content) => match content {
                    ListItemContent::VecInlineContent(inlines) => {
                        vec![pandoc::Block::Para(inlines.to_pandoc_inlines())]
                    }
                    ListItemContent::VecBlockContent(blocks) => blocks.to_pandoc_blocks(),
                },
                None => Vec::new(),
            })
            .collect();
        match &self.order {
            Some(ListOrder::Ascending) => pandoc::Block::OrderedList(
                pandoc::ListAttributes(
                    1,
                    pandoc::ListNumberStyle::Decimal,
                    pandoc::ListNumberDelim::DefaultDelim,
                ),
                items,
            ),
            _ => pandoc::Block::BulletList(items),
        }
    }
}

unimplemented_to_pandoc!(MathBlock);

impl ToPandoc for Paragraph {
    fn to_pandoc_block(&self) -> pandoc::Block {
        pandoc::Block::Para(self.content.to_pandoc_inlines())
    }
}

impl ToPandoc for QuoteBlock {
    fn to_pandoc_block(&self) -> pandoc::Block {
        pandoc::Block::BlockQuote(self.content.to_pandoc_blocks())
    }
}

impl ToPandoc for ThematicBreak {
    fn to_pandoc_block(&self) -> pandoc::Block {
        pandoc::Block::HorizontalRule
    }
}

impl ToPandoc for TableSimple {
    fn to_pandoc_block(&self) -> pandoc::Block {
        let mut head = vec![];
        let mut body = vec![];
        let mut foot = vec![];
        for row in &self.rows {
            let cells = row
                .cells
                .iter()
                .map(|cell| {
                    let blocks = match &cell.content {
                        None => Vec::new(),
                        Some(content) => match content {
                            TableCellContent::VecInlineContent(inlines) => {
                                inlines.to_vec_block_content().to_pandoc_blocks()
                            }
                            TableCellContent::VecBlockContent(blocks) => blocks.to_pandoc_blocks(),
                        },
                    };
                    pandoc::Cell(attrs_empty(), pandoc::Alignment::AlignDefault, 1, 1, blocks)
                })
                .collect();
            let pandoc_row = pandoc::Row(attrs_empty(), cells);
            match row.row_type {
                Some(TableRowRowType::Header) => head.push(pandoc_row),
                Some(TableRowRowType::Footer) => foot.push(pandoc_row),
                _ => body.push(pandoc_row),
            }
        }

        pandoc::Block::Table(
            attrs_empty(),
            pandoc::Caption(None, vec![]),
            vec![],
            pandoc::TableHead(attrs_empty(), head),
            vec![pandoc::TableBody(attrs_empty(), 1, vec![], body)],
            pandoc::TableFoot(attrs_empty(), foot),
        )
    }
}

impl ToPandoc for BlockContent {
    fn to_pandoc_block(&self) -> pandoc::Block {
        match self {
            BlockContent::Claim(node) => node.to_pandoc_block(),
            BlockContent::CodeBlock(node) => node.to_pandoc_block(),
            BlockContent::CodeChunk(node) => node.to_pandoc_block(),
            BlockContent::Collection(node) => node.to_pandoc_block(),
            BlockContent::Figure(node) => node.to_pandoc_block(),
            BlockContent::Heading(node) => node.to_pandoc_block(),
            BlockContent::List(node) => node.to_pandoc_block(),
            BlockContent::MathBlock(node) => node.to_pandoc_block(),
            BlockContent::Paragraph(node) => node.to_pandoc_block(),
            BlockContent::QuoteBlock(node) => node.to_pandoc_block(),
            BlockContent::Table(node) => node.to_pandoc_block(),
            BlockContent::ThematicBreak(node) => node.to_pandoc_block(),
        }
    }
}

impl ToPandoc for [BlockContent] {
    fn to_pandoc_blocks(&self) -> Vec<pandoc::Block> {
        self.iter().map(|item| item.to_pandoc_block()).collect()
    }
}

impl ToPandoc for Article {
    fn to_pandoc(&self) -> pandoc::Pandoc {
        let meta = pandoc::Meta(HashMap::new());

        let blocks = self
            .content
            .as_ref()
            .map_or_else(Vec::new, |content| content.to_pandoc_blocks());

        pandoc::Pandoc(meta, blocks)
    }
}

impl ToPandoc for Node {
    fn to_pandoc(&self) -> pandoc::Pandoc {
        match self {
            Node::Article(node) => node.to_pandoc(),
            _ => {
                unimplemented!(
                    "Encoding via Pandoc is not currently supported for nodes of this type"
                )
            }
        }
    }
}
