use std::collections::HashMap;

use codec::{
    common::{
        base64,
        eyre::{bail, Result},
        serde::Serialize,
        serde_json::{self, json},
    },
    stencila_schema::*,
    EncodeOptions,
};
use codec_md::ToMd;
use codec_txt::ToTxt;

/// Encode a `Node` to a Jupyter Notebook.
///
/// Note that the order of properties in various JSON objects is
/// consistent with Jupyter and other tools. Also, the JSON is
/// pretty printed with a one space indentation.
pub fn encode(node: &Node) -> Result<String> {
    let article = match node {
        Node::Article(article) => article,
        _ => bail!("Only able to encode an `Article` as a Jupyter Notebook`"),
    };

    let notebook = json!({
        "cells": encode_content(&article.content),
        "metadata": encode_metadata(article),
        "nbformat": 4,
        "nbformat_minor": 5
    });

    let buffer = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
    let mut serializer = serde_json::Serializer::with_formatter(buffer, formatter);
    notebook.serialize(&mut serializer).unwrap();
    let json = String::from_utf8(serializer.into_inner())?;

    Ok(json)
}

/// Encode `Article` properties, other than `content` as a JSON object
///
/// The nbformat-v4 schema requires that `title` be a string
/// and that `authors` be an array of objects with a `name` string
/// property (and optional additional properties). This function
/// conforms to those requirements.
fn encode_metadata(article: &Article) -> serde_json::Value {
    let mut metadata = serde_json::to_value(article).expect("Should serialize");
    let object = metadata.as_object_mut().expect("Should be object");
    object.remove("type");
    object.remove("content");

    if let Some(title) = article.title.as_deref() {
        let title = title.to_txt();
        object.insert("title".to_string(), json!(title));
    }

    if let Some(authors) = article.authors.as_ref() {
        let authors = authors
            .iter()
            .map(|author| {
                let name = match author {
                    CreativeWorkAuthors::Person(person) => {
                        let Person {
                            name,
                            given_names,
                            family_names,
                            ..
                        } = person;
                        if let Some(name) = name.as_deref() {
                            name.clone()
                        } else {
                            [
                                &given_names
                                    .as_ref()
                                    .map(|names| names.join(" "))
                                    .unwrap_or_default(),
                                " ",
                                &family_names
                                    .as_ref()
                                    .map(|names| names.join(" "))
                                    .unwrap_or_default(),
                            ]
                            .concat()
                            .trim()
                            .to_string()
                        }
                    }
                    CreativeWorkAuthors::Organization(org) => {
                        if let Some(name) = org.name.as_deref() {
                            name.clone()
                        } else {
                            "Unnamed".to_string()
                        }
                    }
                };
                let mut value = serde_json::to_value(author).expect("Should serialize");
                let object = value.as_object_mut().expect("Should be object");
                object.insert("name".to_string(), json!(name));
                value
            })
            .collect::<Vec<serde_json::Value>>();
        object.insert("authors".to_string(), json!(authors));
    }

    metadata
}

/// Encode a `Article`'s content as a vector of Jupyter cells
fn encode_content(content: &Option<Vec<BlockContent>>) -> Vec<serde_json::Value> {
    let blocks = if let Some(blocks) = content {
        blocks
    } else {
        return Vec::new();
    };

    let mut cells = Vec::with_capacity(blocks.len());
    let mut content = Vec::new();
    for block in blocks {
        match block {
            BlockContent::CodeChunk(chunk) => {
                if !content.is_empty() {
                    cells.push(encode_markdown(&content));
                    content.clear()
                }
                cells.push(encode_chunk(chunk));
            }
            _ => content.push(block.clone()),
        }
    }
    if !content.is_empty() {
        cells.push(encode_markdown(&content))
    }

    cells
}

