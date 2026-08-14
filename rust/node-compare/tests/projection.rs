//! Tests of the canonical projection and the typed scalar model

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{
    CanonicalNumber, CompareError, ScalarValue, Side,
    projection::{Item, Occurrence, Presence, Projection, Root},
    projections_equal,
};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Array, Cord, CordAuthorship, ExecutionMode, Node, Object, Paragraph, Primitive, Text,
    shortcuts::{art, p, sec, t},
};

/// Project a node, or fail the test
fn project(node: &Node) -> Result<Projection> {
    Ok(Projection::new(node, Side::Left)?)
}

/// The occurrence at a path, if any
fn at<'projection>(
    projection: &'projection Projection,
    path: &NodePath,
) -> Option<&'projection Occurrence> {
    projection
        .occurrences()
        .iter()
        .find(|occurrence| &occurrence.path == path)
}

/// The single scalar item of a property of an occurrence
fn scalar(occurrence: &Occurrence, property: NodeProperty) -> Result<Option<&ScalarValue>> {
    for projected in &occurrence.properties {
        if projected.decl.property != property {
            continue;
        }
        return Ok(match (projected.presence, projected.items.first()) {
            (Presence::Absent, ..) => None,
            (Presence::Present, Some(Item::Scalar(value))) => Some(value),
            _ => bail!("Property `{property}` is not a singular present scalar"),
        });
    }
    bail!("Property `{property}` is not projected")
}

/// Every `Node` variant projects, including primitive, array and object variants
#[test]
fn all_root_variants_project() -> Result<()> {
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

    for node in nodes {
        let projection = project(&node)?;
        // A projection always has a root
        match projection.root() {
            Root::Structured(id) => {
                projection.occurrence(*id)?;
            }
            Root::Scalar(..) => {
                assert!(projection.occurrences().is_empty());
            }
        }
    }

    Ok(())
}

/// Projection equality is reflexive for a node and for its clone
#[test]
fn equality_is_reflexive() -> Result<()> {
    let node = art([sec([p([t("Hello")])]), p([t("World")])]);

    assert!(projections_equal(&node, &node)?);
    assert!(projections_equal(&node, &node.clone())?);

    Ok(())
}

/// Union wrappers and `*Options` structs add no occurrences
#[test]
fn wrappers_add_no_occurrences() -> Result<()> {
    // `Article` (1) with `content` holding one `Paragraph` (2) holding one `Text` (3).
    // The `Node`, `Block` and `Inline` union wrappers, and `ArticleOptions`, add none.
    let projection = project(&art([p([t("Hello")])]))?;
    assert_eq!(projection.occurrences().len(), 3);

    assert_eq!(
        projection.occurrences()[0].node_type,
        NodeType::Article,
        "the root is the article"
    );
    assert_eq!(projection.occurrences()[1].node_type, NodeType::Paragraph);
    assert_eq!(projection.occurrences()[2].node_type, NodeType::Text);

    Ok(())
}

/// Paths use typed slots and address occurrences through their containing property
#[test]
fn paths_use_typed_slots() -> Result<()> {
    let projection = project(&art([p([t("Hello")])]))?;

    let paragraph = NodePath::from([
        NodeSlot::Property(NodeProperty::Content),
        NodeSlot::Index(0),
    ]);
    assert_eq!(
        at(&projection, &paragraph).map(|occurrence| occurrence.node_type),
        Some(NodeType::Paragraph)
    );

    let mut text = paragraph.clone();
    text.push_back(NodeSlot::Property(NodeProperty::Content));
    text.push_back(NodeSlot::Index(0));
    assert_eq!(
        at(&projection, &text).map(|occurrence| occurrence.node_type),
        Some(NodeType::Text)
    );

    Ok(())
}

/// The `uid` and the `type` discriminator are absent from the projection
#[test]
fn intrinsic_machinery_is_absent() -> Result<()> {
    let projection = project(&art([p([t("Hello")])]))?;

    for occurrence in projection.occurrences() {
        for projected in &occurrence.properties {
            let name = projected.decl.property.to_string();
            assert_ne!(name, "Uid");
            assert_ne!(name, "Type");
        }
    }

    // Two nodes differing only by `uid` are equal
    let left = art([p([t("Hello")])]);
    let mut right = art([p([t("Hello")])]);
    if let Node::Article(article) = &mut right {
        article.uid = Default::default();
    }
    assert!(projections_equal(&left, &right)?);

    Ok(())
}

/// A `Cord` projects to its string only, and its authorship is ignored entirely
#[test]
fn cord_projects_to_its_string() -> Result<()> {
    let mut with_authorship = Text::from("Hello");
    with_authorship.value = Cord {
        string: "Hello".to_string(),
        authorship: vec![CordAuthorship::new(1, 1, 1, 5)],
    };

    let projection = project(&Node::Text(with_authorship.clone()))?;
    let Root::Structured(id) = projection.root() else {
        bail!("Expected a structured root")
    };
    let occurrence = projection.occurrence(*id)?;
    assert_eq!(
        scalar(occurrence, NodeProperty::Value)?,
        Some(&ScalarValue::string("Hello"))
    );

    // An authorship-only change is not a difference
    let without_authorship = Text::from("Hello");
    assert!(projections_equal(
        &Node::Text(with_authorship.clone()),
        &Node::Text(without_authorship)
    )?);

    // A string change is
    let changed = Text::from("Goodbye");
    assert!(!projections_equal(
        &Node::Text(with_authorship),
        &Node::Text(changed)
    )?);

    Ok(())
}

