use super::{html, md, txt};
use crate::traits::ToNode;
use eyre::Result;
use stencila_schema::{Article, BlockContent, CodeChunk, ImageObject, Node};

/// Decode a Jupyter Notebook to a `Node`.
///
/// Aims to support the [Jupyter Notebook v4.5 schema](https://github.com/jupyter/nbformat/blob/master/nbformat/v4/nbformat.v4.5.schema.json)
/// but should work for any v4 notebook.
///
/// Aims to be permissive by ignoring but warning about data that does not meet v4 schema
/// (rather than erroring on it).
pub fn decode(ipynb: &str) -> Result<Node> {
    let notebook = serde_json::from_str::<serde_json::Value>(ipynb)?;

    if let Some(version) = notebook.get("nbformat").and_then(|value| value.as_u64()) {
        if version != 4 {
            tracing::warn!(
                "Jupyter Notebook has unsupported format version: {}",
                version
            );
        }
    } else {
        tracing::warn!(
            "Jupyter Notebook does not have a valid `nbformat` property; assuming version 4"
        );
    }

    let mut language = None;
    if let Some(metadata) = notebook.get("metadata") {
        if let Some(kernelspec) = metadata.get("kernelspec") {
            if let Some(lang) = kernelspec.get("language") {
                language = lang.as_str();
            }
        }
    };
    let language = match language {
        Some(language) => language.to_string(),
        None => {
            tracing::warn!("Unable to detect notebook language; assuming `python`");
            "python".to_string()
        }
    };

    let content = if let Some(cells) = notebook.get("cells").and_then(|value| value.as_array()) {
        let mut content = Vec::with_capacity(cells.len());
        for cell in cells {
            let cell_type = cell
                .get("cell_type")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let mut blocks = match cell_type {
                "code" => translate_code_cell(cell, &language),
                "markdown" => translate_markdown_cell(cell),
                "raw" => translate_raw_cell(cell),
                _ => {
                    tracing::warn!("Jupyter Notebook cell has unknown type: {}", cell_type);
                    Vec::new()
                }
            };
            content.append(&mut blocks);
        }
        content
    } else {
        tracing::warn!("Jupyter Notebook does not have a valid `cells` property");
        Vec::new()
    };

    let article = Article {
        content: if content.is_empty() {
            None
        } else {
            Some(content)
        },
        ..Default::default()
    };
    Ok(Node::Article(article))
}

/// Translate a Jupyter "code" cell
fn translate_code_cell(cell: &serde_json::Value, lang: &str) -> Vec<BlockContent> {
    let text = if let Some(source) = cell.get("source") {
        translate_multiline_string(source)
    } else {
        tracing::warn!("Code cell does not have a `source` property");
        "".to_string()
    };

    let outputs = if let Some(outputs) = cell.get("outputs").and_then(|value| value.as_array()) {
        let outputs = outputs
            .iter()
            .filter_map(translate_output)
            .collect::<Vec<Node>>();
        if outputs.is_empty() {
            None
        } else {
            Some(outputs)
        }
    } else {
        None
    };

    let chunk = CodeChunk {
        programming_language: lang.to_string(),
        text,
        outputs,
        ..Default::default()
    };

    vec![BlockContent::CodeChunk(chunk)]
}

/// Translate a cell output
fn translate_output(output: &serde_json::Value) -> Option<Node> {
    let output_type = output
        .get("output_type")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    match output_type {
        "execute_result" | "display_data" => output.get("data").and_then(translate_mime_bundle),
        "stream" => output.get("text").and_then(translate_stream_text),
        _ => {
            tracing::warn!("Unhandled output type: {}", output_type);
            None
        }
    }
}

/// Translate a MIME bundle into a `Node` (if possible).
///
/// Attempts to use the most "rich" and reliable MIME types first, returning
/// early on success.
fn translate_mime_bundle(bundle: &serde_json::Value) -> Option<Node> {
    for mime_type in ["image/png", "image/jpg", "image/gif"] {
        if let Some(data) = bundle.get(mime_type) {
            let data = translate_multiline_string(data);
            return Some(Node::ImageObject(ImageObject {
                content_url: ["data:", mime_type, ";base64,", &data].concat(),
                media_type: Some(Box::new(mime_type.to_string())),
                ..Default::default()
            }));
        }
    }
    if let Some(html) = bundle.get("text/html") {
        let html = translate_multiline_string(html);
        let blocks = html::decode_fragment(&html, false);
        if let Some(first) = blocks.first() {
            let node = first.clone().to_node();
            return Some(node);
        }
    }
    if let Some(text) = bundle.get("text/plain") {
        return Some(txt::decode_fragment(&translate_multiline_string(text)));
    }
    tracing::warn!("Unable to decode MIME bundle");
    None
}

/// Translate text from standard output stream into a `Node`.
///
/// Uses `txt:decode` to attempt to parse the text to something other
/// than a string (although that is what it falls back to).
fn translate_stream_text(text: &serde_json::Value) -> Option<Node> {
    Some(txt::decode_fragment(&translate_multiline_string(text)))
}

/// Translate a Jupyter "markdown" cell
fn translate_markdown_cell(cell: &serde_json::Value) -> Vec<BlockContent> {
    if let Some(source) = cell.get("source") {
        md::decode_fragment(&translate_multiline_string(source))
    } else {
        tracing::warn!("Markdown cell does not have a `source` property");
        Vec::new()
    }
}

/// Translate a Jupyter "raw" cell
fn translate_raw_cell(_cell: &serde_json::Value) -> Vec<BlockContent> {
    tracing::warn!("Decoding of raw cells is not yet supported");
    Vec::new()
}

/// Translates a Jupyter `multiline_string` (either a plain string, or an array of strings)
/// to a Rust `String`.
fn translate_multiline_string(multiline_string: &serde_json::Value) -> String {
    if let Some(str) = multiline_string.as_str() {
        str.to_string()
    } else if let Some(array) = multiline_string.as_array() {
        array
            .iter()
            .filter_map(|value| value.as_str().map(String::from))
            .collect::<Vec<String>>()
            .concat()
    } else {
        tracing::warn!("Unexpected value type for multiline string");
        "".to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::tests::snapshot_content;
    use insta::assert_json_snapshot;
    use serde_json::json;

    #[test]
    fn multiline_string() {
        let mls1 = json!(["Line1\n", "Line2"]);
        let mls2 = json!("Line1\nLine2");
        let str1 = "Line1\nLine2";

        assert_eq!(translate_multiline_string(&mls1), str1);
        assert_eq!(translate_multiline_string(&mls2), str1);
    }

    #[test]
    fn ipynb_articles() {
        snapshot_content("articles/*.ipynb", |_path, content| {
            assert_json_snapshot!(decode(&content).unwrap());
        });
    }
}
