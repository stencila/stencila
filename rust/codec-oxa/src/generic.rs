use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Map, Value};
use stencila_codec::{
    Losses, NodeType,
    stencila_schema::{
        Admonition, Block, CodeChunk, Figure, ForBlock, IfBlock, IfBlockClause, Inline, List,
        ListItem, Node, QuoteBlock, Section, Table, TableCell, TableCellType, TableRow,
    },
};

use crate::blocks::encode_block;

/// Returns the child property names (camelCase) that require recursive encoding
/// for a given Stencila node type.
fn child_properties(node_type: NodeType) -> &'static [&'static str] {
    use NodeType::*;
    match node_type {
        Admonition => &["content"],
        CallBlock => &["content"],
        Chat => &["content"],
        ChatMessage => &["content"],
        ChatMessageGroup => &["messages"],
        CitationGroup => &["items"],
        Claim => &["content"],
        CodeChunk => &["caption", "outputs"],
        Emphasis => &["content"],
        Excerpt => &["content"],
        Figure => &["content", "caption"],
        ForBlock => &["content", "otherwise", "iterations"],
        Form => &["content"],
        IfBlock => &["clauses"],
        IfBlockClause => &["content"],
        IncludeBlock => &["content"],
        InlinesBlock => &["content"],
        Link => &["content"],
        List => &["items"],
        ListItem => &["content"],
        QuoteBlock => &["content"],
        QuoteInline => &["content"],
        Section => &["content"],
        Sentence => &["content"],
        Strong => &["content"],
        StyledBlock => &["content"],
        StyledInline => &["content"],
        Subscript => &["content"],
        SuggestionBlock => &["content"],
        SuggestionInline => &["content"],
        Superscript => &["content"],
        Supplement => &["content"],
        Table => &["caption", "rows", "notes"],
        TableCell => &["content"],
        TableRow => &["cells"],
        Walkthrough => &["steps"],
        WalkthroughStep => &["content"],
        _ => &[],
    }
}

/// Returns the property name that should be mapped to `children` in the OXA
/// output. For types with only one child property this is that property. For
/// types with multiple child properties, this picks the most semantically
/// appropriate one — `content` when present, otherwise a type-specific primary
/// (e.g. `rows` for Table, `outputs` for CodeChunk). Remaining child
/// properties are placed into `data`.
fn primary_child_property(node_type: NodeType) -> Option<&'static str> {
    use NodeType::*;
    match node_type {
        // Types with multiple child properties — pick the primary one
        CodeChunk => Some("outputs"),
        Figure => Some("content"),
        ForBlock => Some("content"),
        Table => Some("rows"),
        _ => {
            // Single-child types: use whatever child_properties returns
            let children = child_properties(node_type);
            children.first().copied()
        }
    }
}

/// Properties to skip during generic encoding.
const SKIP_PROPERTIES: &[&str] = &["type", "uid", "options"];

fn to_value(value: &impl Serialize) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

/// Post-process a serde-serialized value to fix known serialization quirks:
/// - Cord type serializes as `{"string": "..."}` → flatten to just the string
/// - Certain enum variants use Rust naming (e.g., `HeaderCell`) → map to OXA naming
fn normalize_value(value: &Value) -> Value {
    match value {
        Value::Object(obj) => {
            // Cord type: {"string": "..."} → "..."
            if obj.len() == 1
                && let Some(s) = obj.get("string").and_then(|v| v.as_str())
            {
                return Value::String(s.to_string());
            }
            Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), normalize_value(v)))
                    .collect(),
            )
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_value).collect()),
        Value::String(s) => match s.as_str() {
            "HeaderCell" => Value::String("Header".into()),
            "DataCell" => Value::String("Data".into()),
            _ => value.clone(),
        },
        other => other.clone(),
    }
}

/// Encode a Block using the generic fallback strategy.
pub fn encode_block_generic(block: &Block, losses: &mut Losses) -> Value {
    losses.add("generic");
    let raw = to_value(block);
    encode_generic_value(block.node_type(), &raw, |prop_name, prop_value| {
        encode_walked_block_property(prop_name, prop_value, block, losses)
    })
}

