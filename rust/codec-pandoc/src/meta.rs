use pandoc_types::definition::{self as pandoc};

use codec::schema::*;

use crate::{
    inlines::{inlines_from_pandoc, inlines_to_pandoc},
    shared::{PandocDecodeContext, PandocEncodeContext},
};

pub(super) fn string_to_meta_value(string: &str) -> pandoc::MetaValue {
    pandoc::MetaValue::MetaString(string.into())
}

pub(super) fn string_from_meta_value(meta: pandoc::MetaValue) -> String {
    match meta {
        pandoc::MetaValue::MetaString(string) => string,
        _ => String::new(),
    }
}

pub(super) fn inlines_to_meta_inlines(
    inlines: &Vec<Inline>,
    context: &mut PandocEncodeContext,
) -> pandoc::MetaValue {
    pandoc::MetaValue::MetaInlines(inlines_to_pandoc(inlines, context))
}

pub(super) fn inlines_from_meta_inlines(
    meta: pandoc::MetaValue,
    context: &mut PandocDecodeContext,
) -> Vec<Inline> {
    match meta {
        pandoc::MetaValue::MetaInlines(inlines) => inlines_from_pandoc(inlines, context),
        _ => Vec::new(),
    }
}

/*
/// Translate a map of `MetaValue` to a map of `serde_json::Value`
fn translate_meta_map(
    map: &HashMap<String, pandoc::MetaValue>,
    context: &mut PandocDecodeContext,
) -> serde_json::Map<String, serde_json::Value> {
    map.iter()
        .map(|(key, value)| (key.clone(), translate_meta_value(value, context)))
        .collect()
}

/// Translate a meta value to a `serde_json::Value`
fn translate_meta_value(
    value: &pandoc::MetaValue,
    context: &PandocDecodeContext,
) -> serde_json::Value {
    match value {
        pandoc::MetaValue::MetaMap(map) => {
            serde_json::Value::Object(translate_meta_map(map, context))
        }
        pandoc::MetaValue::MetaList(vec) => serde_json::Value::Array(
            vec.iter()
                .map(|value| translate_meta_value(value, context))
                .collect(),
        ),
        pandoc::MetaValue::MetaBool(bool) => serde_json::Value::Bool(*bool),
        pandoc::MetaValue::MetaString(string) => serde_json::Value::String(string.clone()),
        pandoc::MetaValue::MetaInlines(inlines) => serde_json::Value::Array(
            translate_inlines(inlines, context)
                .iter()
                .map(|inline| serde_json::to_value(inline).expect("Can serialize to JSON value"))
                .collect(),
        ),
        pandoc::MetaValue::MetaBlocks(blocks) => serde_json::Value::Array(
            translate_blocks(blocks, context)
                .iter()
                .map(|block| serde_json::to_value(block).expect("Can serialize to JSON value"))
                .collect(),
        ),
    }
}
*/
