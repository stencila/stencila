//! Tests of mixed structured/scalar collections, and of primitive and dynamic roots

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{
    Alignment, CompareOptions, Comparison, Correspondence, Difference, PropertyPresence,
    ScalarValue, ValueState, align, align_with_options, compare,
};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Array, Block, CodeChunk, Cord, Node, Object, Primitive,
    shortcuts::{art, p, t},
};

/// A code chunk whose outputs are a mixed collection of nodes
fn chunk(outputs: Vec<Node>) -> Block {
    Block::CodeChunk(CodeChunk {
        outputs: Some(outputs),
        ..CodeChunk::new(Cord::from("plot()"))
    })
}

/// An article holding one code chunk with the given outputs
fn outputs(outputs: Vec<Node>) -> Node {
    art([chunk(outputs)])
}

/// A paragraph as an output node
fn paragraph(text: &str) -> Node {
    match p([t(text)]) {
        Block::Paragraph(paragraph) => Node::Paragraph(paragraph),
        _ => unreachable!("`p` builds a paragraph"),
    }
}

/// The path of the item at an index of the outputs of the first content item
fn output(index: usize) -> NodePath {
    NodePath::from([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
        NodeSlot::Property(NodeProperty::Outputs),
        NodeSlot::Index(index),
    ])
}

/// The paired left and right paths of an alignment
fn pairs(alignment: &Alignment) -> Vec<(NodePath, NodePath)> {
    alignment
        .pairs()
        .map(|(left, right, ..)| (left.path.clone(), right.path.clone()))
        .collect()
}

/// The value differences of a comparison, as location and both states
fn values(comparison: &Comparison) -> Vec<(Option<usize>, Option<usize>, ValueState, ValueState)> {
    comparison
        .differences()
        .iter()
        .filter_map(|difference| match difference {
            Difference::ValueChanged {
                location,
                left,
                right,
            } if location.property == Some(NodeProperty::Outputs) => Some((
                location.left_index,
                location.right_index,
                left.clone(),
                right.clone(),
            )),
            _ => None,
        })
        .collect()
}

/// Sequence alignment operates over both structured and scalar tokens
#[test]
fn a_mixed_collection_aligns_all_its_tokens() -> Result<()> {
    let left = outputs(vec![
        Node::Integer(42),
        paragraph("A distinctive and unique sentence"),
    ]);
    let right = outputs(vec![
        Node::Integer(42),
        paragraph("A distinctive and unique sentence"),
    ]);

    let alignment = align(&left, &right)?;

    // The structured item is paired, and nothing is left over
    assert!(!alignment.has_one_sided());
    assert!(pairs(&alignment).contains(&(output(1), output(1))));

    Ok(())
}

/// A scalar item never becomes a correspondence record
#[test]
fn scalar_items_produce_no_correspondences() -> Result<()> {
    let left = outputs(vec![Node::Integer(1), Node::Integer(2)]);
    let right = outputs(vec![Node::Integer(1), paragraph("Added")]);

    let alignment = align(&left, &right)?;

    // Only the article, the code chunk, the added paragraph and its text are
    // correspondences; the two scalar outputs are not
    for correspondence in alignment.correspondences() {
        for node in [correspondence.left(), correspondence.right()]
            .into_iter()
            .flatten()
        {
            assert_ne!(node.path, output(0));
            if node.node_type != NodeType::Paragraph {
                assert_ne!(node.path, output(1));
            }
        }
    }

    Ok(())
}

/// A structured item is never paired with a scalar item
#[test]
fn structured_and_scalar_items_never_pair() -> Result<()> {
    let left = outputs(vec![paragraph("A distinctive and unique sentence")]);
    let right = outputs(vec![Node::String(
        "A distinctive and unique sentence".to_string(),
    )]);

    let alignment = align(&left, &right)?;

    // The paragraph has no counterpart at all, rather than being paired with the
    // string that happens to carry the same text
    let Some(Correspondence::LeftOnly { left, .. }) = alignment
        .correspondences()
        .iter()
        .find(|correspondence| correspondence.left().map(|node| &node.path) == Some(&output(0)))
    else {
        bail!("Expected the paragraph to be left-only")
    };
    assert_eq!(left.node_type, NodeType::Paragraph);

    Ok(())
}

