//! Utility functions for working with Jupyter notebooks and kernels
//!
//! These functions are shared by the modules for decoding Jupyter notebooks and
//! for executing code within them. They use them to translate Jupyter outputs
//! and errors into their Stencila equivalents.

use crate::methods::{
    decode::{html, txt},
    transform::Transform,
};
use itertools::Itertools;
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
/// Uses `txt:decode` to attempt to parse the text to something other
/// than a string. However, if the result is a `String` (i.e. `txt:decode` could not
/// parse the text), and it contains pre-formatting (tabs, newlines or more than one consecutive space),
// then decode as a `CodeBlock` since formatting is often important in text output of cells.
pub fn translate_text(text: &serde_json::Value) -> Option<Node> {
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
pub fn translate_stderr(text: &serde_json::Value) -> Option<Node> {
    Some(Node::CodeError(CodeError {
        error_message: translate_multiline_string(text),
        error_type: Some(Box::new("stderr".to_string())),
        ..Default::default()
    }))
}

/// Translate a cell error into a `CodeError`
pub fn translate_error(error: &serde_json::Value) -> Option<Node> {
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