/// Explicit `id`, provenance, execution state and compilation messages are projected
/// by default
#[test]
fn declared_schema_data_is_projected() -> Result<()> {
    let mut paragraph = Paragraph::new(vec![t("Hello")]);
    paragraph.id = Some("para-1".to_string());

    let projection = project(&Node::Paragraph(paragraph))?;
    let Root::Structured(id) = projection.root() else {
        bail!("Expected a structured root")
    };
    let occurrence = projection.occurrence(*id)?;

    assert_eq!(
        scalar(occurrence, NodeProperty::Id)?,
        Some(&ScalarValue::string("para-1"))
    );
    // Present but absent-valued properties are still projected
    assert!(scalar(occurrence, NodeProperty::Provenance)?.is_none());

    // An explicit `id` is an ordinary compared value
    let mut other = Paragraph::new(vec![t("Hello")]);
    other.id = Some("para-2".to_string());
    assert!(!projections_equal(
        &Node::Paragraph(Paragraph {
            id: Some("para-1".to_string()),
            ..Paragraph::new(vec![t("Hello")])
        }),
        &Node::Paragraph(other)
    )?);

    // Execution state is projected: an article declares `execution_mode`
    let projection = project(&art([p([t("Hello")])]))?;
    let Root::Structured(root) = projection.root() else {
        bail!("Expected a structured root")
    };
    let article = projection.occurrence(*root)?;
    assert!(
        article
            .properties
            .iter()
            .any(|projected| projected.decl.property == NodeProperty::ExecutionMode)
    );
    assert!(
        article
            .properties
            .iter()
            .any(|projected| projected.decl.property == NodeProperty::CompilationMessages)
    );

    Ok(())
}

/// Absence is distinguished from a present null, and from a present empty sequence
#[test]
fn absence_is_distinguished() -> Result<()> {
    let absent = Paragraph::new(vec![t("Hello")]);
    let empty = Paragraph {
        authors: Some(Vec::new()),
        ..Paragraph::new(vec![t("Hello")])
    };

    assert!(!projections_equal(
        &Node::Paragraph(absent),
        &Node::Paragraph(empty)
    )?);

    // A present null is a scalar value, not absence
    let null = project(&Node::Null(stencila_schema::Null))?;
    assert_eq!(null.root(), &Root::Scalar(ScalarValue::Null));

    Ok(())
}

/// The scalar model preserves scalar type and union-variant identity
#[test]
fn scalar_types_are_preserved() -> Result<()> {
    let integer = project(&Node::Integer(1))?;
    let unsigned = project(&Node::UnsignedInteger(1))?;
    let number = project(&Node::Number(1.))?;
    let string = project(&Node::String("1".to_string()))?;
    let boolean = project(&Node::Boolean(true))?;

    assert_eq!(
        integer.root(),
        &Root::Scalar(ScalarValue::Integer { value: 1 })
    );
    assert_eq!(
        unsigned.root(),
        &Root::Scalar(ScalarValue::UnsignedInteger { value: 1 })
    );
    assert_eq!(number.root(), &Root::Scalar(ScalarValue::number(1.)));
    assert_eq!(string.root(), &Root::Scalar(ScalarValue::string("1")));
    assert_eq!(
        boolean.root(),
        &Root::Scalar(ScalarValue::Boolean { value: true })
    );

    // Same underlying value, different union variants, are not equal
    assert!(!projections_equal(
        &Node::Integer(1),
        &Node::UnsignedInteger(1)
    )?);
    assert!(!projections_equal(&Node::Integer(1), &Node::Number(1.))?);
    assert!(!projections_equal(
        &Node::Integer(1),
        &Node::String("1".to_string())
    )?);

    Ok(())
}

/// Schema enums project as enum scalars carrying their type and variant
#[test]
fn schema_enums_are_projected() -> Result<()> {
    let mut article = match art([p([t("Hello")])]) {
        Node::Article(article) => article,
        _ => bail!("Expected an article"),
    };
    article.execution_mode = Some(ExecutionMode::Always);

    let projection = project(&Node::Article(article))?;
    let Root::Structured(id) = projection.root() else {
        bail!("Expected a structured root")
    };
    let occurrence = projection.occurrence(*id)?;

    assert_eq!(
        scalar(occurrence, NodeProperty::ExecutionMode)?,
        Some(&ScalarValue::Enum {
            schema_type: "ExecutionMode".to_string(),
            variant: "Always".to_string()
        })
    );

    Ok(())
}

