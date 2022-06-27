use std::{collections::HashMap, path::Path};

use pandoc_types::definition as pandoc;

use codec::{
    common::{eyre::Result, tempfile},
    stencila_schema::*,
    CodecTrait, EncodeOptions,
};
use codec_json::JsonCodec;
use node_transform::Transform;
use path_utils::path_slash::PathBufExt;

use crate::to_pandoc;

/// Encode a `Node` to a document via Pandoc
///
/// Intended primarily for use by other internal codec crates e.g. `codec-docx`, `codec-latex`
pub async fn encode(
    node: &Node,
    path: Option<&Path>,
    format: &str,
    args: &[String],
) -> Result<String> {
    let mut context = EncodeContext::new()?;
    let pandoc = node.to_pandoc(&mut context);
    context.generate_rpngs().await?;
    to_pandoc(pandoc, path, format, args).await
}

/// Encode a `Node` to a Pandoc document
///
/// Compared to the `encode` function this function does not spawn a Pandoc
/// process, or create RPNGs and returns a `pandoc_types` definition instead.
/// It intended mainly for generative testing.
pub fn encode_node(node: &Node) -> Result<pandoc::Pandoc> {
    let mut context = EncodeContext::new()?;
    let pandoc = node.to_pandoc(&mut context);
    Ok(pandoc)
}

/// The encoding context.
struct EncodeContext {
    /// The directory where any temporary files are placed
    temp_dir: tempfile::TempDir,

    /// The nodes that should be encoded as RPNGs
    rpng_nodes: Vec<(String, Node)>,
}

impl EncodeContext {
    fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let rpng_nodes = Vec::new();
        Ok(EncodeContext {
            temp_dir,
            rpng_nodes,
        })
    }

    /// Push a node to be encoded as an RPNG
    fn push_rpng(&mut self, type_name: &str, node: Node) -> pandoc::Inline {
        let path = self
            .temp_dir
            .path()
            .join([&self.rpng_nodes.len().to_string(), ".png"].concat())
            .to_slash_lossy()
            .into_owned();

        let json = JsonCodec::to_string(
            &node,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
        )
        .expect("Should be able to encode as JSON");

        self.rpng_nodes.push((path.clone(), node));

        pandoc::Inline::Image(
            attrs_empty(),
            vec![pandoc::Inline::Str(json)],
            pandoc::Target {
                url: path,
                title: type_name.into(),
            },
        )
    }

    /// Generate all the RPNGS
    async fn generate_rpngs(&self) -> Result<()> {
        let nodes: Vec<&Node> = self.rpng_nodes.iter().map(|(_id, node)| node).collect();
        let rpngs = codec_rpng::nodes_to_bytes(&nodes, None).await?;
        for (index, bytes) in rpngs.iter().enumerate() {
            let (path, ..) = &self.rpng_nodes[index];
            std::fs::write(path, bytes)?;
        }
        Ok(())
    }
}

/// A trait to encode a `Node` as a Pandoc element
trait ToPandoc {
    /// Encode to a Pandoc document
    fn to_pandoc(&self, _context: &mut EncodeContext) -> pandoc::Pandoc {
        pandoc::Pandoc {
            meta: HashMap::new(),
            blocks: Vec::new(),
        }
    }

    /// Encode to a Pandoc inline element
    fn to_pandoc_inline(&self, _context: &mut EncodeContext) -> pandoc::Inline {
        pandoc::Inline::Str("".to_string())
    }

    /// Encode to a Pandoc block element
    fn to_pandoc_block(&self, _context: &mut EncodeContext) -> pandoc::Block {
        pandoc::Block::HorizontalRule
    }

    /// Encode to a vector of Pandoc inline elements
    fn to_pandoc_inlines(&self, _context: &mut EncodeContext) -> Vec<pandoc::Inline> {
        Vec::new()
    }

    /// Encode to a vector of Pandoc block elements
    fn to_pandoc_blocks(&self, _context: &mut EncodeContext) -> Vec<pandoc::Block> {
        Vec::new()
    }
}

