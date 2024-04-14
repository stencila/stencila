//! Unit tests of diffing and patching for various types

use common::eyre::Result;
use common_dev::pretty_assertions::assert_eq;
use schema::{
    diff, patch,
    shortcuts::{art, p, t},
    Article, Block, Figure, Inline, Node, NodeProperty, Paragraph, PatchNode, PatchOp, PatchPath,
    PatchSlot, PatchValue, Primitive, Strong, Text, TimeUnit,
};

#[test]
fn atoms() -> Result<()> {
    // Boolean

    assert_eq!(diff(&true, &true)?, vec![]);

    let mut old = true;
    let new = false;
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Integer

    assert_eq!(diff(&1_i64, &1_i64)?, vec![]);

    let mut old = 1_i64;
    let new = 2_i64;
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Number

    assert_eq!(diff(&1_f64, &1_f64)?, vec![]);

    let mut old = 1_f64;
    let new = 2_f64;
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // String

    assert_eq!(diff(&String::from("abc"), &String::from("abc"))?, vec![]);

    let mut old = String::from("abc");
    let new = String::from("bcd");
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Set(new.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn options() {}

#[test]
fn vecs() -> Result<()> {
    // No ops: Both empty
    let mut old: Vec<i32> = vec![];
    let new = vec![];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Push: Old empty, new only has one
    let mut old = vec![];
    let new = vec![1];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Push(1.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Append: Old empty, new has more than one
    let mut old = vec![];
    let new = vec![1, 2, 3];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Append(vec![1.to_value()?, 2.to_value()?, 3.to_value()?])
        )]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Clear: New empty
    let mut old = vec![1, 2, 3];
    let new = vec![];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Clear)]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Remove: all different
    let mut old = vec![1, 2, 3, 4, 5, 6, 7];
    let new = vec![1, 3, 7];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Remove(vec![1, 3, 4, 5]))]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Remove: some same
    let mut old = vec![1, 1, 2, 2, 2, 3, 3, 4, 5, 5];
    let new = vec![1, 2, 3, 3, 4, 5];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(PatchPath::new(), PatchOp::Remove(vec![1, 3, 4, 9]))]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Push: adding one
    let mut old = vec![1];
    let new = vec![1, 2];
    let ops = diff(&old, &new)?;
    assert_eq!(ops, vec![(PatchPath::new(), PatchOp::Push(2.to_value()?))]);
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Append: adding more than one
    let mut old = vec![1];
    let new = vec![1, 1, 2, 3];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Append(vec![1.to_value()?, 2.to_value()?, 3.to_value()?])
        )]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Insert
    let mut old = vec![1, 3];
    let new = vec![0, 1, 2, 3, 4, 5];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Insert(vec![
                (0, 0.to_value()?),
                (2, 2.to_value()?),
                (4, 4.to_value()?),
                (5, 5.to_value()?)
            ])
        )]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    // Change: same size, all different
    let mut old = vec![1, 2];
    let new = vec![3, 4];
    let ops = diff(&old, &new)?;
    assert_eq!(
        ops,
        vec![(
            PatchPath::new(),
            PatchOp::Replace(vec![(0, 3.to_value()?), (1, 4.to_value()?),])
        )]
    );
    patch(&mut old, ops)?;
    assert_eq!(old, new);

    Ok(())
}

#[test]
fn enums() -> Result<()> {
    // Equal variants and values: no ops

    let node = Node::Article(Article::default());
    assert_eq!(diff(&node, &node)?, vec![]);

    let block = Block::Paragraph(Paragraph::default());
    assert_eq!(diff(&block, &block)?, vec![]);

    let inline = Inline::Text(Text::default());
    assert_eq!(diff(&inline, &inline)?, vec![]);

    let primitive = Primitive::Integer(1);
    assert_eq!(diff(&primitive, &primitive)?, vec![]);

    let time_unit = TimeUnit::Millisecond;
    assert_eq!(diff(&time_unit, &time_unit)?, vec![]);

    // Different variants: single replace op at root

    let node1 = Node::Article(Article::default());
    let node2 = Node::Integer(1);
    assert_eq!(
        diff(&node1, &node2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Node(node2)))]
    );

    let block1 = Block::Paragraph(Paragraph::default());
    let block2 = Block::Figure(Figure::default());
    assert_eq!(
        diff(&block1, &block2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Block(block2)))]
    );

    let inline1 = Inline::Text(Text::default());
    let inline2 = Inline::Strong(Strong::default());
    assert_eq!(
        diff(&inline1, &inline2)?,
        vec![(PatchPath::new(), PatchOp::Set(PatchValue::Inline(inline2)))]
    );

    let primitive1 = Primitive::Integer(1);
    let primitive2 = Primitive::String(String::new());
    assert_eq!(
        diff(&primitive1, &primitive2)?,
        vec![(PatchPath::new(), PatchOp::Set(primitive2.to_value()?))]
    );

    let time_unit1 = TimeUnit::Day;
    let time_unit2 = TimeUnit::Month;
    assert_eq!(
        diff(&time_unit1, &time_unit2)?,
        vec![(PatchPath::new(), PatchOp::Set(time_unit2.to_value()?))]
    );

    // Same variants, different values: ops depend differences
    use PatchSlot::*;

    let node1 = art([]);
    let node2 = art([p([t("para1")])]);
    assert_eq!(
        diff(&node1, &node2)?,
        vec![(
            PatchPath::from([Property(NodeProperty::Content)]),
            PatchOp::Push(PatchValue::Block(p([t("para1")])))
        )]
    );

    Ok(())
}
