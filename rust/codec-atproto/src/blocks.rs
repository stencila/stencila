use serde_json::{Value, json};
use stencila_codec::{
    Losses,
    stencila_schema::{
        Block, CodeBlock, Heading, List, ListItem, ListOrder, MathBlock, Paragraph, QuoteBlock,
    },
};

use crate::{
    facets::{ByteSlice, Facet, RichText, flatten_inlines},
    nsids,
};

/// Encode a single Stencila block node to an AT Protocol JSON value.
///
/// Returns `None` for unsupported block types (recording a loss).
pub fn encode_block(block: &Block, losses: &mut Losses) -> Option<Value> {
    record_dropped_properties(block, losses);

    match block {
        Block::Paragraph(para) => Some(encode_paragraph(para, losses)),
        Block::Heading(heading) => Some(encode_heading(heading, losses)),
        Block::CodeBlock(code_block) => Some(encode_code_block(code_block)),
        Block::ThematicBreak(..) => Some(json!({ "$type": nsids::OXA_THEMATIC_BREAK })),
        Block::MathBlock(math_block) => Some(encode_math_block(math_block)),
        Block::QuoteBlock(quote_block) => Some(encode_quote_block(quote_block, losses)),
        Block::List(list) => Some(encode_list(list, losses)),
        _ => {
            losses.add(format!("encode:unsupported_block_{block}"));
            None
        }
    }
}

fn record_dropped_properties(block: &Block, losses: &mut Losses) {
    let has_id = matches!(
        block,
        Block::Paragraph(Paragraph { id: Some(_), .. })
            | Block::Heading(Heading { id: Some(_), .. })
            | Block::CodeBlock(CodeBlock { id: Some(_), .. })
            | Block::MathBlock(MathBlock { id: Some(_), .. })
            | Block::QuoteBlock(QuoteBlock { id: Some(_), .. })
            | Block::List(List { id: Some(_), .. })
    );
    if has_id {
        losses.add("encode:dropped_property_id");
    }

    if let Block::StyledBlock(styled) = block
        && styled.options.class_list.is_some()
    {
        losses.add("encode:dropped_property_classes");
    }
}

fn encode_paragraph(para: &Paragraph, losses: &mut Losses) -> Value {
    let rt = flatten_inlines(&para.content, losses);
    richtext_to_block_value(nsids::OXA_PARAGRAPH, &rt)
}

fn encode_heading(heading: &Heading, losses: &mut Losses) -> Value {
    let rt = flatten_inlines(&heading.content, losses);
    let mut value = richtext_to_block_value(nsids::OXA_HEADING, &rt);
    if let Some(obj) = value.as_object_mut() {
        obj.insert("level".to_string(), json!(heading.level));
    }
    value
}

fn encode_code_block(code_block: &CodeBlock) -> Value {
    let mut obj = serde_json::Map::new();
    obj.insert("$type".to_string(), json!(nsids::OXA_CODE));
    obj.insert(
        "value".to_string(),
        Value::String(code_block.code.to_string()),
    );
    if let Some(lang) = &code_block.programming_language {
        obj.insert("language".to_string(), Value::String(lang.clone()));
    }
    Value::Object(obj)
}

fn encode_math_block(math_block: &MathBlock) -> Value {
    json!({
        "$type": nsids::OXA_MATH,
        "tex": math_block.code.as_str(),
    })
}

fn encode_quote_block(quote_block: &QuoteBlock, losses: &mut Losses) -> Value {
    let mut combined_text = String::new();
    let mut combined_facets: Vec<Facet> = Vec::new();

    for child in &quote_block.content {
        if let Block::Paragraph(para) = child {
            if !combined_text.is_empty() {
                combined_text.push('\n');
            }

            let offset = combined_text.len();
            let rt = flatten_inlines(&para.content, losses);

            combined_text.push_str(&rt.text);

            for facet in rt.facets {
                combined_facets.push(Facet {
                    index: ByteSlice {
                        byte_start: facet.index.byte_start + offset,
                        byte_end: facet.index.byte_end + offset,
                    },
                    features: facet.features,
                });
            }
        } else {
            losses.add("encode:blockquote_non_paragraph_child");
        }
    }

    losses.add("encode:blockquote_structure_flattened");

    let rt = RichText {
        text: combined_text,
        facets: combined_facets,
    };
    richtext_to_block_value(nsids::OXA_BLOCKQUOTE, &rt)
}

fn is_ordered(order: &ListOrder) -> bool {
    matches!(order, ListOrder::Ascending | ListOrder::Descending)
}

fn encode_list(list: &List, losses: &mut Losses) -> Value {
    let ordered = is_ordered(&list.order);
    let type_str = if ordered {
        nsids::OXA_ORDERED_LIST
    } else {
        nsids::OXA_UNORDERED_LIST
    };

    let children: Vec<Value> = list
        .items
        .iter()
        .map(|item| encode_list_item(item, &list.order, losses))
        .collect();

    let mut obj = serde_json::Map::new();
    obj.insert("$type".to_string(), json!(type_str));
    if ordered {
        obj.insert("startIndex".to_string(), json!(1));
    }
    obj.insert("children".to_string(), Value::Array(children));

    Value::Object(obj)
}

fn encode_list_item(item: &ListItem, parent_order: &ListOrder, losses: &mut Losses) -> Value {
    let mut paragraph_content = None;
    let mut nested_list = None;
    let mut extra_block_count = 0;

    for child in &item.content {
        match child {
            Block::Paragraph(para) if paragraph_content.is_none() => {
                paragraph_content = Some(para);
            }
            Block::List(sub_list) if nested_list.is_none() => {
                if is_ordered(&sub_list.order) != is_ordered(parent_order) {
                    losses.add("encode:list_mixed_nesting_type");
                }
                nested_list = Some(sub_list);
            }
            _ => {
                extra_block_count += 1;
            }
        }
    }

    if extra_block_count > 0 {
        losses.add("encode:list_item_extra_blocks");
    }

    let mut obj = if let Some(para) = paragraph_content {
        let rt = flatten_inlines(&para.content, losses);
        let value = rt.to_value();
        value.as_object().cloned().unwrap_or_default()
    } else {
        serde_json::Map::new()
    };

    if let Some(sub_list) = nested_list {
        let sub_children: Vec<Value> = sub_list
            .items
            .iter()
            .map(|sub_item| encode_list_item(sub_item, &sub_list.order, losses))
            .collect();
        obj.insert("children".to_string(), Value::Array(sub_children));
    }

    Value::Object(obj)
}

fn richtext_to_block_value(type_str: &str, rt: &RichText) -> Value {
    let mut value = rt.to_value();
    if let Some(obj) = value.as_object_mut() {
        obj.insert("$type".to_string(), json!(type_str));
    }
    value
}