/// Create an empty Pandoc `Attr` tuple
fn attrs_empty() -> pandoc::Attr {
    pandoc::Attr {
        identifier: "".to_string(),
        classes: Vec::new(),
        attributes: Vec::new(),
    }
}

macro_rules! unimplemented_to_pandoc {
    ($type:ty) => {
        impl ToPandoc for $type {}
    };
}

macro_rules! inline_primitive_to_pandoc_str {
    ($type:ty) => {
        impl ToPandoc for $type {
            fn to_pandoc_inline(&self, _context: &mut EncodeContext) -> pandoc::Inline {
                pandoc::Inline::Str(self.to_string())
            }
        }
    };
}

inline_primitive_to_pandoc_str!(Null);
inline_primitive_to_pandoc_str!(Boolean);
inline_primitive_to_pandoc_str!(Integer);
inline_primitive_to_pandoc_str!(Number);
inline_primitive_to_pandoc_str!(String);

macro_rules! inline_content_to_pandoc_inline {
    ($type:ty, $pandoc:expr) => {
        impl ToPandoc for $type {
            fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
                $pandoc(self.content.to_pandoc_inlines(context))
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
            fn to_pandoc_inline(&self, _context: &mut EncodeContext) -> pandoc::Inline {
                let url = if let Some(path) = self.content_url.strip_prefix("file://") {
                    path.to_string()
                } else {
                    self.content_url.clone()
                };
                // TODO: Work out if / what should go into title
                let title = "".to_string();
                pandoc::Inline::Image(
                    attrs_empty(),
                    Vec::new(), // TODO: content or caption here
                    pandoc::Target { url, title },
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

impl ToPandoc for CodeExpression {
    fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
        context.push_rpng("CodeExpression", Node::CodeExpression(self.clone()))
    }
}

impl ToPandoc for CodeFragment {
    fn to_pandoc_inline(&self, _context: &mut EncodeContext) -> pandoc::Inline {
        pandoc::Inline::Code(attrs_empty(), self.text.clone())
    }
}

impl ToPandoc for Link {
    fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
        pandoc::Inline::Link(
            attrs_empty(),
            self.content.to_pandoc_inlines(context),
            pandoc::Target {
                url: self.target.clone(),
                title: self
                    .title
                    .as_ref()
                    .map_or("".to_string(), |title| title.to_string()),
            },
        )
    }
}

impl ToPandoc for MathFragment {
    fn to_pandoc_inline(&self, _context: &mut EncodeContext) -> pandoc::Inline {
        pandoc::Inline::Math(pandoc::MathType::InlineMath, self.text.clone())
    }
}

unimplemented_to_pandoc!(Note);
unimplemented_to_pandoc!(Parameter);

impl ToPandoc for Quote {
    fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
        pandoc::Inline::Quoted(
            pandoc::QuoteType::DoubleQuote,
            self.content.to_pandoc_inlines(context),
        )
    }
}

impl ToPandoc for InlineContent {
    fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
        match self {
            InlineContent::AudioObject(node) => node.to_pandoc_inline(context),
            InlineContent::Boolean(node) => node.to_pandoc_inline(context),
            InlineContent::Cite(node) => node.to_pandoc_inline(context),
            InlineContent::CiteGroup(node) => node.to_pandoc_inline(context),
            InlineContent::CodeExpression(node) => node.to_pandoc_inline(context),
            InlineContent::CodeFragment(node) => node.to_pandoc_inline(context),
            InlineContent::Delete(node) => node.to_pandoc_inline(context),
            InlineContent::Emphasis(node) => node.to_pandoc_inline(context),
            InlineContent::ImageObject(node) => node.to_pandoc_inline(context),
            InlineContent::Integer(node) => node.to_pandoc_inline(context),
            InlineContent::Link(node) => node.to_pandoc_inline(context),
            InlineContent::MathFragment(node) => node.to_pandoc_inline(context),
            InlineContent::NontextualAnnotation(node) => node.to_pandoc_inline(context),
            InlineContent::Note(node) => node.to_pandoc_inline(context),
            InlineContent::Null(node) => node.to_pandoc_inline(context),
            InlineContent::Number(node) => node.to_pandoc_inline(context),
            InlineContent::Parameter(node) => node.to_pandoc_inline(context),
            InlineContent::Quote(node) => node.to_pandoc_inline(context),
            InlineContent::String(node) => node.to_pandoc_inline(context),
            InlineContent::Strong(node) => node.to_pandoc_inline(context),
            InlineContent::Subscript(node) => node.to_pandoc_inline(context),
            InlineContent::Superscript(node) => node.to_pandoc_inline(context),
            InlineContent::VideoObject(node) => node.to_pandoc_inline(context),
        }
    }
}

impl ToPandoc for [InlineContent] {
    fn to_pandoc_inlines(&self, context: &mut EncodeContext) -> Vec<pandoc::Inline> {
        self.iter()
            .map(|item| item.to_pandoc_inline(context))
            .collect()
    }
}

unimplemented_to_pandoc!(ClaimSimple);

impl ToPandoc for CodeBlock {
    fn to_pandoc_block(&self, _context: &mut EncodeContext) -> pandoc::Block {
        let id = self.id.as_ref().map_or("".to_string(), |id| *id.clone());
        let classes = self
            .programming_language
            .as_ref()
            .map_or(vec![], |lang| vec![*lang.clone()]);
        let attrs = pandoc::Attr {
            identifier: id,
            classes,
            attributes: vec![],
        };
        pandoc::Block::CodeBlock(attrs, self.text.clone())
    }
}

impl ToPandoc for CodeChunk {
    /// Encode a `CodeChunk` to a Pandoc block element
    ///
    /// Encodes the code chunk as a RPNG.
    /// Places any label and figure after the code chunk normal text, rather than as screenshotted content.
    /// Note that these are re-constituted into the code chunk in the reshape function.
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        let CodeChunk { label, caption, .. } = self;

        let mut stripped = self.clone();
        stripped.label = None;
        stripped.caption = None;

        let image = context.push_rpng("CodeChunk", Node::CodeChunk(stripped));
        let image_para = pandoc::Block::Para(vec![image]);

        let blocks = if label.is_some() || caption.is_some() {
            let mut inlines = vec![];
            if let Some(label) = label.as_deref() {
                let mut label = label.to_string();
                label.push_str(". ");
                inlines.push(pandoc::Inline::Strong(vec![pandoc::Inline::Str(label)]))
            }
            if let Some(caption) = caption.as_deref() {
                match caption {
                    CodeChunkCaption::String(string) => {
                        inlines.push(pandoc::Inline::Str(string.clone()))
                    }
                    CodeChunkCaption::VecBlockContent(blocks) => {
                        let mut blocks_as_inlines = blocks.to_inlines().to_pandoc_inlines(context);
                        inlines.append(&mut blocks_as_inlines);
                    }
                };
            }
            vec![image_para, pandoc::Block::Para(inlines)]
        } else {
            vec![image_para]
        };

        pandoc::Block::Div(attrs_empty(), blocks)
    }
}

