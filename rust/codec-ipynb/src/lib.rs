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
        serde_json::Value,
    },
    format::Format,
    schema::{Article, Block, CodeChunk, CodeChunkOptions, Node, RawBlock},
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
                        id: cell_id_from_node_id(node_id.unwrap_or_else(|| NodeId::null()))?,
                        source: string_to_multiline_vec(&md),
                        metadata: cell_metadata_default(),
                        attachments: None,
                    });
                    md.clear();
                    node_id = None;
                }
                cells.push(cell);
            }
            block => {
                md += &codec_markdown::to_markdown(block);
                node_id = block.node_id()
            }
        }
    }

    if !md.is_empty() {
        cells.push(Cell::Markdown {
            id: cell_id_from_node_id(node_id.unwrap_or_else(|| NodeId::null()))?,
            source: string_to_multiline_vec(&md),
            metadata: cell_metadata_default(),
            attachments: None,
        });
    }

    let mut metadata = Metadata {
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

fn blocks_from_markdown_cell(
    source: Vec<String>,
    metadata: CellMetadata,
    attachments: Option<Value>,
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

fn code_chunk_from_code_cell(
    source: Vec<String>,
    outputs: Vec<Output>,
    metadata: CellMetadata,
    execution_count: Option<i32>,
) -> Block {
    Block::CodeChunk(CodeChunk {
        code: source.join("\n").into(),
        options: Box::new(CodeChunkOptions {
            execution_count: execution_count.map(|count| count as i64),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn code_chunk_to_code_cell(code_chunk: &CodeChunk) -> Result<Cell> {
    Ok(Cell::Code {
        id: cell_id_from_node_id(code_chunk.node_id())?,
        metadata: cell_metadata_default(), // TODO
        execution_count: code_chunk.options.execution_count.map(|count| count as i32),
        source: code_chunk
            .code
            .to_string()
            .split('\n')
            .map(String::from)
            .collect(),
        outputs: Vec::new(), // TODO
    })
}

fn raw_block_from_raw_cell(source: Vec<String>, metadata: CellMetadata) -> Block {
    Block::RawBlock(RawBlock {
        content: source.join("\n").into(),
        format: metadata.format.unwrap_or_default(),
        ..Default::default()
    })
}

fn raw_block_to_raw_cell(raw_block: &RawBlock) -> Result<Cell> {
    Ok(Cell::Raw {
        id: cell_id_from_node_id(raw_block.node_id())?,
        source: raw_block
            .content
            .to_string()
            .split('\n')
            .map(String::from)
            .collect(),
        metadata: CellMetadata {
            format: Some(raw_block.format.clone()),
            ..cell_metadata_default()
        },
    })
}

fn string_to_multiline_vec(string: &str) -> Vec<String> {
    string.split('\n').map(String::from).collect()
}

fn cell_id_from_node_id(node_id: NodeId) -> Result<CellId> {
    CellId::new(&node_id.to_string()).map_err(|error| eyre!(error))
}

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
