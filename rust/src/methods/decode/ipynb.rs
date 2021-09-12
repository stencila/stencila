use super::{html, md, txt};
use crate::methods::coerce::coerce;
use crate::traits::ToNode;
use eyre::Result;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use stencila_schema::{Article, BlockContent, CodeBlock, CodeChunk, CodeError, ImageObject, Node};

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

    let metadata = notebook.get("metadata");

    let mut article = if let Some(metadata) = metadata {
        match coerce(metadata.clone(), Some("Article".to_string()))? {
            Node::Article(article) => article,
            _ => unreachable!("Should always be an article"),
        }
    } else {
        Article::default()
    };

    let mut language = None;
    if let Some(metadata) = metadata {
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

    if let Some(cells) = notebook.get("cells").and_then(|value| value.as_array()) {
        let mut content = Vec::with_capacity(cells.len());
        for cell in cells {
            let cell_type = cell
                .get("cell_type")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let mut blocks = match cell_type {
                "code" => translate_code_cell(cell, &language),
                "markdown" => translate_markdown_cell(cell, &language),
                "raw" => translate_raw_cell(cell),
                _ => {
                    tracing::warn!("Jupyter Notebook cell has unknown type: {}", cell_type);
                    Vec::new()
                }
            };
            content.append(&mut blocks);
        }
        article.content = Some(content)
    } else {
        tracing::warn!("Jupyter Notebook does not have a valid `cells` property");
    };

    Ok(Node::Article(article))
}

/// Translate a Jupyter "code" cell
fn translate_code_cell(cell: &serde_json::Value, notebook_lang: &str) -> Vec<BlockContent> {
    let metadata = cell.get("metadata");

    let programming_language = if let Some(cell_lang) = metadata
        .and_then(|value| value.get("language_info"))
        .and_then(|value| value.get("name"))
        .and_then(|value| value.as_str())
    {
        cell_lang.to_string()
    } else {
        notebook_lang.to_string()
    };

    let text = if let Some(source) = cell.get("source") {
        translate_multiline_string(source)
    } else {
        tracing::warn!("Code cell does not have a `source` property");
        "".to_string()
    };

    let mut outputs = Vec::with_capacity(1);
    let mut errors = Vec::new();

    if let Some(cell_outputs) = cell.get("outputs").and_then(|value| value.as_array()) {
        for output in cell_outputs {
            match translate_output(output) {
                Some(Node::CodeError(error)) => errors.push(error),
                Some(node) => outputs.push(node),
                None => (),
            }
        }
    }

    let chunk = CodeChunk {
        programming_language,
        text,
        outputs: if outputs.is_empty() {
            None
        } else {
            Some(outputs)
        },
        errors: if errors.is_empty() {
            None
        } else {
            Some(errors)
        },
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
        "stream" => {
            let text = output.get("text");
            let name = output.get("name").and_then(|value| value.as_str());
            match name {
                Some("stderr") => text.and_then(translate_stderr),
                _ => text.and_then(translate_text),
            }
        }
        "error" => translate_error(output),
        _ => {
            tracing::warn!("Unhandled output type: {}", output_type);
            None
        }
    }
}

/// Translate a MIME bundle into a `Node` (if possible).
///
/// Attempts to use the most "rich" and reliable (in terms of conversion loss)
/// MIME types first, returning early on success.
fn translate_mime_bundle(bundle: &serde_json::Value) -> Option<Node> {
    let plotly_media_type = "application/vnd.plotly.v1+json";
    if let Some(value) = bundle.get(plotly_media_type) {
        return translate_image_data(value, plotly_media_type);
    }

    // Spec and version numbers ordered for soonest potential match
    for spec in ["application/vnd.vegalite", "application/vnd.vega"] {
        for version in ["4", "5", "3", "2", "1"] {
            let media_type = [spec, ".v", version, "+json"].concat();
            if let Some(value) = bundle.get(&media_type) {
                return translate_image_data(value, &media_type);
            }
        }
    }

    for mime_type in ["image/png", "image/jpg", "image/gif", "image/svg+xml"] {
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
        return translate_text(text);
    }
    tracing::warn!("Unable to decode MIME bundle");
    None
}

/// Translate Plotly or Vega image data/spec into an `ImangeObject` node
fn translate_image_data(data: &serde_json::Value, media_type: &str) -> Option<Node> {
    let json = data.to_string();
    let bytes = json.as_bytes();
    let data = base64::encode(bytes);
    let content_url = ["data:", media_type, ";base64,", &data].concat();
    Some(Node::ImageObject({
        ImageObject {
            content_url,
            media_type: Some(Box::new(media_type.to_string())),
            ..Default::default()
        }
    }))
}

/// Translate text from a cell result or standard output stream into a `Node`.
///
/// Uses `txt:decode` to attempt to parse the text to something other
/// than a string. However, if the result is a `String` (i.e. `txt:decode` could not
/// parse the text), and it contains pre-formatting (tabs, newlines or more than one consecutive space),
// then decode as a `CodeBlock` since formatting is often important in text output of cells.
fn translate_text(text: &serde_json::Value) -> Option<Node> {
    let node = txt::decode_fragment(&translate_multiline_string(text));
    if let Node::String(text) = &node {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("[ ]{2,}|\t|\n").expect("Unable to create regex"));
        if REGEX.is_match(text.trim()) {
            return Some(Node::CodeBlock(CodeBlock {
                text: text.to_string(),
                ..Default::default()
            }));
        }
    }
    Some(node)
}

/// Translate text from a standard error stream into a `CodeError`.
fn translate_stderr(text: &serde_json::Value) -> Option<Node> {
    Some(Node::CodeError(CodeError {
        error_message: translate_multiline_string(text),
        error_type: Some(Box::new("stderr".to_string())),
        ..Default::default()
    }))
}

/// Translate a cell error into a `CodeError`
fn translate_error(error: &serde_json::Value) -> Option<Node> {
    let error_message = error
        .get("evalue")
        .and_then(|value| value.as_str())
        .map_or_else(|| "Unknown error".to_string(), |str| str.to_string());
    let error_type = error
        .get("ename")
        .and_then(|value| value.as_str())
        .map(|str| Box::new(str.to_string()));
    let stack_trace = error
        .get("traceback")
        .and_then(|value| value.as_array())
        .map(|vec| {
            let trace = vec.iter().filter_map(|line| line.as_str()).join("\n");
            let stripped = strip_ansi_escapes::strip(&trace).map_or_else(
                |_| trace,
                |bytes| String::from_utf8_lossy(&bytes).to_string(),
            );
            Box::new(stripped)
        });

    Some(Node::CodeError(CodeError {
        error_message,
        error_type,
        stack_trace,
        ..Default::default()
    }))
}

/// Translate a Jupyter "markdown" cell
fn translate_markdown_cell(cell: &serde_json::Value, default_lang: &str) -> Vec<BlockContent> {
    if let Some(source) = cell.get("source") {
        md::decode_fragment(
            &translate_multiline_string(source),
            Some(default_lang.to_string()),
        )
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