unimplemented_to_pandoc!(CollectionSimple);

unimplemented_to_pandoc!(FigureSimple);

impl ToPandoc for Heading {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        pandoc::Block::Header(
            self.depth.unwrap_or(1) as i32,
            attrs_empty(),
            self.content.to_pandoc_inlines(context),
        )
    }
}

unimplemented_to_pandoc!(Include);

impl ToPandoc for List {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        let items = self
            .items
            .iter()
            .map(|item| match &item.content {
                Some(content) => match content {
                    ListItemContent::VecInlineContent(inlines) => {
                        vec![pandoc::Block::Para(inlines.to_pandoc_inlines(context))]
                    }
                    ListItemContent::VecBlockContent(blocks) => blocks.to_pandoc_blocks(context),
                },
                None => Vec::new(),
            })
            .collect();
        match &self.order {
            Some(ListOrder::Ascending) => pandoc::Block::OrderedList(
                pandoc::ListAttributes {
                    start_number: 1,
                    style: pandoc::ListNumberStyle::Decimal,
                    delim: pandoc::ListNumberDelim::DefaultDelim,
                },
                items,
            ),
            _ => pandoc::Block::BulletList(items),
        }
    }
}

unimplemented_to_pandoc!(MathBlock);

impl ToPandoc for Paragraph {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        pandoc::Block::Para(self.content.to_pandoc_inlines(context))
    }
}