/// Encode a `CodeChunk` to a Jupyter code cell
fn encode_chunk(chunk: &CodeChunk) -> serde_json::Value {
    let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();

    if let Some(id) = chunk.id.as_ref() {
        metadata.insert("id".to_string(), json!(*id));
    }

    if let Some(label) = chunk.label.as_ref() {
        metadata.insert("label".to_string(), json!(*label));
    }

    if let Some(caption) = chunk.caption.as_deref() {
        let caption = match caption {
            CodeChunkCaption::String(string) => string.clone(),
            CodeChunkCaption::VecBlockContent(blocks) => blocks.to_md(&EncodeOptions::default()),
        };
        metadata.insert("caption".to_string(), json!(caption));
    }

    if !chunk.programming_language.is_empty() {
        metadata.insert(
            "language_info".to_string(),
            json!({
                "name": chunk.programming_language
            }),
        );
    }

    let source = encode_multiline_string(&chunk.code);

    let outputs = if let Some(outputs) = &chunk.outputs {
        encode_outputs(outputs)
    } else {
        Vec::new()
    };

    json!({
        "cell_type": "code",
        "source" : source,
        "outputs": outputs,
        // The `execution_count` is required in Jupyter Notebook v4.5 but can be `null`;
        // it is not the same as Stencila `execute_count`which increments across sessions.
        "execution_count": null,
        "metadata": metadata
    })
}

/// Encode the `outputs` of a Stencila `CodeChunk` to an array of Jupyter `Output`s.
///
/// Note that the Stencila document model does not differentiate among different sources
/// of outputs e.g. `stdout` from a `print` statement versus a `string` from a `execute_result`.
/// So, we don't try to revert to the source that may have been in the `ipynb` originally.
/// Instead, consistent with decoding, preformatted text in a code block goes to a stream
/// output.
fn encode_outputs(nodes: &[Node]) -> Vec<serde_json::Value> {
    nodes
        .iter()
        .map(|node| match node {
            Node::ImageObject(image) => encode_display_data(image),
            _ => encode_execute_result(node),
        })
        .collect()
}

/// Encode a `String` as a Jupyter `Stream`.
#[allow(dead_code)]
fn encode_stream(text: &str) -> serde_json::Value {
    json!({
        "output_type": "stream",
        "name": "stdout",
        "text": text,
    })
}

/// Encode an `ImageObject` as a Jupyter `DisplayData`.
fn encode_display_data(image: &ImageObject) -> serde_json::Value {
    let url = &image.content_url;
    let data = if let Some(data) = url.strip_prefix("data:") {
        let parts = data.split(";base64,").collect::<Vec<&str>>();
        let mime_type = parts.first().map(|str| str.to_string()).unwrap_or_default();
        let data = parts.get(1);
        let data = if mime_type.starts_with("application/vnd.plotly")
            || mime_type.starts_with("application/vnd.vega")
        {
            data.and_then(|data| base64::decode(data).ok())
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or_else(|| json!({}))
        } else {
            json!(data)
        };
        json!({ mime_type: data })
    } else {
        json!({})
    };
    json!({
        "output_type": "display_data",
        "data": data,
        "metadata": {},
    })
}

/// Encode a `Node` as a Jupyter `ExecuteResult`.
fn encode_execute_result(node: &Node) -> serde_json::Value {
    let text = node.to_txt();
    let data = json!({ "text/plain": encode_multiline_string(&text) });
    json!({
        "output_type": "execute_result",
        "data": data,
        // The `execution_count` is required in Jupyter Notebook v4.5 but can be `null`;
        // it is not the same as Stencila `execution_count`which increments across sessions.
        "execution_count": null,
        "metadata": {},
    })
}

/// Encode a vector of `BlockContent` to a Jupyter Markdown cell
fn encode_markdown(blocks: &[BlockContent]) -> serde_json::Value {
    let md = blocks.to_md(&EncodeOptions::default());
    json!({
        "cell_type": "markdown",
        "source" : encode_multiline_string(&md),
        "metadata": {}
    })
}

/// Encode a `String` to a Jupyter multiline string
fn encode_multiline_string(string: &str) -> Vec<&str> {
    string.split_inclusive('\n').collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::decode::decode;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures_content};

    #[test]
    fn encode_ipynb_articles() {
        snapshot_fixtures_content("articles/*.ipynb", |content| {
            let decoded = decode(content).unwrap();
            let encoded = encode(&decoded).unwrap();
            let json = serde_json::from_str::<serde_json::Value>(&encoded).unwrap();
            assert_json_snapshot!(json);
        });
    }
}