/// Paired and one-sided scalar branches produce indexed value observations carrying
/// typed values or absence
#[test]
fn scalar_items_produce_indexed_value_observations() -> Result<()> {
    // An edited scalar item is still similar enough to pair, so both typed values are
    // carried by one observation indexed on both sides
    let left = outputs(vec![
        Node::String("The quick brown fox".to_string()),
        paragraph("A distinctive and unique sentence"),
    ]);
    let right = outputs(vec![
        Node::String("The quick brown foxes".to_string()),
        paragraph("A distinctive and unique sentence"),
    ]);

    let comparison = compare(&left, &right)?;

    assert_eq!(
        values(&comparison),
        vec![(
            Some(0),
            Some(0),
            ValueState::One {
                value: ScalarValue::string("The quick brown fox")
            },
            ValueState::One {
                value: ScalarValue::string("The quick brown foxes")
            }
        )]
    );

    // A scalar with no counterpart is one side present and the other absent
    let left = outputs(vec![
        paragraph("A distinctive and unique sentence"),
        Node::Integer(7),
    ]);
    let right = outputs(vec![paragraph("A distinctive and unique sentence")]);

    let comparison = compare(&left, &right)?;
    assert_eq!(
        values(&comparison),
        vec![(
            Some(1),
            None,
            ValueState::One {
                value: ScalarValue::Integer { value: 7 }
            },
            ValueState::Absent
        )]
    );

    Ok(())
}

/// A union-valued collection remains mixed even when both snapshots currently hold
/// only scalar branches
#[test]
fn scalar_only_union_items_remain_indexed() -> Result<()> {
    let left = outputs(vec![Node::Integer(1), Node::String("kept".to_string())]);
    let right = outputs(vec![Node::Integer(2), Node::String("kept".to_string())]);

    let comparison = compare(&left, &right)?;
    assert_eq!(
        values(&comparison),
        vec![
            (
                None,
                Some(0),
                ValueState::Absent,
                ValueState::One {
                    value: ScalarValue::Integer { value: 2 }
                }
            ),
            (
                Some(0),
                None,
                ValueState::One {
                    value: ScalarValue::Integer { value: 1 }
                },
                ValueState::Absent
            ),
        ]
    );

    Ok(())
}

/// Two wholly dissimilar scalar items are two one-sided observations rather than one
/// pair, because the same cost model governs scalar and structured items alike
#[test]
fn dissimilar_scalar_items_are_two_observations() -> Result<()> {
    let left = outputs(vec![
        Node::Integer(1),
        paragraph("A distinctive and unique sentence"),
    ]);
    let right = outputs(vec![
        Node::Integer(2),
        paragraph("A distinctive and unique sentence"),
    ]);

    let comparison = compare(&left, &right)?;

    assert_eq!(
        values(&comparison),
        vec![
            (
                None,
                Some(0),
                ValueState::Absent,
                ValueState::One {
                    value: ScalarValue::Integer { value: 2 }
                }
            ),
            (
                Some(0),
                None,
                ValueState::One {
                    value: ScalarValue::Integer { value: 1 }
                },
                ValueState::Absent
            ),
        ]
    );

    Ok(())
}

