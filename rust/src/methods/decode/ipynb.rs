use eyre::Result;
use stencila_schema::{Article, BlockContent, Node};

use super::md;

/// Decode a Jupyter Notebook to a `Node`.
///
/// Aims to support the [Jupyter Notebook v4.5 schema](https://github.com/jupyter/nbformat/blob/master/nbformat/v4/nbformat.v4.5.schema.json)
/// but should work for any v4 notebook.
///
/// Aims to be permissive by ignoring but warning about data that does not meet v4 schema, rather than
/// erroring on it.
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

    let content = if let Some(cells) = notebook.get("cells").and_then(|value| value.as_array()) {
        let mut content = Vec::with_capacity(cells.len());
        for cell in cells {
            let cell_type = cell
                .get("cell_type")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let mut blocks = match cell_type {
                "code" => translate_code_cell(cell),
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
fn translate_code_cell(_cell: &serde_json::Value) -> Vec<BlockContent> {
    todo!()
}

/// Translate a Jupyter "markdown" cell
fn translate_markdown_cell(cell: &serde_json::Value) -> Vec<BlockContent> {
    let markdown = if let Some(source) = cell.get("source") {
        translate_multiline_string(source)
    } else {
        tracing::warn!("Markdown cell does not have a `source` property");
        return Vec::new();
    };
    md::decode_fragment(&markdown)
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
    use serde_json::json;

    use super::*;

    #[test]
    fn multiline_string() {
        let mls1 = json!(["Line1\n", "Line2"]);
        let mls2 = json!("Line1\nLine2");
        let str1 = "Line1\nLine2";

        assert_eq!(translate_multiline_string(&mls1), str1);
        assert_eq!(translate_multiline_string(&mls2), str1);
    }
}
