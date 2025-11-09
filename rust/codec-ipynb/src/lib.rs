use std::{collections::HashMap, str::FromStr};

use jupyter_protocol::{ExecutionCount, Media, MediaType};
use nbformat::{
    Notebook, parse_notebook, serialize_notebook, upgrade_legacy_notebook,
    v4::{
        Author as NotebookAuthor, Cell, CellId, CellMetadata, ErrorOutput, ExecuteResult, Metadata,
        MultilineString, Notebook as NotebookV4, Output,
    },
};
use serde_json::{Map, Value, json};

use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, Losses, NodeId,
    NodeType, async_trait,
    eyre::{Result, bail, eyre},
    stencila_format::Format,
    stencila_schema::{
        Article, Author, Block, CodeChunk, CodeChunkOptions, ExecutionMessage, ImageObject,
        LabelType, Node, Object, Person, RawBlock,
    },
};
use stencila_codec_dom::to_dom;

/// A codec for Jupyter Notebooks
pub struct IpynbCodec;

#[async_trait]
impl Codec for IpynbCodec {
    fn name(&self) -> &str {
        "ipynb"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Ipynb => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        match format {
            Format::Ipynb => CodecSupport::LowLoss,
            _ => CodecSupport::None,
        }
    }

    fn supports_from_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::LowLoss
    }

    async fn from_str(
        &self,
        json: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let notebook = parse_notebook(json)?;

        let (node, losses) = node_from_notebook(notebook)?;

        let info = DecodeInfo {
            losses,
            ..Default::default()
        };

        Ok((node, info))
    }

    async fn to_string(
        &self,
        node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let (notebook, losses) = node_to_notebook(node)?;

        let json = serialize_notebook(&notebook)?;

        let info = EncodeInfo {
            losses,
            ..Default::default()
        };

        Ok((json, info))
    }
}

/// Convert a Jupyter [`Notebook`] to a Stencila [`Node`]
fn node_from_notebook(notebook: Notebook) -> Result<(Node, Losses)> {
    let notebook = match notebook {
        Notebook::V4(nb) => nb,
        Notebook::Legacy(nb) => upgrade_legacy_notebook(nb).map_err(|error| eyre!(error))?,
    };

    let lang = notebook
        .metadata
        .kernelspec
        .and_then(|spec| spec.language)
        .or_else(|| notebook.metadata.language_info.map(|info| info.name));

    let mut content = Vec::new();
    for cell in notebook.cells {
        match cell {
            Cell::Markdown {
                source,
                metadata,
                attachments,
                ..
            } => content.append(&mut blocks_from_markdown_cell(
                source,
                metadata,
                attachments,
            )?),

            Cell::Code {
                source,
                outputs,
                metadata,
                execution_count,
                ..
            } => content.push(code_chunk_from_code_cell(
                source,
                outputs,
                metadata,
                execution_count,
                lang.clone(),
            )),

            Cell::Raw {
                source, metadata, ..
            } => content.push(raw_block_from_raw_cell(source, metadata)),
        }
    }

    let authors = notebook.metadata.authors.and_then(|authors| {
        let authors: Vec<Author> = authors
            .into_iter()
            .flat_map(|author| match Person::from_str(&author.name) {
                Ok(person) => Some(Author::Person(person)),
                _ => None,
            })
            .collect();

        (!authors.is_empty()).then_some(authors)
    });

    // Get any Stencila metadata
    let mut id = None;
    if let Some(stencila) = notebook.metadata.additional.get("stencila") {
        id = stencila
            .get("id")
            .and_then(|id| id.as_str())
            .map(String::from);
    };

    let node = Node::Article(Article {
        content,
        authors,
        id,
        ..Default::default()
    });

    Ok((node, Losses::none()))
}

