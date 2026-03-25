//! Snapshot tests for Visitor-based generic fallback encoding.
//!
//! These tests verify that Stencila node types with no direct OXA equivalent
//! are encoded into OXA-convention JSON using the generic fallback visitor.
//! Each test captures the full JSON shape using `insta::assert_json_snapshot!`
//! and compares against a pre-committed `.snap` baseline.
//!
//! There are exactly 6 snapshot tests, one per required structural pattern:
//!
//! 1. **List** — single walked property (`items`) → `children`
//! 2. **ForBlock** — primary walked property (`content`) → `children`; others (`otherwise`, `iterations`) → `data`
//! 3. **Table** — primary walked property (`rows`) → `children`; nested intermediate structs (TableRow, TableCell)
//! 4. **CodeChunk** — primary walked property (`outputs`) → `children`; `caption` → `data`
//! 5. **IfBlock** — IfBlockClause intermediate struct handling
//! 6. **MathBlock** — no walked properties → `data` only
//!
//! Each node type with walked properties has a *primary* child property that
//! maps to `children` in the OXA output; any remaining walked properties go
//! into `data`. For types with a single walked property, that property is
//! the primary. For types with multiple walked properties, the primary is
//! chosen explicitly (e.g. `content` for ForBlock, `rows` for Table,
//! `outputs` for CodeChunk).
//!
//! NOTE on walked properties: The Rust `#[walk]` attribute is the source of
//! truth. Properties named `content` default to walked even if not annotated
//! in the YAML schema. Key implications for intermediate structs:
//! - ListItem: 1 walked property (`content`) → uses `children`
//! - TableRow: 1 walked property (`cells`) → uses `children`
//! - TableCell: 1 walked property (`content`) → uses `children`
//! - IfBlockClause: 1 walked property (`content`) → uses `children`
//! - WalkthroughStep: 1 walked property (`content`) → uses `children`

use insta::assert_json_snapshot;
use serde_json::Value;
use stencila_codec::{
    Codec,
    eyre::{OptionExt, Result},
    stencila_schema::{
        Block, CodeChunk, Cord, ForBlock, Node, Section,
        shortcuts::{art, ibc, ifb, li, mb, p, t, tbl, td, th, tr, ul},
    },
};
use stencila_codec_oxa::OxaCodec;

/// Helper: encode a Stencila Article containing the given blocks to OXA JSON
/// and return the parsed JSON Value.
async fn encode_blocks_to_value(blocks: Vec<Block>) -> Result<Value> {
    let node = art(blocks);
    let codec = OxaCodec;
    let (json_str, _info) = codec.to_string(&node, None).await?;
    let value: Value = serde_json::from_str(&json_str)?;
    Ok(value)
}

/// Helper: encode a document and extract the first block child.
async fn encode_first_block(blocks: Vec<Block>) -> Result<Value> {
    let value = encode_blocks_to_value(blocks).await?;
    let first = value["children"]
        .as_array()
        .ok_or_eyre("missing children")?
        .first()
        .ok_or_eyre("empty children")?
        .clone();
    Ok(first)
}

// ===========================================================================
// Snapshot 1: List — single walked property → children
// ===========================================================================

/// An unordered List with two items.
///
/// Encoding rules:
/// - List has 1 walked property (`items`) → items become `children`
/// - ListItem has 1 walked property (`content`) → content becomes `children`
/// - Non-walked scalar `order` goes into `data`
#[tokio::test]
async fn snapshot_list() -> Result<()> {
    let block = ul([li([t("Item one")]), li([t("Item two")])]);
    let value = encode_first_block(vec![block]).await?;

    assert_json_snapshot!(value);

    Ok(())
}

// ===========================================================================
// Snapshot 2: ForBlock — primary child `content` → children, rest → data
// ===========================================================================

