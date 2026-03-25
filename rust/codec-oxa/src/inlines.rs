use serde_json::{Map, Value, json};
use stencila_codec::{
    Losses,
    eyre::Result,
    stencila_schema::{CodeInline, Emphasis, Inline, Strong, Subscript, Superscript, Text},
};

use crate::{
    generic::encode_inline_generic,
    helpers::{oxa_type_str, record_classes_loss},
};

pub(crate) fn encode_inline_children(content: &[Inline], losses: &mut Losses) -> Value {
    Value::Array(content.iter().map(|i| encode_inline(i, losses)).collect())
}

fn encode_inline_container(oxa_type: &str, content: &[Inline], losses: &mut Losses) -> Value {
    json!({
        "type": oxa_type,
        "children": encode_inline_children(content, losses),
    })
}

pub fn encode_inline(inline: &Inline, losses: &mut Losses) -> Value {
    match inline {
        Inline::Text(text) => json!({
            "type": "Text",
            "value": text.value.as_str(),
        }),
        Inline::Emphasis(Emphasis { content, .. }) => {
            encode_inline_container("Emphasis", content, losses)
        }
        Inline::Strong(Strong { content, .. }) => {
            encode_inline_container("Strong", content, losses)
        }
        Inline::CodeInline(code_inline) => json!({
            "type": "InlineCode",
            "value": code_inline.code.as_str(),
        }),
        Inline::Subscript(Subscript { content, .. }) => {
            encode_inline_container("Subscript", content, losses)
        }
        Inline::Superscript(Superscript { content, .. }) => {
            encode_inline_container("Superscript", content, losses)
        }
        _ => encode_inline_generic(inline, losses),
    }
}

pub fn decode_inline(obj: &Map<String, Value>, losses: &mut Losses) -> Result<Inline> {
    let type_str = oxa_type_str(obj);
    record_classes_loss(obj, losses);

    match type_str {
        "Text" => {
            let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Inline::Text(Text::new(value.into())))
        }
        "Emphasis" => {
            decode_inline_children(obj, losses).map(|c| Inline::Emphasis(Emphasis::new(c)))
        }
        "Strong" => decode_inline_children(obj, losses).map(|c| Inline::Strong(Strong::new(c))),
        "InlineCode" => {
            let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Inline::CodeInline(CodeInline::new(value.into())))
        }
        "Subscript" => {
            decode_inline_children(obj, losses).map(|c| Inline::Subscript(Subscript::new(c)))
        }
        "Superscript" => {
            decode_inline_children(obj, losses).map(|c| Inline::Superscript(Superscript::new(c)))
        }
        _ => {
            // Unknown inline type → Text with recursive text extraction
            losses.add("unknown_inline_to_text");
            let text = extract_text_recursive(obj);
            Ok(Inline::Text(Text::new(text.into())))
        }
    }
}

/// Recursively extract all text content from an OXA JSON object.
///
/// Checks the `value` field first (for leaf nodes like MathInline),
/// then recurses into `children` to concatenate all nested text.
fn extract_text_recursive(obj: &Map<String, Value>) -> String {
    if let Some(value) = obj.get("value").and_then(|v| v.as_str())
        && obj.get("children").and_then(|v| v.as_array()).is_none()
    {
        return value.to_string();
    }

    obj.get("children")
        .and_then(|v| v.as_array())
        .map(|children| {
            children
                .iter()
                .filter_map(|v| v.as_object())
                .map(extract_text_recursive)
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn decode_inline_children(
    obj: &Map<String, Value>,
    losses: &mut Losses,
) -> Result<Vec<Inline>> {
    obj.get("children")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_object())
                .map(|o| decode_inline(o, losses))
                .collect::<Result<Vec<_>>>()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}
