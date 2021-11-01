//! Utility functions for working with Jupyter notebooks and kernels
//!
//! These functions are shared by the modules for decoding Jupyter notebooks and
//! for executing code within them. They use them to translate Jupyter outputs
//! and errors into their Stencila equivalents.

use codec_trait::Codec;
use codec_txt::TxtCodec;
use node_transform::Transform;
use once_cell::sync::Lazy;
use regex::Regex;
use stencila_schema::{CodeBlock, CodeError, ImageObject, Node};

/// Translate a MIME bundle into a `Node` (if possible).
///
/// Attempts to use the most "rich" and reliable (in terms of conversion loss)
/// MIME types first, returning early on success.
pub fn translate_mime_bundle(bundle: &serde_json::Value) -> Option<Node> {
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
        let blocks = codec_html::decode_fragment(&html, false);
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
pub fn translate_image_data(data: &serde_json::Value, media_type: &str) -> Option<Node> {
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
/// Uses `TxtCodec` to attempt to parse the text to something other
/// than a string. However, if the result is a `String` (i.e. `TxtCodec` could not
/// parse the text), and it contains pre-formatting (tabs, newlines or more than one consecutive space),
// then decode as a `CodeBlock` since formatting is often important in text output of cells.
pub fn translate_text(text: &serde_json::Value) -> Option<Node> {
    let node = TxtCodec::from_str(&translate_multiline_string(text)).ok();
    if let Some(Node::String(text)) = &node {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("[ ]{2,}|\t|\n").expect("Unable to create regex"));
        if REGEX.is_match(text.trim()) {
            return Some(Node::CodeBlock(CodeBlock {
                text: text.to_string(),
                ..Default::default()
            }));
        }
    }
    node
}

/// Translate text from a standard error stream into a `CodeError`.
pub fn translate_stderr(text: &serde_json::Value) -> CodeError {
    CodeError {
        error_message: translate_multiline_string(text),
        error_type: Some(Box::new("stderr".to_string())),
        ..Default::default()
    }
}

/// Language specific conversion of a `JupyterError` to a Stencila `CodeError`.
///
/// This function attempts to normalize error messages and stack traces
/// across kernels and remove extraneous (and thereby distracting) content
/// usually associated with running code in a kernel.
///
/// General approaches to normalization:
///
/// - remove lines in stack trace that are already in the error message
/// - remove text in error message or in the stack trace related to running in
///   a kernel
pub fn translate_error(error: &serde_json::Value, language: &str) -> CodeError {
    let error_type = error
        .get("ename")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    let error_message = error
        .get("evalue")
        .and_then(|value| value.as_str())
        .unwrap_or("Unknown error")
        .to_string();
    let mut stack_trace: Vec<String> = error
        .get("traceback")
        .and_then(|value| value.as_array())
        .map(|vec| {
            vec.iter()
                .filter_map(|item| item.as_str().map(|str| str.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Remove unnecessary lines from stack trace
    stack_trace.retain(|line| {
        let line = line.trim();
        !line.is_empty() && line != error_message && !line.starts_with("Traceback")
    });

    // Do language specific tidy ups
    if language == "python" {
        // Remove ANSI colour codes
        stack_trace.iter_mut().for_each(|line| {
            if let Ok(bytes) = strip_ansi_escapes::strip(line.clone()) {
                *line = String::from_utf8_lossy(&bytes).to_string();
            }
        });
    }

    let error_type = if error_type.to_lowercase() == "error" {
        None
    } else {
        Some(Box::new(error_type))
    };
    let stack_trace = if stack_trace.is_empty() {
        None
    } else {
        Some(Box::new(stack_trace.join("\n")))
    };
    CodeError {
        error_type,
        error_message,
        stack_trace,
        ..Default::default()
    }
}

/// Translates a Jupyter `multiline_string` (either a plain string, or an array of strings)
/// to a Rust `String`.
pub fn translate_multiline_string(multiline_string: &serde_json::Value) -> String {
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
    use serde_json::json;

    #[test]
    fn multiline_string() {
        let mls1 = json!(["Line1\n", "Line2"]);
        let mls2 = json!("Line1\nLine2");
        let str1 = "Line1\nLine2";

        assert_eq!(translate_multiline_string(&mls1), str1);
        assert_eq!(translate_multiline_string(&mls2), str1);
    }
}