/// Convert a Stencila [`Node`] to a Jupyter [`Notebook`]
fn node_to_notebook(node: &Node) -> Result<(Notebook, Losses)> {
    let Node::Article(article) = node else {
        bail!("Unable to encode a `{node}` as a notebook")
    };

    let mut cells = Vec::new();
    let mut md = String::new();
    let mut node_id = None;

    for block in &article.content {
        match block {
            Block::CodeChunk(..) | Block::RawBlock(..) => {
                let cell = match block {
                    Block::CodeChunk(code_chunk) => code_chunk_to_code_cell(code_chunk)?,
                    Block::RawBlock(raw_block) => raw_block_to_raw_cell(raw_block)?,
                    _ => unreachable!(),
                };
                if !md.is_empty() {
                    cells.push(Cell::Markdown {
                        id: node_id_to_cell_id(node_id.unwrap_or_else(NodeId::null))?,
                        source: vec![md.clone()],
                        metadata: cell_metadata_default(),
                        attachments: None,
                    });
                    md.clear();
                    node_id = None;
                }
                cells.push(cell);
            }
            block => {
                let block_md = stencila_codec_markdown::encode(
                    // Treat as an article so that footnotes are encoded
                    &Node::Article(Article::new(vec![block.clone()])),
                    Some(EncodeOptions {
                        format: Some(Format::Myst),
                        ..Default::default()
                    }),
                )?
                .0;

                if !md.is_empty() {
                    md.push('\n');
                }
                md += &block_md;

                node_id = block.node_id()
            }
        }
    }

    if !md.is_empty() {
        cells.push(Cell::Markdown {
            id: node_id_to_cell_id(node_id.unwrap_or_else(NodeId::null))?,
            source: vec![md],
            metadata: cell_metadata_default(),
            attachments: None,
        });
    }

    let authors = article
        .authors
        .iter()
        .flatten()
        .map(|author| match author {
            Author::Person(person) => Some(NotebookAuthor {
                name: person.as_string(),
                additional: HashMap::new(),
            }),
            _ => None,
        })
        .collect();

    // Create a Stencila metadata object
    let mut stencila = HashMap::new();
    if let Some(id) = &article.id {
        stencila.insert("id", json!(id));
    }
    let stencila = serde_json::to_value(stencila)?;

    let metadata = Metadata {
        kernelspec: None,
        language_info: None,
        authors,
        additional: HashMap::from([("stencila".to_string(), stencila)]),
    };

    let notebook = Notebook::V4(NotebookV4 {
        cells,
        metadata,
        nbformat: 4,
        nbformat_minor: 5,
    });

    Ok((notebook, Losses::none()))
}

/// Convert a Jupyter Markdown cell to Stencila [`Block`]s
fn blocks_from_markdown_cell(
    source: Vec<String>,
    // TODO: Use these?
    _metadata: CellMetadata,
    _attachments: Option<Value>,
) -> Result<Vec<Block>> {
    let md = source.join("");

    let (Node::Article(Article { content, .. }), ..) = stencila_codec_markdown::decode(
        &md,
        Some(DecodeOptions {
            format: Some(Format::Myst),
            ..Default::default()
        }),
    )?
    else {
        bail!("Expected an Article")
    };

    Ok(content)
}

