use serde_json::{Map, Value, json};
use stencila_codec::{
    Losses,
    eyre::Result,
    stencila_schema::{Block, CodeBlock, Heading, Paragraph, RawBlock, ThematicBreak},
};

use crate::{
    generic::encode_block_generic,
    helpers::{oxa_type_str, record_classes_loss},
    inlines::{decode_inline_children, encode_inline_children},
};

pub fn encode_block(block: &Block, losses: &mut Losses) -> Value {
    match block {
        Block::Paragraph(Paragraph { content, .. }) => json!({
            "type": "Paragraph",
            "children": encode_inline_children(content, losses),
        }),
        Block::Heading(Heading { level, content, .. }) => json!({
            "type": "Heading",
            "level": level,
            "children": encode_inline_children(content, losses),
        }),
        Block::CodeBlock(CodeBlock {
            code,
            programming_language,
            ..
        }) => {
            let mut obj = json!({
                "type": "Code",
                "value": code.as_str(),
            });
            if let Some(lang) = programming_language {
                obj["language"] = json!(lang);
            }
            obj
        }
        Block::ThematicBreak(_) => json!({
            "type": "ThematicBreak",
        }),
        _ => encode_block_generic(block, losses),
    }
}

pub fn decode_block(obj: &Map<String, Value>, losses: &mut Losses) -> Result<Block> {
    let type_str = oxa_type_str(obj);
    record_classes_loss(obj, losses);

    match type_str {
        "Paragraph" => {
            let children = decode_inline_children(obj, losses)?;
            Ok(Block::Paragraph(Paragraph::new(children)))
        }
        "Heading" => {
            let level = obj.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
            let children = decode_inline_children(obj, losses)?;
            Ok(Block::Heading(Heading::new(level, children)))
        }
        "Code" => {
            let value = obj.get("value").and_then(|v| v.as_str()).unwrap_or("");
            let language = obj
                .get("language")
                .and_then(|v| v.as_str())
                .map(String::from);
            Ok(Block::CodeBlock(CodeBlock {
                code: value.into(),
                programming_language: language,
                ..Default::default()
            }))
        }
        "ThematicBreak" => Ok(Block::ThematicBreak(ThematicBreak::new())),
        _ => {
            // Unknown block type → RawBlock with verbatim JSON
            losses.add("unknown_block_to_raw");
            let content = serde_json::to_string(&Value::Object(obj.clone()))?;
            Ok(Block::RawBlock(RawBlock::new(
                "application/vnd.oxa+json".to_string(),
                content.into(),
            )))
        }
    }
}
