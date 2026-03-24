use serde_json::{Map, Value, json};
use stencila_codec::{
    eyre::{Result, eyre},
    stencila_schema::{CodeInline, Emphasis, Inline, Strong, Subscript, Superscript, Text},
};

pub(crate) fn encode_inline_children(content: &[Inline]) -> Value {
    Value::Array(content.iter().map(encode_inline).collect())
}

fn encode_inline_container(oxa_type: &str, content: &[Inline]) -> Value {
    json!({
        "type": oxa_type,
        "children": encode_inline_children(content),
    })
}

pub fn encode_inline(inline: &Inline) -> Value {
    match inline {
        Inline::Text(text) => json!({
            "type": "Text",
            "value": text.value.as_str(),
        }),
        Inline::Emphasis(Emphasis { content, .. }) => encode_inline_container("Emphasis", content),
        Inline::Strong(Strong { content, .. }) => encode_inline_container("Strong", content),
        Inline::CodeInline(code_inline) => json!({
            "type": "InlineCode",
            "value": code_inline.code.as_str(),
        }),
        Inline::Subscript(Subscript { content, .. }) => {
            encode_inline_container("Subscript", content)
        }
        Inline::Superscript(Superscript { content, .. }) => {
            encode_inline_container("Superscript", content)
        }
        _ => json!({
            "type": "Text",
            "value": "",
        }),
    }
}

pub fn decode_inline(obj: &Map<String, Value>) -> Result<Inline> {
    let type_str = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match type_str {
        "Text" => {
            let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Inline::Text(Text::new(value.into())))
        }
        "Emphasis" => decode_inline_children(obj).map(|c| Inline::Emphasis(Emphasis::new(c))),
        "Strong" => decode_inline_children(obj).map(|c| Inline::Strong(Strong::new(c))),
        "InlineCode" => {
            let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Inline::CodeInline(CodeInline::new(value.into())))
        }
        "Subscript" => decode_inline_children(obj).map(|c| Inline::Subscript(Subscript::new(c))),
        "Superscript" => {
            decode_inline_children(obj).map(|c| Inline::Superscript(Superscript::new(c)))
        }
        _ => Err(eyre!("Unknown inline type: \"{type_str}\"")),
    }
}

pub(crate) fn decode_inline_children(obj: &Map<String, Value>) -> Result<Vec<Inline>> {
    obj.get("children")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_object())
                .map(decode_inline)
                .collect::<Result<Vec<_>>>()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}