/// Convert a Jupyter code cell to a Stencila [`CodeChunk`]
fn code_chunk_from_code_cell(
    source: Vec<String>,
    outputs: Vec<Output>,
    metadata: CellMetadata,
    execution_count: Option<i32>,
    mut programming_language: Option<String>,
) -> Block {
    let mut nodes = Vec::new();
    let mut errors = Vec::new();
    for output in outputs {
        match output {
            Output::ExecuteResult(result) => nodes.push(node_from_media(result.data)),
            Output::DisplayData(data) => nodes.push(node_from_media(data.data)),
            Output::Stream { name, text } => match name.as_str() {
                "stderr" => errors.push(execution_message_from_stream(text)),
                _ => nodes.push(node_from_multiline_string(text)),
            },
            Output::Error(error) => errors.push(execution_message_from_error_output(error)),
        }
    }

    let mut label_type = None;
    let mut label = None;
    let mut caption = None;

    if let Some(meta) = metadata
        .additional
        .get("stencila")
        .and_then(|value| value.as_object())
    {
        programming_language = meta
            .get("programmingLanguage")
            .and_then(|value| value.as_str())
            .map(String::from);
        label_type = meta
            .get("labelType")
            .and_then(|value| value.as_str())
            .and_then(|value| LabelType::try_from(value).ok());
        label = meta
            .get("label")
            .and_then(|value| value.as_str())
            .map(String::from);
        if let Some(md) = meta.get("caption").and_then(|value| value.as_str())
            && let Ok((Node::Article(Article { content, .. }), ..)) =
                stencila_codec_markdown::decode(md, None)
        {
            caption = Some(content);
        }
    }

    if let Some(meta) = metadata
        .additional
        .get("vscode")
        .and_then(|value| value.as_object())
    {
        programming_language = meta
            .get("languageId")
            .and_then(|value| value.as_str())
            .map(String::from);
    }

    Block::CodeChunk(CodeChunk {
        code: source.join("\n").into(),
        programming_language,
        label_type,
        label_automatically: label.is_some().then_some(false),
        label,
        caption,
        outputs: (!nodes.is_empty()).then_some(nodes),
        options: Box::new(CodeChunkOptions {
            execution_count: execution_count.map(|count| count as i64),
            execution_messages: (!errors.is_empty()).then_some(errors),
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Convert a Stencila [`CodeChunk`] to a Jupyter code cell
fn code_chunk_to_code_cell(code_chunk: &CodeChunk) -> Result<Cell> {
    let mut stencila = serde_json::Map::new();
    let mut vscode = serde_json::Map::new();
    if let Some(value) = &code_chunk.programming_language {
        stencila.insert("programmingLanguage".into(), json!(value));
        vscode.insert("languageId".into(), json!(value));
    }
    if let Some(value) = &code_chunk.label_type {
        stencila.insert("labelType".into(), json!(value));
    }
    if let Some(value) = &code_chunk.label {
        stencila.insert("label".into(), json!(value));
    }
    if let Some(blocks) = &code_chunk.caption {
        let md = stencila_codec_markdown::encode(
            &Node::Article(Article::new(blocks.clone())),
            Some(EncodeOptions {
                format: Some(Format::Markdown),
                ..Default::default()
            }),
        )?
        .0;
        stencila.insert("caption".into(), json!(md));
    }

    let additional = if stencila.is_empty() && vscode.is_empty() {
        HashMap::new()
    } else {
        HashMap::from([
            ("stencila".into(), Value::Object(stencila)),
            ("vscode".into(), Value::Object(vscode)),
        ])
    };

    let metadata = CellMetadata {
        additional,
        ..cell_metadata_default()
    };

    let execution_count = code_chunk
        .options
        .execution_count
        .unwrap_or_default()
        .min(1) as usize;

    let outputs = code_chunk
        .outputs
        .iter()
        .flatten()
        .map(|output| node_to_output(output, execution_count))
        .collect();

    Ok(Cell::Code {
        id: node_id_to_cell_id(code_chunk.node_id())?,
        metadata,
        execution_count: code_chunk.options.execution_count.map(|count| count as i32),
        source: vec![code_chunk.code.to_string()],
        outputs,
    })
}

/// Convert a Jupyter [`Media`] to a Stencila [`Node`]
fn node_from_media(media: Media) -> Node {
    // First, try to convert to an interactive plot
    for media_type in &media.content {
        match media_type {
            MediaType::Plotly(value) => {
                return image_object_from_object("application/vnd.plotly.v1+json", value);
            }
            MediaType::VegaLiteV2(value) => {
                return image_object_from_object("application/vnd.vegalite.v2+json", value);
            }
            MediaType::VegaLiteV3(value) => {
                return image_object_from_object("application/vnd.vegalite.v3+json", value);
            }
            MediaType::VegaLiteV4(value) => {
                return image_object_from_object("application/vnd.vegalite.v4+json", value);
            }
            MediaType::VegaLiteV5(value) => {
                return image_object_from_object("application/vnd.vegalite.v5+json", value);
            }
            MediaType::VegaLiteV6(value) => {
                return image_object_from_object("application/vnd.vegalite.v6+json", value);
            }
            _ => {}
        }
    }

    // Second, try to convert to a static image
    for media_type in &media.content {
        match media_type {
            MediaType::Svg(value) => return image_object_from_string("image/svg+xml", value),
            MediaType::Png(value) => return image_object_from_string("image/png", value),
            MediaType::Jpeg(value) => return image_object_from_string("image/jpeg", value),
            MediaType::Gif(value) => return image_object_from_string("image/gif", value),
            _ => {}
        }
    }

    // Fallbacks
    for media_type in media.content {
        match media_type {
            MediaType::Plain(value) => return Node::String(value),

            // TODO: Parse these
            MediaType::Html(value)
            | MediaType::Latex(value)
            | MediaType::Javascript(value)
            | MediaType::Markdown(value) => return Node::String(value),

            // TODO: Consider parsing some of these
            MediaType::Json(value)
            | MediaType::GeoJson(value)
            | MediaType::WidgetView(value)
            | MediaType::WidgetState(value)
            | MediaType::VegaV3(value)
            | MediaType::VegaV4(value)
            | MediaType::VegaV5(value)
            | MediaType::Vdom(value) => return object_from_value(value),

            _ => {}
        }
    }

    Node::String("Unhandled media type".into())
}

/// Convert a Stencila [`Node`] to a Jupyter [`Output`]
fn node_to_output(node: &Node, execution_count: usize) -> Output {
    Output::ExecuteResult(ExecuteResult {
        data: Media {
            content: node_to_media_types(node),
        },
        execution_count: ExecutionCount::new(execution_count),
        metadata: Default::default(),
    })
}

/// Convert a Stencila [`Node`] to a vector of [`MediaType`]s
///
/// This function returns multiple media type representations for rich display in Jupyter.
/// The order matters: Jupyter frontends will typically use the first format they can render.
fn node_to_media_types(node: &Node) -> Vec<MediaType> {
    let mut media_types = Vec::new();

    // Handle each node type with appropriate media representations
    match node {
        // Primitive types: prioritize plain text for readability, add JSON for roundtripping
        Node::Null(null) => {
            media_types.push(MediaType::Plain(null.to_string()));
        }
        Node::Boolean(bool) => {
            media_types.push(MediaType::Plain(bool.to_string()));
        }
        Node::Integer(int) => {
            media_types.push(MediaType::Plain(int.to_string()));
        }
        Node::UnsignedInteger(uint) => {
            media_types.push(MediaType::Plain(uint.to_string()));
        }
        Node::Number(num) => {
            media_types.push(MediaType::Plain(num.to_string()));
        }
        Node::String(string) => {
            media_types.push(MediaType::Plain(string.clone()));
        }

        // Date/Time types: use plain text ISO format, add JSON
        Node::Date(date) => {
            media_types.push(MediaType::Plain(date.to_string()));
            add_json_media_type(&mut media_types, node);
        }
        Node::DateTime(datetime) => {
            media_types.push(MediaType::Plain(datetime.to_string()));
            add_json_media_type(&mut media_types, node);
        }
        Node::Time(time) => {
            media_types.push(MediaType::Plain(time.to_string()));
            add_json_media_type(&mut media_types, node);
        }
        Node::Timestamp(timestamp) => {
            media_types.push(MediaType::Plain(timestamp.to_string()));
            add_json_media_type(&mut media_types, node);
        }
        Node::Duration(duration) => {
            media_types.push(MediaType::Plain(duration.to_string()));
            add_json_media_type(&mut media_types, node);
        }

        // Media object: because these tend to be large, only encode a
        // single media type (rather than say HTML and JSON)
        Node::ImageObject(image_object) => {
            media_types.push(image_object_to_media_type(image_object));
        }
        Node::AudioObject(_audio_object) => {
            // TODO: Add native audio media type if supported by jupyter_protocol
            media_types.push(MediaType::Html(to_dom(node)));
        }
        Node::VideoObject(_video_object) => {
            // TODO: Add native video media type if supported by jupyter_protocol
            media_types.push(MediaType::Html(to_dom(node)));
        }

        // Math expressions: use LaTeX for native rendering
        Node::MathBlock(math_block) => {
            media_types.push(MediaType::Latex(math_block.code.to_string()));
            media_types.push(MediaType::Html(to_dom(node)));
            add_json_media_type(&mut media_types, node);
        }
        Node::MathInline(math_inline) => {
            media_types.push(MediaType::Latex(math_inline.code.to_string()));
            media_types.push(MediaType::Html(to_dom(node)));
            add_json_media_type(&mut media_types, node);
        }

        // All other types: provide HTML and JSON
        _ => {
            media_types.push(MediaType::Html(to_dom(node)));
            add_json_media_type(&mut media_types, node);
        }
    }

    // Fallback if no media types were added
    if media_types.is_empty() {
        media_types.push(MediaType::Plain(format!(
            "Unable to convert Stencila `{node}` to Jupyter output"
        )));
    }

    media_types
}

/// Add a JSON media type representation of a node to the media types vector
fn add_json_media_type(media_types: &mut Vec<MediaType>, node: &Node) {
    if let Ok(value) = serde_json::to_value(node)
        && let Some(object) = value.as_object().cloned()
    {
        media_types.push(MediaType::Json(object));
    }
}

/// Create a Stencila [`ImageObject`] from a JSON object
fn image_object_from_object(media_type: &str, object: &Map<String, Value>) -> Node {
    Node::ImageObject(ImageObject {
        media_type: Some(media_type.into()),
        content_url: serde_json::to_string(&object).unwrap_or_default(),
        ..Default::default()
    })
}

/// Convert a Stencila [`ImageObject`] to a Jupyter [`MediaType`]
fn image_object_to_media_type(image_object: &ImageObject) -> MediaType {
    let url = &image_object.content_url;

    // Determine the media type of the image, falling back to PNG
    let media_type = match &image_object.media_type {
        Some(media_type) => media_type,
        None => {
            if let (Some(start), Some(end)) = (url.find("data:"), url.find(";base64,")) {
                &url[(start + 5)..end]
            } else {
                "image/png"
            }
        }
    };

    // Deserialize a visualization spec to an JSON object
    let object = || {
        serde_json::from_str(&url)
            .ok()
            .and_then(|value: Value| value.as_object().cloned())
            .unwrap_or_default()
    };

    match media_type {
        "application/vnd.plotly.v1+json" => MediaType::Plotly(object()),
        "application/vnd.vegalite.v2+json" => MediaType::VegaLiteV2(object()),
        "application/vnd.vegalite.v3+json" => MediaType::VegaLiteV3(object()),
        "application/vnd.vegalite.v4+json" => MediaType::VegaLiteV4(object()),
        "application/vnd.vegalite.v5+json" => MediaType::VegaLiteV5(object()),
        "application/vnd.vegalite.v6+json" => MediaType::VegaLiteV6(object()),
        "application/vnd.vega.v3+json" => MediaType::VegaV3(object()),
        "application/vnd.vega.v4+json" => MediaType::VegaV4(object()),
        "application/vnd.vega.v5+json" => MediaType::VegaV5(object()),
        "image/svg+xml" => MediaType::Svg(url.into()),
        "image/png" => MediaType::Png(url.into()),
        "image/jpeg" => MediaType::Jpeg(url.into()),
        "image/gif" => MediaType::Gif(url.into()),
        _ => MediaType::Png(url.clone()),
    }
}

/// Create a Stencila [`ImageObject`] from a string value
///
/// Note that Jupyter stores the Base64 encoding of images without the dataURI
/// so this function add them back in.
fn image_object_from_string(media_type: &str, content_url: &str) -> Node {
    Node::ImageObject(ImageObject {
        media_type: Some(media_type.into()),
        content_url: ["data:", media_type, ";base64,", content_url].concat(),
        ..Default::default()
    })
}

/// Create a Stencila [`Object`] from a JSON object
fn object_from_value(object: Map<String, Value>) -> Node {
    Node::Object(Object(
        object
            .into_iter()
            .map(|(key, value)| (key, serde_json::from_value(value).unwrap_or_default()))
            .collect(),
    ))
}

/// Convert a Jupyter code cell stream output to a Stencila [`Node`]
fn node_from_multiline_string(text: MultilineString) -> Node {
    Node::String(text.0)
}

/// Convert a Jupyter code cell stream output to a Stencila [`ExecutionMessage`]
fn execution_message_from_stream(text: MultilineString) -> ExecutionMessage {
    ExecutionMessage {
        message: text.0,
        ..Default::default()
    }
}

/// Convert a Jupyter code cell [`ErrorOutput`] to a Stencila [`ExecutionMessage`]
fn execution_message_from_error_output(error: ErrorOutput) -> ExecutionMessage {
    ExecutionMessage {
        message: error.evalue,
        error_type: (!error.ename.is_empty()).then_some(error.ename),
        stack_trace: (!error.traceback.is_empty()).then(|| error.traceback.join("\n")),
        ..Default::default()
    }
}

/// Convert a Jupyter raw block to a Stencila [`RawBlock`]
fn raw_block_from_raw_cell(source: Vec<String>, metadata: CellMetadata) -> Block {
    Block::RawBlock(RawBlock {
        content: source.join("\n").into(),
        format: metadata.format.unwrap_or_default(),
        ..Default::default()
    })
}

/// Convert a Stencila [`RawBlock`] to a Jupyter raw block
fn raw_block_to_raw_cell(raw_block: &RawBlock) -> Result<Cell> {
    Ok(Cell::Raw {
        id: node_id_to_cell_id(raw_block.node_id())?,
        source: vec![raw_block.content.to_string()],
        metadata: CellMetadata {
            format: Some(raw_block.format.clone()),
            ..cell_metadata_default()
        },
    })
}

/// Convert a Stencila [`NodeId`] to a Jupyter [`CellId`]
fn node_id_to_cell_id(node_id: NodeId) -> Result<CellId> {
    CellId::new(&node_id.to_string()).map_err(|error| eyre!(error))
}

/// Create a default Jupyter [`CellMetadata`]
fn cell_metadata_default() -> CellMetadata {
    CellMetadata {
        id: None,
        collapsed: None,
        scrolled: None,
        deletable: None,
        editable: None,
        format: None,
        name: None,
        tags: None,
        jupyter: None,
        execution: None,
        additional: HashMap::new(),
    }
}