/// Encode an Inline using the generic fallback strategy.
pub fn encode_inline_generic(inline: &Inline, losses: &mut Losses) -> Value {
    losses.add("generic");
    let raw = to_value(inline);
    encode_generic_value(inline.node_type(), &raw, |_prop_name, raw_value| {
        raw_value.clone()
    })
}

/// Core generic encoding logic.
fn encode_generic_value<F>(node_type: NodeType, raw: &Value, mut encode_walked: F) -> Value
where
    F: FnMut(&str, &Value) -> Value,
{
    let obj = match raw.as_object() {
        Some(o) => o,
        None => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String(node_type.to_string()));
            return Value::Object(m);
        }
    };

    let children = child_properties(node_type);
    let mut data = Map::new();
    let mut walked_values: BTreeMap<String, Value> = BTreeMap::new();

    let options_obj = obj.get("options").and_then(|v| v.as_object());

    for (key, value) in obj
        .iter()
        .chain(options_obj.into_iter().flat_map(|o| o.iter()))
    {
        if SKIP_PROPERTIES.contains(&key.as_str()) || value.is_null() {
            continue;
        }

        if children.contains(&key.as_str()) {
            walked_values.insert(key.clone(), encode_walked(key, value));
        } else {
            data.insert(key.clone(), normalize_value(value));
        }
    }

    let mut result = Map::new();
    result.insert("type".into(), Value::String(node_type.to_string()));

    let primary = primary_child_property(node_type);

    // The primary child property becomes `children`; all other walked
    // properties go into `data`.
    if let Some(primary_name) = primary
        && let Some(val) = walked_values.remove(primary_name)
    {
        result.insert("children".into(), val);
    }
    for (name, val) in walked_values {
        data.insert(name, val);
    }
    if !data.is_empty() {
        result.insert("data".into(), Value::Object(data));
    }

    Value::Object(result)
}

/// Encode any `Serialize` struct whose primary child property is `content` (a `Vec<Block>`).
fn encode_content_bearing(
    node_type: NodeType,
    content: &[Block],
    value: &impl Serialize,
    losses: &mut Losses,
) -> Value {
    let raw = to_value(value);
    encode_generic_value(node_type, &raw, |prop_name, raw_value| {
        if prop_name == "content" {
            Value::Array(content.iter().map(|b| encode_block(b, losses)).collect())
        } else {
            raw_value.clone()
        }
    })
}

/// Insert a key-value pair into the `data` object of a generic-encoded result,
/// creating the `data` object if it doesn't exist.
fn insert_into_data(result: &mut Value, key: &str, value: Value) {
    if let Some(obj) = result.as_object_mut() {
        if let Some(data) = obj.get_mut("data").and_then(|d| d.as_object_mut()) {
            data.insert(key.into(), value);
        } else {
            let mut data_map = Map::new();
            data_map.insert(key.into(), value);
            obj.insert("data".into(), Value::Object(data_map));
        }
    }
}