/// An exact scalar item anchors the structured items around it
#[test]
fn an_exact_scalar_item_anchors() -> Result<()> {
    // Every paragraph is edited, so none of them can anchor; only the integer is an
    // exact unique item on both sides
    let texts = ["Alpha one", "Beta two", "Gamma three", "Delta four"];
    let edited = ["Alpha ONE", "Beta TWO", "Gamma THREE", "Delta FOUR"];
    let mixed = |texts: [&str; 4]| {
        outputs(vec![
            paragraph(texts[0]),
            paragraph(texts[1]),
            Node::Integer(42),
            paragraph(texts[2]),
            paragraph(texts[3]),
        ])
    };
    let left = mixed(texts);
    let right = mixed(edited);

    // The anchor partitions five items into two segments of two, which is eight cells
    // rather than the twenty-five an unpartitioned alignment would need. The budget is
    // cumulative across the run, so it also covers the one cell the article's own
    // content sequence costs, and the one cell each paired paragraph's own content
    // costs.
    let options = CompareOptions {
        alignment_cell_budget: 1 + 8 + 4,
        ..Default::default()
    };
    let alignment = align_with_options(&left, &right, &options)?;

    assert_eq!(
        pairs(&alignment)
            .into_iter()
            .filter(|(left, ..)| left.len() == 4)
            .collect::<Vec<_>>(),
        vec![
            (output(0), output(0)),
            (output(1), output(1)),
            (output(3), output(3)),
            (output(4), output(4)),
        ]
    );

    // Without the anchor there is no budget for the same alignment
    let unanchored = outputs(vec![
        paragraph(texts[0]),
        paragraph(texts[1]),
        Node::Integer(42),
        paragraph(texts[2]),
        paragraph(texts[3]),
    ]);
    let unanchored_right = outputs(vec![
        paragraph(edited[0]),
        paragraph(edited[1]),
        Node::Integer(43),
        paragraph(edited[2]),
        paragraph(edited[3]),
    ]);
    assert!(align_with_options(&unanchored, &unanchored_right, &options).is_err());

    Ok(())
}

/// A homogeneous repeated scalar property is one sequence-valued difference
#[test]
fn a_homogeneous_scalar_property_is_one_difference() -> Result<()> {
    let keywords = |values: &[&str]| {
        let mut article = stencila_schema::Article::new(Vec::new());
        article.options.keywords = Some(values.iter().map(|value| value.to_string()).collect());
        Node::Article(article)
    };

    let comparison = compare(&keywords(&["one", "two"]), &keywords(&["one", "three"]))?;

    let keyword_differences: Vec<&Difference> = comparison
        .differences()
        .iter()
        .filter(|difference| difference.property() == Some(NodeProperty::Keywords))
        .collect();
    assert_eq!(keyword_differences.len(), 1);

    let Some(Difference::ValueChanged {
        location,
        left,
        right,
    }) = keyword_differences.first()
    else {
        bail!("Expected one value difference for the whole sequence")
    };
    assert_eq!(location.left_index, None);
    assert_eq!(location.right_index, None);
    assert_eq!(
        left,
        &ValueState::Many {
            values: vec![ScalarValue::string("one"), ScalarValue::string("two")]
        }
    );
    assert_eq!(
        right,
        &ValueState::Many {
            values: vec![ScalarValue::string("one"), ScalarValue::string("three")]
        }
    );

    Ok(())
}

/// Nested scalar properties stay values and never become alignable occurrences
#[test]
fn nested_scalars_are_not_occurrences() -> Result<()> {
    let node = Node::Object(Object::from([
        ("a", Primitive::Integer(1)),
        (
            "b",
            Primitive::Array(Array::from([Primitive::String("two".to_string())])),
        ),
    ]));

    let alignment = align(&node, &node)?;

    // The root correspondence is the only record: nothing inside the dynamic object
    // became an occurrence
    assert_eq!(alignment.correspondences().len(), 1);

    Ok(())
}

/// Every primitive, dynamic array and dynamic object root aligns and compares
#[test]
fn all_roots_align_and_compare() -> Result<()> {
    let nodes = [
        Node::Null(stencila_schema::Null),
        Node::Boolean(true),
        Node::Integer(42),
        Node::UnsignedInteger(42),
        Node::Number(1.5),
        Node::String("Hello".to_string()),
        Node::Array(Array::from([Primitive::Integer(1)])),
        Node::Object(Object::from([("a", Primitive::Integer(1))])),
        art([p([t("Hello")])]),
    ];

    for left in &nodes {
        for right in &nodes {
            let comparison = compare(left, right)?;

            // The selected roots always receive a root correspondence, whatever their
            // variants, and it always pairs them
            let root = comparison
                .alignment()
                .correspondences()
                .iter()
                .find(|correspondence| {
                    matches!(correspondence, Correspondence::Paired { left, right, .. }
                        if left.path.is_empty() && right.path.is_empty())
                });
            if root.is_none() {
                bail!("Expected a root correspondence for {left:?} and {right:?}")
            }

            // Swapping the inputs and inverting the output is the same artifact
            assert_eq!(comparison, compare(right, left)?.invert());
        }
    }

    Ok(())
}

