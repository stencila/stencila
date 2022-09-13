use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use cloud::nodes::{node_create, node_url};
use node_dispatch::{dispatch_block, dispatch_inline};
use pandoc_types::definition as pandoc;

use codec::{
    common::{eyre::Result, futures, itertools::Itertools, tempfile},
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
    options: Option<EncodeOptions>,
) -> Result<String> {
    let mut context = EncodeContext::new(options)?;
    let pandoc = node.to_pandoc(&mut context);
    context.finalize().await?;
    to_pandoc(pandoc, path, format, args).await
}

/// Encode a `Node` to a Pandoc document
///
/// Compared to the `encode` function this function does not spawn a Pandoc
/// process, or create RPNGs and returns a `pandoc_types` definition instead.
/// It intended mainly for generative testing.
pub fn encode_node(node: &Node, options: Option<EncodeOptions>) -> Result<pandoc::Pandoc> {
    let mut context = EncodeContext::new(options)?;
    let pandoc = node.to_pandoc(&mut context);
    Ok(pandoc)
}

/// The encoding context.
struct EncodeContext {
    /// Encoding options
    options: EncodeOptions,

    /// The directory where any temporary files are placed
    ///
    /// Note: this will get cleaned up when the context is destroyed (ie. encoding finished)
    temp_dir: tempfile::TempDir,

    /// The nodes that should be encoded as RPNGs
    rpng_nodes: Vec<(String, PathBuf, Node)>,
}

impl EncodeContext {
    /// Create a new encoding context
    fn new(options: Option<EncodeOptions>) -> Result<Self> {
        let options = options.unwrap_or_default();
        let temp_dir = tempfile::tempdir()?;
        let rpng_nodes = Vec::new();
        Ok(EncodeContext {
            options,
            temp_dir,
            rpng_nodes,
        })
    }

    /// Should a node type be encoded as an RPNG, based on provided options
    fn should_rpng(&mut self, node_type: &str) -> bool {
        !self.options.lossy && self.options.rpng_types.contains(&node_type.to_string())
    }

