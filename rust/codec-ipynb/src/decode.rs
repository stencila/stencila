use codec::{
    common::{eyre::Result, serde_json, tracing},
    stencila_schema::*,
};
use node_coerce::coerce;
use node_transform::Transform;

use super::translate::{
    translate_error, translate_mime_bundle, translate_multiline_string, translate_stderr,
    translate_text,
};

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

    let (id, label, caption) = if let Some(metadata) = metadata {
        let id = metadata
            .get("id")
            .and_then(|value| value.as_str())
            .map(|value| value.into());

        let label = metadata
            .get("label")
            .and_then(|value| value.as_str())
            .map(|value| Box::new(value.to_string()));

        let caption = if let Some(caption) = metadata.get("caption") {
            let blocks = if let Some(string) = caption.as_str() {
                codec_md::decode_fragment(string, Some(notebook_lang.to_string()))
            } else if let Some(array) = caption.as_array() {
                array
                    .iter()
                    .filter_map(|item| match coerce(item.clone(), None) {
                        Ok(node) => Some(node),
                        Err(error) => {
                            tracing::warn!("While coercing cell caption: {}", error);
                            None
                        }
                    })
                    .collect::<Vec<Node>>()
                    .to_blocks()
            } else {
                match coerce(caption.clone(), None) {
                    Ok(node) => node.to_blocks(),
                    Err(error) => {
                        tracing::warn!("While coercing cell caption: {}", error);
                        Vec::new()
                    }
                }
            };
            Some(Box::new(CodeChunkCaption::VecBlockContent(blocks)))
        } else {
            None
        };

        (id, label, caption)
    } else {
        (None, None, None)
    };

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
            match translate_output(output, &programming_language) {
                Some(Node::CodeError(error)) => errors.push(error),
                Some(node) => outputs.push(node),
                None => (),
            }
        }
    }

    let chunk = CodeChunk {
        id,
        label,
        caption,
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
fn translate_output(output: &serde_json::Value, language: &str) -> Option<Node> {
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
                Some("stderr") => text.map(|value| {
                    let error = translate_stderr(value);
                    Node::CodeError(error)
                }),
                _ => text.and_then(translate_text),
            }
        }
        "error" => Some(Node::CodeError(translate_error(output, language))),
        _ => {
            tracing::warn!("Unhandled output type: {}", output_type);
            None
        }
    }
}

/// Translate a Jupyter "markdown" cell
fn translate_markdown_cell(cell: &serde_json::Value, default_lang: &str) -> Vec<BlockContent> {
    if let Some(source) = cell.get("source") {
        codec_md::decode_fragment(
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

#[cfg(test)]
mod test {
    use super::*;
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures_content};

    #[test]
    fn decode_ipynb_articles() {
        snapshot_fixtures_content("articles/*.ipynb", |content| {
            assert_json_snapshot!(decode(content).unwrap());
        });
    }
}