/// A ForBlock with all three walked properties populated.
///
/// Encoding rules:
/// - ForBlock has 3 walked properties (`content`, `otherwise`, `iterations`)
/// - `content` is the primary child → becomes `children`
/// - `otherwise` and `iterations` are secondary → go into `data`
/// - Non-walked scalars (`variable`, `code`) also go into `data`
#[tokio::test]
async fn snapshot_for_block() -> Result<()> {
    let block = Block::ForBlock(ForBlock {
        code: Cord::from("collection"),
        variable: "x".into(),
        content: vec![p([t("Has items")])],
        otherwise: Some(vec![p([t("No items")])]),
        iterations: Some(vec![
            Block::Section(Section {
                content: vec![p([t("Iteration 1")])],
                ..Default::default()
            }),
            Block::Section(Section {
                content: vec![p([t("Iteration 2")])],
                ..Default::default()
            }),
        ]),
        ..Default::default()
    });
    let value = encode_first_block(vec![block]).await?;

    assert_json_snapshot!(value);

    Ok(())
}

// ===========================================================================
// Snapshot 3: Table — nested intermediate structs
// ===========================================================================

/// A Table with header and data rows.
///
/// Encoding rules:
/// - Table has multiple walked properties (`caption`, `rows`, `notes`)
/// - `rows` is the primary child → becomes `children`
/// - `caption` and `notes` are secondary → go into `data`
/// - TableRow has 1 walked property (`cells`) → cells become `children`
/// - TableCell has 1 walked property (`content`) → content becomes `children`
/// - Non-walked scalars of each struct go into their respective `data`
#[tokio::test]
async fn snapshot_table() -> Result<()> {
    let block = tbl([
        tr([th([t("Header A")]), th([t("Header B")])]),
        tr([td([t("Cell 1")]), td([t("Cell 2")])]),
    ]);
    let value = encode_first_block(vec![block]).await?;

    assert_json_snapshot!(value);

    Ok(())
}

// ===========================================================================
// Snapshot 4: CodeChunk — primary child `outputs` → children, caption → data
// ===========================================================================

/// A CodeChunk with both caption and outputs populated.
///
/// Encoding rules:
/// - CodeChunk has 2 walked properties (`caption`, `outputs`)
/// - `outputs` is the primary child → becomes `children`
/// - `caption` is secondary → goes into `data`
/// - Non-walked scalars (`code`, `programmingLanguage`) also go into `data`
#[tokio::test]
async fn snapshot_code_chunk() -> Result<()> {
    let block = Block::CodeChunk(CodeChunk {
        code: Cord::from("print('hello')"),
        programming_language: Some("python".into()),
        caption: Some(vec![p([t("A code example")])]),
        outputs: Some(vec![Node::Integer(2)]),
        ..Default::default()
    });
    let value = encode_first_block(vec![block]).await?;

    assert_json_snapshot!(value);

    Ok(())
}

// ===========================================================================
// Snapshot 5: IfBlock — IfBlockClause intermediate struct handling
// ===========================================================================

/// An IfBlock with two clauses.
///
/// Encoding rules:
/// - IfBlock has 1 walked property (`clauses`) → clauses become `children`
/// - IfBlockClause has 1 walked property (`content`) → content becomes `children`
/// - Non-walked scalars of each clause (`code`, `programmingLanguage`) go into `data`
#[tokio::test]
async fn snapshot_if_block() -> Result<()> {
    let block = ifb([
        ibc("x > 0", Some("python"), [p([t("Positive")])]),
        ibc("true", Some("python"), [p([t("Default")])]),
    ]);
    let value = encode_first_block(vec![block]).await?;

    assert_json_snapshot!(value);

    Ok(())
}

// ===========================================================================
// Snapshot 6: MathBlock — no walked properties → data only
// ===========================================================================

/// A MathBlock with code and language.
///
/// Encoding rules:
/// - MathBlock has 0 walked properties → all fields go into `data`
/// - No `children` key (or empty `children`)
#[tokio::test]
async fn snapshot_math_block() -> Result<()> {
    let block = mb("E = mc^2", Some("tex"));
    let value = encode_first_block(vec![block]).await?;

    assert_json_snapshot!(value);

    Ok(())
}