    /// Push a node to be encoded as an RPNG
    fn push_rpng(&mut self, type_name: &str, node: Node) -> pandoc::Inline {
        let key = key_utils::generate("snk");
        let url = node_url(&key);
        let path = self.temp_dir.path().join(format!("{}.png", key));

        self.rpng_nodes.push((key, path.clone(), node.clone()));

        let inlines = if self.options.rpng_text {
            // Generate compact JSON for the node
            let json = JsonCodec::to_string(
                &node,
                Some(EncodeOptions {
                    compact: true,
                    ..Default::default()
                }),
            )
            .expect("Should be able to encode as JSON");

            // If the JSON is small enough (e.g 5000 is the limit
            // for image alt text in Google Docs) then use it, otherwise
            // if rpng_link is enabled, use its URL.
            if json.len() <= 5000 {
                vec![pandoc::Inline::Str(json)]
            } else if self.options.rpng_link {
                vec![pandoc::Inline::Str([&url, "/json"].concat())]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let title = type_name.to_string();

        let image = pandoc::Inline::Image(
            attrs_empty(),
            inlines,
            pandoc::Target {
                url: path.to_slash_lossy().to_string(),
                title: title.clone(),
            },
        );

        if self.options.rpng_link {
            pandoc::Inline::Link(attrs_empty(), vec![image], pandoc::Target { url, title })
        } else {
            image
        }
    }

    /// Generate all the RPNGs that have been pushed to this context
    ///
    /// This is called once, after all nodes have been encoded, thereby avoiding having to have all
    /// async functions, as well as being faster.
    async fn generate_rpngs(&self) -> Result<()> {
        let nodes: Vec<&Node> = self.rpng_nodes.iter().map(|(.., node)| node).collect();
        let rpngs = codec_rpng::nodes_to_bytes(&nodes, None).await?;
        for (index, bytes) in rpngs.iter().enumerate() {
            let (_key, path, ..) = &self.rpng_nodes[index];
            std::fs::write(path, bytes)?;
        }
        Ok(())
    }

    /// Post all the RPNGs to Stencila Cloud
    async fn post_nodes(&self) -> Result<()> {
        let futures = self
            .rpng_nodes
            .iter()
            .map(|(key, .., node)| node_create(key, node, None, None));
        futures::future::try_join_all(futures).await?;
        Ok(())
    }

    /// Finalize the encoding
    async fn finalize(&self) -> Result<()> {
        self.generate_rpngs().await?;
        if self.options.rpng_link {
            self.post_nodes().await?;
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

macro_rules! inline_with_value_to_pandoc_str {
    ($type:ty) => {
        impl ToPandoc for $type {
            fn to_pandoc_inline(&self, _context: &mut EncodeContext) -> pandoc::Inline {
                pandoc::Inline::Str(self.value.to_string())
            }
        }
    };
}

inline_with_value_to_pandoc_str!(Date);
inline_with_value_to_pandoc_str!(Time);
inline_with_value_to_pandoc_str!(DateTime);
inline_with_value_to_pandoc_str!(Timestamp);
inline_with_value_to_pandoc_str!(Duration);

macro_rules! inline_content_to_pandoc_inline {
    ($type:ty, $pandoc:expr) => {
        impl ToPandoc for $type {
            fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
                $pandoc(self.content.to_pandoc_inlines(context))
            }
        }
    };
}

inline_content_to_pandoc_inline!(Emphasis, pandoc::Inline::Emph);
inline_content_to_pandoc_inline!(Strikeout, pandoc::Inline::Strikeout);
inline_content_to_pandoc_inline!(Delete, pandoc::Inline::Strikeout);
inline_content_to_pandoc_inline!(Strong, pandoc::Inline::Strong);
inline_content_to_pandoc_inline!(Subscript, pandoc::Inline::Subscript);
inline_content_to_pandoc_inline!(Superscript, pandoc::Inline::Superscript);
inline_content_to_pandoc_inline!(Underline, pandoc::Inline::Underline);
inline_content_to_pandoc_inline!(NontextualAnnotation, pandoc::Inline::Underline);

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
    /// Encode a `CodeExpression` to a Pandoc inline element
    ///
    /// Lossless encoding represents the entire expression as an RPNG.
    /// Lossy encoding represents the expression `output`, falling back to `text` if no output.
    fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
        if context.should_rpng("CodeExpression") {
            context.push_rpng("CodeExpression", Node::CodeExpression(self.clone()))
        } else {
            self.output
                .as_ref()
                .map(|node| node.to_inline())
                .map(|inline| inline.to_pandoc_inline(context))
                .unwrap_or_else(|| pandoc::Inline::Code(attrs_empty(), self.text.clone()))
        }
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
    fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
        if context.should_rpng("MathFragment") {
            context.push_rpng("MathFragment", Node::MathFragment(self.clone()))
        } else {
            pandoc::Inline::Math(pandoc::MathType::InlineMath, self.text.clone())
        }
    }
}

unimplemented_to_pandoc!(Note);

impl ToPandoc for Parameter {
    /// Encode a `Parameter` to a Pandoc inline element
    ///
    /// Lossless encoding represents the entire parameter as an RPNG.
    /// Lossy encoding represents the `value` (or `default`) only.
    fn to_pandoc_inline(&self, context: &mut EncodeContext) -> pandoc::Inline {
        if context.should_rpng("Parameter") {
            context.push_rpng("Parameter", Node::Parameter(self.clone()))
        } else {
            self.value
                .as_ref()
                .or(self.default.as_ref())
                .map(|node| node.to_inline())
                .map(|inline| inline.to_pandoc_inline(context))
                .unwrap_or_else(|| pandoc::Inline::Str("".to_string()))
        }
    }
}

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
        dispatch_inline!(self, to_pandoc_inline, context)
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
    /// Lossless encoding represents the entire code chunk as an RPNG.
    /// Any label or caption are put in a paragraph after the code chunk, rather than as screenshotted content.
    /// This allows them to be endit. Note that these are re-constituted into the code chunk in the reshape function.
    ///
    /// Lossy encoding represents `outputs` (if any) only. Each output is represented as a Pandoc block
    /// element (e.g. an image wrapped in a paragraph)
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        if context.should_rpng("CodeChunk") {
            let mut stripped = self.clone();
            stripped.label = None;
            stripped.caption = None;

            let image = context.push_rpng("CodeChunk", Node::CodeChunk(stripped));
            let image_para = pandoc::Block::Para(vec![image]);

            let CodeChunk { label, caption, .. } = self;
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
                            let mut blocks_as_inlines =
                                blocks.to_inlines().to_pandoc_inlines(context);
                            inlines.append(&mut blocks_as_inlines);
                        }
                    };
                }
                vec![image_para, pandoc::Block::Para(inlines)]
            } else {
                vec![image_para]
            };

            pandoc::Block::Div(attrs_empty(), blocks)
        } else {
            match &self.outputs {
                Some(nodes) => pandoc::Block::Div(
                    attrs_empty(),
                    nodes
                        .iter()
                        .map(|node| node.to_block())
                        .map(|node| node.to_pandoc_block(context))
                        .collect_vec(),
                ),
                None => pandoc::Block::Null,
            }
        }
    }
}

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

impl ToPandoc for Include {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        let mut blocks = vec![pandoc::Block::Para(vec![context.push_rpng(
            "Include",
            Node::Include(Include {
                content: None,
                ..self.clone()
            }),
        )])];

        blocks.append(
            &mut self
                .content
                .as_ref()
                .map(|blocks| blocks.to_pandoc_blocks(context))
                .unwrap_or_default(),
        );

        pandoc::Block::Div(attrs_empty(), blocks)
    }
}

impl ToPandoc for Call {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        let mut blocks = vec![pandoc::Block::Para(vec![context.push_rpng(
            "Call",
            Node::Call(Call {
                content: None,
                ..self.clone()
            }),
        )])];

        blocks.append(
            &mut self
                .content
                .as_ref()
                .map(|blocks| blocks.to_pandoc_blocks(context))
                .unwrap_or_default(),
        );

        pandoc::Block::Div(attrs_empty(), blocks)
    }
}

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

impl ToPandoc for MathBlock {
    fn to_pandoc_block(&self, context: &mut EncodeContext) -> pandoc::Block {
        let inline = if context.should_rpng("MathBlock") {
            context.push_rpng("MathBlock", Node::MathBlock(self.clone()))
        } else {
            pandoc::Inline::Math(pandoc::MathType::DisplayMath, self.text.clone())
        };
        pandoc::Block::Para(vec![inline])
    }
}

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
        let mut cols = 0;
        for row in &self.rows {
            if row.cells.len() > cols {
                cols = row.cells.len();
            }
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

        let colspecs = (0..cols)
            .map(|_| pandoc::ColSpec {
                ..Default::default()
            })
            .collect_vec();

        pandoc::Block::Table(pandoc::Table {
            attr: attrs_empty(),
            caption: pandoc::Caption {
                short: None,
                long: vec![],
            },
            colspecs,
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
        dispatch_block!(self, to_pandoc_block, context)
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
