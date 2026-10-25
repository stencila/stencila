//! Tests of the operational nesting-depth limit

use eyre::{Result, bail};

use stencila_node_compare::{CompareError, MAX_COMPARISON_DEPTH, Side, align, compare};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::NodeProperty;
use stencila_schema::{
    Array, Block, CodeChunk, Node, Object, Primitive, Section,
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

/// A dynamic value with its leaf at the specified depth, optionally inside a
/// structured property to exercise the shared depth counter.
fn dynamic(depth: usize, wrap: fn(Primitive) -> Primitive, in_property: bool) -> Node {
    let mut value = Primitive::Integer(1);
    for _ in 1..depth {
        value = wrap(value);
    }
    let value = Node::Array(Array::from([value]));
    if in_property {
        Node::CodeChunk(CodeChunk {
            outputs: Some(vec![value]),
            ..CodeChunk::new("".into())
        })
    } else {
        value
    }
}

/// The same depth limit covers nested arrays, objects, and structured properties.
#[test]
fn dynamic_nesting_obeys_the_depth_limit() -> Result<()> {
    let wrappers: [fn(Primitive) -> Primitive; 2] = [
        |value| Primitive::Array(Array::from([value])),
        |value| Primitive::Object(Object::from([("nested", value)])),
    ];
    for wrap in wrappers {
        for in_property in [false, true] {
            let depth = MAX_COMPARISON_DEPTH - usize::from(in_property);
            let at_limit = dynamic(depth, wrap, in_property);
            assert!(compare(&at_limit, &at_limit)?.is_equal());

            let too_deep = dynamic(depth + 1, wrap, in_property);
            let expected_path = if in_property {
                NodePath::from([
                    NodeSlot::Property(NodeProperty::Outputs),
                    NodeSlot::Index(0),
                ])
            } else {
                NodePath::new()
            };
            for (left, right, expected_side) in [
                (&too_deep, &at_limit, Side::Left),
                (&at_limit, &too_deep, Side::Right),
            ] {
                let Err(CompareError::DepthExceeded {
                    side,
                    path,
                    depth,
                    allowed,
                }) = compare(left, right)
                else {
                    bail!("Expected excessive dynamic nesting to return a depth error")
                };
                assert_eq!(side, expected_side);
                assert_eq!(path, expected_path);
                assert_eq!(depth, MAX_COMPARISON_DEPTH + 1);
                assert_eq!(allowed, MAX_COMPARISON_DEPTH);
            }
        }
    }
    Ok(())
}