enum ChildItems<'a> {
    Blocks(&'a [Block]),
    IfBlockClauses(&'a [IfBlockClause]),
    ListItems(&'a [ListItem]),
    TableRows(&'a [TableRow]),
    Nodes(&'a [Node]),
    None,
}

impl<'a> ChildItems<'a> {
    fn from_optional_blocks(opt: &'a Option<Vec<Block>>) -> Self {
        opt.as_deref().map_or(Self::None, Self::Blocks)
    }

    fn from_optional_nodes(opt: &'a Option<Vec<Node>>) -> Self {
        opt.as_deref().map_or(Self::None, Self::Nodes)
    }
}

fn encode_walked_block_property(
    prop_name: &str,
    raw_value: &Value,
    block: &Block,
    losses: &mut Losses,
) -> Value {
    if !raw_value.is_array() {
        return raw_value.clone();
    }

    let items = get_block_child_items(prop_name, block);
    match items {
        ChildItems::Blocks(blocks) => {
            Value::Array(blocks.iter().map(|b| encode_block(b, losses)).collect())
        }
        ChildItems::IfBlockClauses(clauses) => Value::Array(
            clauses
                .iter()
                .map(|c| encode_if_block_clause(c, losses))
                .collect(),
        ),
        ChildItems::ListItems(items) => {
            Value::Array(items.iter().map(|i| encode_list_item(i, losses)).collect())
        }
        ChildItems::TableRows(rows) => {
            Value::Array(rows.iter().map(|r| encode_table_row(r, losses)).collect())
        }
        ChildItems::Nodes(nodes) => Value::Array(nodes.iter().map(to_value).collect()),
        ChildItems::None => raw_value.clone(),
    }
}

fn get_block_child_items<'a>(prop_name: &str, block: &'a Block) -> ChildItems<'a> {
    match block {
        Block::List(List { items, .. }) if prop_name == "items" => ChildItems::ListItems(items),
        Block::ForBlock(ForBlock {
            content,
            otherwise,
            iterations,
            ..
        }) => match prop_name {
            "content" => ChildItems::Blocks(content),
            "otherwise" => ChildItems::from_optional_blocks(otherwise),
            "iterations" => ChildItems::from_optional_blocks(iterations),
            _ => ChildItems::None,
        },
        Block::Table(Table {
            rows,
            caption,
            notes,
            ..
        }) => match prop_name {
            "rows" => ChildItems::TableRows(rows),
            "caption" => ChildItems::from_optional_blocks(caption),
            "notes" => ChildItems::from_optional_blocks(notes),
            _ => ChildItems::None,
        },
        Block::CodeChunk(CodeChunk {
            caption, outputs, ..
        }) => match prop_name {
            "caption" => ChildItems::from_optional_blocks(caption),
            "outputs" => ChildItems::from_optional_nodes(outputs),
            _ => ChildItems::None,
        },
        Block::IfBlock(IfBlock { clauses, .. }) if prop_name == "clauses" => {
            ChildItems::IfBlockClauses(clauses)
        }
        Block::Section(Section { content, .. }) if prop_name == "content" => {
            ChildItems::Blocks(content)
        }
        Block::QuoteBlock(QuoteBlock { content, .. }) if prop_name == "content" => {
            ChildItems::Blocks(content)
        }
        Block::Admonition(Admonition { content, .. }) if prop_name == "content" => {
            ChildItems::Blocks(content)
        }
        Block::Figure(Figure {
            content, caption, ..
        }) => match prop_name {
            "content" => ChildItems::Blocks(content),
            "caption" => ChildItems::from_optional_blocks(caption),
            _ => ChildItems::None,
        },
        _ => ChildItems::None,
    }
}

fn encode_if_block_clause(clause: &IfBlockClause, losses: &mut Losses) -> Value {
    encode_content_bearing(NodeType::IfBlockClause, &clause.content, clause, losses)
}

fn encode_list_item(item: &ListItem, losses: &mut Losses) -> Value {
    encode_content_bearing(NodeType::ListItem, &item.content, item, losses)
}

fn encode_table_row(row: &TableRow, losses: &mut Losses) -> Value {
    let raw = to_value(row);
    let mut result = encode_generic_value(NodeType::TableRow, &raw, |prop_name, raw_value| {
        if prop_name == "cells" {
            Value::Array(
                row.cells
                    .iter()
                    .map(|c| encode_table_cell(c, losses))
                    .collect(),
            )
        } else {
            raw_value.clone()
        }
    });

    // Auto-detect rowType from cell types if not explicitly set
    if row.row_type.is_none() {
        let has_header_cells = row
            .cells
            .iter()
            .any(|c| matches!(c.cell_type, Some(TableCellType::HeaderCell)));
        if has_header_cells {
            insert_into_data(&mut result, "rowType", Value::String("HeaderRow".into()));
        }
    }

    result
}

fn encode_table_cell(cell: &TableCell, losses: &mut Losses) -> Value {
    let mut result = encode_content_bearing(NodeType::TableCell, &cell.content, cell, losses);

    if cell.cell_type.is_none() {
        insert_into_data(&mut result, "cellType", Value::String("Data".into()));
    }

    result
}