/// Dynamic arrays and objects are preserved, with object entries ordered by key
#[test]
fn dynamic_values_are_preserved() -> Result<()> {
    let array = project(&Node::Array(Array::from([
        Primitive::Integer(1),
        Primitive::String("two".to_string()),
    ])))?;
    assert_eq!(
        array.root(),
        &Root::Scalar(ScalarValue::Array {
            items: vec![
                ScalarValue::Integer { value: 1 },
                ScalarValue::string("two")
            ]
        })
    );

    // Insertion order does not affect the projection
    let one = Node::Object(Object::from([
        ("b", Primitive::Integer(2)),
        ("a", Primitive::Integer(1)),
    ]));
    let other = Node::Object(Object::from([
        ("a", Primitive::Integer(1)),
        ("b", Primitive::Integer(2)),
    ]));
    assert_eq!(project(&one)?.root(), project(&other)?.root());
    assert!(projections_equal(&one, &other)?);

    // Nested dynamic values are preserved
    let nested = Node::Object(Object::from([(
        "a",
        Primitive::Array(Array::from([Primitive::Null(stencila_schema::Null)])),
    )]));
    assert_eq!(
        project(&nested)?.root(),
        &Root::Scalar(ScalarValue::object([(
            "a".to_string(),
            ScalarValue::Array {
                items: vec![ScalarValue::Null]
            }
        )]))
    );

    Ok(())
}

/// NaN is reflexive, signed zeros are equal, infinity keeps its sign, and every other
/// finite value is exact
#[test]
fn canonical_numbers() -> Result<()> {
    assert_eq!(
        CanonicalNumber::new(f64::NAN),
        CanonicalNumber::new(f64::NAN)
    );
    assert!(projections_equal(
        &Node::Number(f64::NAN),
        &Node::Number(f64::NAN)
    )?);

    assert_eq!(CanonicalNumber::new(0.), CanonicalNumber::new(-0.));
    assert!(projections_equal(&Node::Number(0.), &Node::Number(-0.))?);

    assert_ne!(
        CanonicalNumber::new(f64::INFINITY),
        CanonicalNumber::new(f64::NEG_INFINITY)
    );
    assert!(!projections_equal(
        &Node::Number(f64::INFINITY),
        &Node::Number(f64::NEG_INFINITY)
    )?);
    assert!(projections_equal(
        &Node::Number(f64::INFINITY),
        &Node::Number(f64::INFINITY)
    )?);

    assert_ne!(
        CanonicalNumber::new(0.1 + 0.2),
        CanonicalNumber::new(0.30000000000000004_f64.next_up())
    );
    assert_eq!(CanonicalNumber::new(0.1 + 0.2).get(), 0.1 + 0.2);

    Ok(())
}

/// Non-finite numbers serialize explicitly and round trip
#[test]
fn non_finite_numbers_serialize_explicitly() -> Result<()> {
    for (number, expected) in [
        (f64::NAN, r#""NaN""#),
        (f64::INFINITY, r#""Infinity""#),
        (f64::NEG_INFINITY, r#""-Infinity""#),
        (1.5, "1.5"),
    ] {
        let number = CanonicalNumber::new(number);
        let json = serde_json::to_string(&number)?;
        assert_eq!(json, expected);
        assert_eq!(serde_json::from_str::<CanonicalNumber>(&json)?, number);
    }

    Ok(())
}

/// A projection error carries its side and path
#[test]
fn projection_errors_carry_context() -> Result<()> {
    let error = CompareError::Projection {
        side: Side::Right,
        path: NodePath::from([NodeSlot::Property(NodeProperty::Content)]),
        message: "value is neither a structured occurrence nor a scalar".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "Unable to project the right node at path `content`: \
         value is neither a structured occurrence nor a scalar"
    );

    Ok(())
}

/// Scalar values serialize with camel case fields, and round trip
#[test]
fn scalars_serialize_canonically() -> Result<()> {
    let value = ScalarValue::Enum {
        schema_type: "LabelType".to_string(),
        variant: "AppendixLabel".to_string(),
    };
    assert_eq!(
        serde_json::to_string(&value)?,
        r#"{"type":"enum","schemaType":"LabelType","variant":"AppendixLabel"}"#
    );
    assert_eq!(
        serde_json::from_str::<ScalarValue>(
            r#"{"type":"enum","schemaType":"LabelType","variant":"AppendixLabel"}"#
        )?,
        value
    );

    Ok(())
}

/// Canonical ordering of dynamic object entries holds after deserialization, not only
/// in memory
#[test]
fn object_entries_are_canonical_after_deserialization() -> Result<()> {
    let canonical = ScalarValue::object([
        ("a".to_string(), ScalarValue::Integer { value: 1 }),
        ("b".to_string(), ScalarValue::Integer { value: 2 }),
    ]);

    let out_of_order = r#"{"type":"object","entries":[["b",{"type":"integer","value":2}],["a",{"type":"integer","value":1}]]}"#;
    assert_eq!(
        serde_json::from_str::<ScalarValue>(out_of_order)?,
        canonical
    );

    Ok(())
}
