use serde_json::{Map, Value, json};
use stencila_codec::{
    eyre::{Result, eyre},
    stencila_schema::{Block, CodeBlock, Heading, Paragraph, ThematicBreak},
};

use crate::inlines::{decode_inline_children, encode_inline_children};

pub fn encode_block(block: &Block) -> Value {
    match block {
        Block::Paragraph(Paragraph { content, .. }) => json!({
            "type": "Paragraph",
            "children": encode_inline_children(content),
        }),
        Block::Heading(Heading { level, content, .. }) => json!({
            "type": "Heading",
            "level": level,
            "children": encode_inline_children(content),
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
        _ => json!({
            "type": "Paragraph",
            "children": [],
        }),
    }
}

pub fn decode_block(obj: &Map<String, Value>) -> Result<Block> {
    let type_str = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match type_str {
        "Paragraph" => {
            let children = decode_inline_children(obj)?;
            Ok(Block::Paragraph(Paragraph::new(children)))
        }
        "Heading" => {
            let level = obj.get("level").and_then(|v| v.as_i64()).unwrap_or(1);
            let children = decode_inline_children(obj)?;
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
        _ => Err(eyre!("Unknown block type: \"{type_str}\"")),
    }
}