impl ToPandoc for QuoteBlock {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        pandoc::Block::BlockQuote(self.content.to_pandoc_blocks(context))
    }
}

impl ToPandoc for ThematicBreak {
    fn to_pandoc_block(&self, _context: &mut EncodeContext) -> pandoc::Block {
        pandoc::Block::HorizontalRule
    }
}

impl ToPandoc for TableSimple {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
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
                                inlines.to_blocks().to_pandoc_blocks(context)
                            }
                            TableCellContent::VecBlockContent(blocks) => {
                                blocks.to_pandoc_blocks(context)
                            }
                        },
                    };
                    pandoc::Cell {
                        attr: attrs_empty(),
                        align: pandoc::Alignment::AlignDefault,
                        row_span: 1,
                        col_span: 1,
                        content: blocks,
                    }
                })
                .collect();
            let pandoc_row = pandoc::Row {
                attr: attrs_empty(),
                cells,
            };
            match row.row_type {
                Some(TableRowRowType::Header) => head.push(pandoc_row),
                Some(TableRowRowType::Footer) => foot.push(pandoc_row),
                _ => body.push(pandoc_row),
            }
        }

        pandoc::Block::Table(pandoc::Table {
            attr: attrs_empty(),
            caption: pandoc::Caption {
                short: None,
                long: vec![],
            },
            colspecs: vec![],
            head: pandoc::TableHead {
                attr: attrs_empty(),
                rows: head,
            },
            bodies: vec![pandoc::TableBody {
                attr: attrs_empty(),
                row_head_columns: 1,
                head: vec![],
                body,
            }],
            foot: pandoc::TableFoot {
                attr: attrs_empty(),
                rows: foot,
            },
        })
    }
}

impl ToPandoc for BlockContent {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        match self {
            BlockContent::Claim(node) => node.to_pandoc_block(context),
            BlockContent::CodeBlock(node) => node.to_pandoc_block(context),
            BlockContent::CodeChunk(node) => node.to_pandoc_block(context),
            BlockContent::Collection(node) => node.to_pandoc_block(context),
            BlockContent::Figure(node) => node.to_pandoc_block(context),
            BlockContent::Heading(node) => node.to_pandoc_block(context),
            BlockContent::Include(node) => node.to_pandoc_block(context),
            BlockContent::List(node) => node.to_pandoc_block(context),
            BlockContent::MathBlock(node) => node.to_pandoc_block(context),
            BlockContent::Paragraph(node) => node.to_pandoc_block(context),
            BlockContent::QuoteBlock(node) => node.to_pandoc_block(context),
            BlockContent::Table(node) => node.to_pandoc_block(context),
            BlockContent::ThematicBreak(node) => node.to_pandoc_block(context),
        }
    }
}

impl ToPandoc for [BlockContent] {
    fn to_pandoc_blocks(&self, context: &mut EncodeContext) -> Vec<pandoc::Block> {
        self.iter()
            .map(|item| item.to_pandoc_block(context))
            .collect()
    }
}

impl ToPandoc for Article {
    fn to_pandoc(&self, context: &mut EncodeContext) -> pandoc::Pandoc {
        let meta = HashMap::new();

        let blocks = self
            .content
            .as_ref()
            .map_or_else(Vec::new, |content| content.to_pandoc_blocks(context));

        pandoc::Pandoc { meta, blocks }
    }
}

impl ToPandoc for Node {
    fn to_pandoc(&self, context: &mut EncodeContext) -> pandoc::Pandoc {
        match self {
            Node::Article(node) => node.to_pandoc(context),
            _ => {
                unimplemented!(
                    "Encoding via Pandoc is not currently supported for nodes of this type"
                )
            }
        }
    }
}
