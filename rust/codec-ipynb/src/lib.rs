use std::collections::HashMap;

use nbformat::{
    parse_notebook, serialize_notebook, upgrade_legacy_notebook,
    v4::{Cell, CellId, CellMetadata, Metadata, Notebook as NotebookV4, Output},
    Notebook,
};

use codec::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Result},
        serde_json::{self, json, Value},
    },
    format::Format,
    schema::{Article, Block, CodeChunk, CodeChunkOptions, LabelType, Node, RawBlock},
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
            )),

            Cell::Raw {
                source, metadata, ..
            } => content.push(raw_block_from_raw_cell(source, metadata)),
        }
    }

    let node = Node::Article(Article {
        content,
        ..Default::default()
    });

    Ok((node, Losses::none()))
}

/// Convert a Stencila [`Node`] to a Jupyter [`Notebook`]
fn node_to_notebook(node: &Node) -> Result<(Notebook, Losses)> {
    let Node::Article(Article { content, .. }) = node else {
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

    let metadata = Metadata {
        // TODO: Carry over authors and other article metadata
        kernelspec: None,
        language_info: None,
        authors: None,
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
    let md = source.join("\n");

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
    // TODO: Convert outputs to Stencila nodes
    _outputs: Vec<Output>,
    metadata: CellMetadata,
    execution_count: Option<i32>,
) -> Block {
    let mut programming_language = None;
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

    Block::CodeChunk(CodeChunk {
        code: source.join("\n").into(),
        programming_language,
        label_type,
        label_automatically: label.is_some().then_some(false),
        label,
        caption,
        options: Box::new(CodeChunkOptions {
            execution_count: execution_count.map(|count| count as i64),
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

    Ok(Cell::Code {
        id: node_id_to_cell_id(code_chunk.node_id())?,
        metadata,
        execution_count: code_chunk.options.execution_count.map(|count| count as i32),
        source: vec![code_chunk.code.to_string()],
        // TODO: convert to Jupyter mime bundle
        outputs: Vec::new(),
    })
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
