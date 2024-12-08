use std::{collections::HashMap, str::FromStr};

use jupyter_protocol::{Media, MediaType};
use nbformat::{
    parse_notebook, serialize_notebook, upgrade_legacy_notebook,
    v4::{
        Author as NotebookAuthor, Cell, CellId, CellMetadata, ErrorOutput, ExecuteResult, Metadata,
        MultilineString, Notebook as NotebookV4, Output,
    },
    Notebook,
};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        serde_json::{self, json, Map, Value},
    },
    format::Format,
    schema::{
        Article, Author, Block, CodeChunk, CodeChunkOptions, ExecutionMessage, ImageObject,
        LabelType, Node, Object, Person, RawBlock,
    },
    status::Status,
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, Losses, NodeId,
    NodeType,
};

/// A codec for Jupyter Notebooks
pub struct IpynbCodec;

#[async_trait]
impl Codec for IpynbCodec {
    fn name(&self) -> &str {
        "ipynb"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
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

    let node = Node::Article(Article {
        content,
        authors,
        ..Default::default()
    });

    Ok((node, Losses::none()))
}

/// Convert a Stencila [`Node`] to a Jupyter [`Notebook`]
fn node_to_notebook(node: &Node) -> Result<(Notebook, Losses)> {
    let Node::Article(Article {
        content, authors, ..
    }) = node
    else {
        bail!("Unable to encode a `{node}` as a notebook")
    };

    let mut cells = Vec::new();
    let mut md = String::new();
    let mut node_id = None;

    for block in content {
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
                let block_md = codec_markdown::encode(
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

    let authors = authors
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

    let metadata = Metadata {
        // TODO: Encode other article metadata
        kernelspec: None,
        language_info: None,
        authors,
        additional: HashMap::new(),
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

    let (Node::Article(Article { content, .. }), ..) = codec_markdown::decode(
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
        if let Some(md) = meta.get("caption").and_then(|value| value.as_str()) {
            if let Ok((Node::Article(Article { content, .. }), ..)) =
                codec_markdown::decode(md, None)
            {
                caption = Some(content);
            }
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
    if let Some(value) = &code_chunk.programming_language {
        stencila.insert("programmingLanguage".into(), json!(value));
    }
    if let Some(value) = &code_chunk.label_type {
        stencila.insert("labelType".into(), json!(value));
    }
    if let Some(value) = &code_chunk.label {
        stencila.insert("label".into(), json!(value));
    }
    if let Some(blocks) = &code_chunk.caption {
        let md = codec_markdown::encode(
            &Node::Article(Article::new(blocks.clone())),
            Some(EncodeOptions {
                format: Some(Format::Markdown),
                ..Default::default()
            }),
        )?
        .0;
        stencila.insert("caption".into(), json!(md));
    }

    let additional = if stencila.is_empty() {
        HashMap::new()
    } else {
        HashMap::from([("stencila".into(), Value::Object(stencila))])
    };

    let metadata = CellMetadata {
        additional,
        ..cell_metadata_default()
    };

    let outputs = code_chunk
        .outputs
        .iter()
        .flatten()
        .map(node_to_output)
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
                return image_object_from_object("application/vnd.plotly.v1+json", value)
            }
            MediaType::VegaLiteV2(value) => {
                return image_object_from_object("application/vnd.vegalite.v2+json", value)
            }
            MediaType::VegaLiteV3(value) => {
                return image_object_from_object("application/vnd.vegalite.v3+json", value)
            }
            MediaType::VegaLiteV4(value) => {
                return image_object_from_object("application/vnd.vegalite.v4+json", value)
            }
            MediaType::VegaLiteV5(value) => {
                return image_object_from_object("application/vnd.vegalite.v5+json", value)
            }
            MediaType::VegaLiteV6(value) => {
                return image_object_from_object("application/vnd.vegalite.v6+json", value)
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

/// Convert a Stencila [`Node`] to a Jupyter [`Media`]
fn node_to_output(node: &Node) -> Output {
    let media_type = match node {
        Node::String(string) => string_to_media_type(string),
        Node::ImageObject(image_object) => image_object_to_media_type(image_object),
        _ => match serde_json::to_value(node)
            .ok()
            .and_then(|value| value.as_object().cloned())
        {
            Some(object) => MediaType::Json(object),
            None => MediaType::Plain("Unable to convert".into()),
        },
    };

    Output::ExecuteResult(ExecuteResult {
        data: Media {
            content: vec![media_type],
        },
        execution_count: Default::default(),
        metadata: Default::default(),
    })
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
    let Some(media_type) = &image_object.media_type else {
        return MediaType::Png(image_object.content_url.clone());
    };

    let object = || {
        serde_json::from_str(&image_object.content_url)
            .ok()
            .and_then(|value: Value| value.as_object().cloned())
            .unwrap_or_default()
    };

    match media_type.as_str() {
        "application/vnd.plotly.v1+json" => MediaType::Plotly(object()),
        "application/vnd.vegalite.v2+json" => MediaType::VegaLiteV2(object()),
        "application/vnd.vegalite.v3+json" => MediaType::VegaLiteV3(object()),
        "application/vnd.vegalite.v4+json" => MediaType::VegaLiteV4(object()),
        "application/vnd.vegalite.v5+json" => MediaType::VegaLiteV5(object()),
        "application/vnd.vegalite.v6+json" => MediaType::VegaLiteV6(object()),
        "application/vnd.vega.v3+json" => MediaType::VegaV3(object()),
        "application/vnd.vega.v4+json" => MediaType::VegaV4(object()),
        "application/vnd.vega.v5+json" => MediaType::VegaV5(object()),
        _ => MediaType::Png(image_object.content_url.clone()),
    }
}

/// Create a Stencila [`ImageObject`] from a string
fn image_object_from_string(media_type: &str, content_url: &str) -> Node {
    Node::ImageObject(ImageObject {
        media_type: Some(media_type.into()),
        content_url: content_url.into(),
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

/// Convert a string to a Jupyter [`MediaType`]
fn string_to_media_type(string: &str) -> MediaType {
    MediaType::Plain(string.into())
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