/// Two same-type primitive roots produce a root value difference
#[test]
fn same_type_primitive_roots_compare_by_value() -> Result<()> {
    let comparison = compare(&Node::Integer(1), &Node::Integer(2))?;

    assert_eq!(comparison.differences().len(), 1);
    let Some(Difference::ValueChanged {
        location,
        left,
        right,
    }) = comparison.differences().first()
    else {
        bail!("Expected a root value difference")
    };
    assert_eq!(location.property, None);
    assert_eq!(location.left.path, NodePath::default());
    assert_eq!(location.left.node_type, NodeType::Integer);
    assert_eq!(
        left,
        &ValueState::One {
            value: ScalarValue::Integer { value: 1 }
        }
    );
    assert_eq!(
        right,
        &ValueState::One {
            value: ScalarValue::Integer { value: 2 }
        }
    );

    // Equal primitive roots are difference free
    assert!(compare(&Node::Integer(1), &Node::Integer(1))?.is_equal());

    Ok(())
}

/// An incompatible root pair is a node type change, with no structural recursion
#[test]
fn incompatible_roots_change_node_type() -> Result<()> {
    for (left, right) in [
        (Node::Integer(1), Node::String("one".to_string())),
        (
            Node::Array(Array::from([Primitive::Integer(1)])),
            Node::Object(Object::from([("a", Primitive::Integer(1))])),
        ),
    ] {
        let comparison = compare(&left, &right)?;
        assert_eq!(comparison.differences().len(), 1);
        assert!(matches!(
            comparison.differences().first(),
            Some(Difference::NodeTypeChanged { .. })
        ));
    }

    // A primitive against a structured root records the node type change, and the
    // structured root's contents are one-sided rather than compared against the scalar
    let comparison = compare(&Node::Integer(1), &art([p([t("Hello")])]))?;
    let node_types: Vec<&Difference> = comparison
        .differences()
        .iter()
        .filter(|difference| matches!(difference, Difference::NodeTypeChanged { .. }))
        .collect();
    assert_eq!(node_types.len(), 1);
    assert!(
        !comparison.differences().iter().any(|difference| matches!(
            difference,
            Difference::PropertyPresenceChanged { .. } | Difference::ValueChanged { .. }
        )),
        "the contents of the structured root are not forced into a comparison"
    );
    assert!(comparison.alignment().has_one_sided());

    Ok(())
}

/// Complete coverage, canonical ordering and swap symmetry hold for mixed collections
#[test]
fn mixed_collections_keep_the_invariants() -> Result<()> {
    let left = outputs(vec![
        Node::Integer(1),
        paragraph("A distinctive and unique sentence"),
        Node::String("kept".to_string()),
        paragraph("Another wholly different sentence"),
    ]);
    let right = outputs(vec![
        paragraph("A distinctive and unique sentence"),
        Node::Integer(2),
        Node::String("kept".to_string()),
    ]);

    let comparison = compare(&left, &right)?;

    assert!(comparison.alignment().correspondences().is_sorted());
    assert!(comparison.differences().is_sorted());

    assert_eq!(comparison, compare(&right, &left)?.invert());

    comparison.validate(&left, &right)?;

    // Presence differences are only about declared properties, never about scalars
    assert!(!comparison.differences().iter().any(|difference| matches!(
        difference,
        Difference::PropertyPresenceChanged {
            left_presence: PropertyPresence::Undeclared,
            ..
        }
    )));

    Ok(())
}
