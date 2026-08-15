//! Tests of the operational nesting-depth limit

use eyre::{Result, bail};

use stencila_node_compare::{CompareError, MAX_COMPARISON_DEPTH, align};
use stencila_schema::{
    Block, Node, Section,
    shortcuts::{p, t},
};

/// A node nesting `depth` sections around a paragraph
fn nested(depth: usize) -> Node {
    let mut block = p([t("Hello")]);
    for _ in 0..depth {
        block = Block::Section(Section::new(vec![block]));
    }
    Node::Section(Section::new(vec![block]))
}

/// A deeply, but legally, nested document aligns
#[test]
fn deep_nesting_aligns() -> Result<()> {
    let node = nested(MAX_COMPARISON_DEPTH - 4);

    let alignment = align(&node, &node)?;
    assert!(!alignment.has_one_sided());

    Ok(())
}

/// Nesting beyond the limit returns a typed error, rather than exhausting the stack
#[test]
fn excessive_nesting_is_an_error() -> Result<()> {
    let node = nested(MAX_COMPARISON_DEPTH + 10);

    let Err(CompareError::DepthExceeded { allowed, .. }) = align(&node, &node) else {
        bail!("Expected a depth error")
    };
    assert_eq!(allowed, MAX_COMPARISON_DEPTH);

    Ok(())
}
