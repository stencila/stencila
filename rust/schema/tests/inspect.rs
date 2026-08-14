//! Tests of the `InspectType`/`InspectNode` schema introspection seam

use std::collections::HashSet;

use eyre::{Result, bail};
use pretty_assertions::assert_eq;
use strum::IntoEnumIterator;

use stencila_schema::{
    Article, Block, Cord, Heading, Inline, InspectNode, InspectType, InspectValue, Node, NodeType,
    Paragraph, PropertyDecl, ScalarRef, Table, TableCell, TableRow, Text, ValueKind,
    inspect_declared_properties,
};
use stencila_schema::{NodeProperty, shortcuts::t};

/// Find the declaration for a property of a node
fn decl(node: &dyn InspectNode, property: NodeProperty) -> Result<PropertyDecl> {
    for prop in node.properties() {
        if prop.decl.property == property {
            return Ok(prop.decl);
        }
    }
    bail!("Property `{property}` is not reported")
}

/// The properties reported for a node, in order
fn properties(node: &dyn InspectNode) -> Vec<NodeProperty> {
    node.properties()
        .into_iter()
        .map(|prop| prop.decl.property)
        .collect()
}

/// The value of a property of a node
fn value<'node>(
    node: &'node dyn InspectNode,
    property: NodeProperty,
) -> Result<InspectValue<'node>> {
    for prop in node.properties() {
        if prop.decl.property == property {
            return Ok(prop.value);
        }
    }
    bail!("Property `{property}` is not reported")
}

/// Whether `items` appears in `sequence`, in order but not necessarily contiguously
fn is_subsequence(items: &[NodeProperty], sequence: &[NodeProperty]) -> bool {
    let mut sequence = sequence.iter();
    items
        .iter()
        .all(|item| sequence.any(|candidate| candidate == item))
}

/// The length of the longest prefix of `items` that is a subsequence of `sequence`
fn longest_subsequence_prefix(items: &[NodeProperty], sequence: &[NodeProperty]) -> usize {
    let mut sequence = sequence.iter();
    for (index, item) in items.iter().enumerate() {
        if !sequence.any(|candidate| candidate == item) {
            return index;
        }
    }
    items.len()
}

/// Every node type that declares properties reports exactly those properties
///
/// This is the generator-level check: `NodeType::properties` is generated from the
/// schema definitions, while `inspect_declared_properties` dispatches to the
/// `InspectType` implementation derived from the generated Rust type. A newly
/// generated schema field that was invisible to introspection would show up here.
#[test]
fn declared_properties_cover_the_schema() -> Result<()> {
    for node_type in NodeType::iter() {
        let mut expected = node_type.properties();
        let Some(declared) = inspect_declared_properties(node_type) else {
            if !expected.is_empty() {
                bail!("`{node_type}` declares properties but is not introspectable")
            }
            continue;
        };

        let actual: Vec<NodeProperty> = declared.iter().map(|decl| decl.property).collect();

        // Compared as sets, because properties of a flattened `*Options` struct are
        // reported after those of the owning struct, whereas the schema interleaves
        // them
        let mut sorted_actual: Vec<String> =
            actual.iter().map(|property| property.to_string()).collect();
        sorted_actual.sort();
        let mut sorted_expected: Vec<String> = expected
            .drain(..)
            .map(|property| property.to_string())
            .collect();
        sorted_expected.sort();
        assert_eq!(
            sorted_actual, sorted_expected,
            "for node type `{node_type}`"
        );

        // Order is checked as well as membership: the reported properties are those
        // of the owning struct followed by those of its flattened options, and each
        // of those two runs must preserve the schema's relative order
        let expected = node_type.properties();
        let split = longest_subsequence_prefix(&actual, &expected);
        if !is_subsequence(&actual[split..], &expected) {
            bail!(
                "`{node_type}` reports properties in an order that is not the schema's \
                 order for the owning struct followed by the schema's order for its options"
            )
        }
    }

    Ok(())
}

/// Declared properties are unique within a type, including across flattened options
#[test]
fn declared_properties_are_unique() -> Result<()> {
    for node_type in NodeType::iter() {
        let Some(declared) = inspect_declared_properties(node_type) else {
            continue;
        };

        let mut seen = HashSet::new();
        for decl in declared {
            if !seen.insert(decl.property) {
                bail!("`{node_type}` reports `{}` more than once", decl.property)
            }
        }
    }

    Ok(())
}

/// The declarations of a value match the declarations of its type
#[test]
fn value_and_type_declarations_agree() -> Result<()> {
    let paragraph = Paragraph::new(vec![t("Hello")]);
    let from_value: Vec<PropertyDecl> = paragraph
        .properties()
        .into_iter()
        .map(|prop| prop.decl)
        .collect();
    assert_eq!(
        from_value,
        <Paragraph as InspectType>::declared_properties()
    );

    Ok(())
}

/// Required, optional, singular, repeated, structured, scalar and union properties
/// are all distinguished
#[test]
fn property_shapes() -> Result<()> {
    let heading = Heading::new(1, vec![t("Hello")]);

    // Required, singular, scalar: `level` is an `Integer`
    let level = decl(&heading, NodeProperty::Level)?;
    assert!(level.required);
    assert!(!level.repeated);
    assert_eq!(level.kind, ValueKind::Scalar);

    // Required, repeated, union: `content` is a `Vec<Inline>`
    let content = decl(&heading, NodeProperty::Content)?;
    assert!(content.required);
    assert!(content.repeated);
    assert_eq!(content.kind, ValueKind::Union);

    // Optional, singular, scalar: `id` is an `Option<String>`
    let id = decl(&heading, NodeProperty::Id)?;
    assert!(!id.required);
    assert!(!id.repeated);
    assert_eq!(id.kind, ValueKind::Scalar);

    // Optional, singular, scalar: `label_type` is an `Option<LabelType>` schema enum
    let label_type = decl(&heading, NodeProperty::LabelType)?;
    assert!(!label_type.required);
    assert!(!label_type.repeated);
    assert_eq!(label_type.kind, ValueKind::Scalar);

    // Optional, repeated, union: `authors` is an `Option<Vec<Author>>`
    let authors = decl(&heading, NodeProperty::Authors)?;
    assert!(!authors.required);
    assert!(authors.repeated);
    assert_eq!(authors.kind, ValueKind::Union);

    // Optional, repeated, structured: `provenance` is an `Option<Vec<ProvenanceCount>>`
    let provenance = decl(&heading, NodeProperty::Provenance)?;
    assert!(!provenance.required);
    assert!(provenance.repeated);
    assert_eq!(provenance.kind, ValueKind::Structured);

    // Required, singular, structured: a `TableCell`'s `content` is repeated `Block`s,
    // but a `TableRow`'s `cells` are structured
    let row = TableRow::new(vec![TableCell::new(vec![])]);
    let cells = decl(&row, NodeProperty::Cells)?;
    assert!(cells.required);
    assert!(cells.repeated);
    assert_eq!(cells.kind, ValueKind::Structured);

    Ok(())
}

/// Absence is distinguished from a present but empty sequence
#[test]
fn absence_is_distinguished_from_empty() -> Result<()> {
    let mut paragraph = Paragraph::new(Vec::new());

    // `content` is required and present, though empty
    assert!(matches!(
        value(&paragraph, NodeProperty::Content)?,
        InspectValue::Many(items) if items.is_empty()
    ));

    // `authors` is optional and absent
    assert!(matches!(
        value(&paragraph, NodeProperty::Authors)?,
        InspectValue::Absent
    ));

    // `authors` present, but empty, is not absent
    paragraph.authors = Some(Vec::new());
    assert!(matches!(
        value(&paragraph, NodeProperty::Authors)?,
        InspectValue::Many(items) if items.is_empty()
    ));

    Ok(())
}

/// Properties that are not marked `#[walk]` are still visible
#[test]
fn properties_without_walk_are_visible() -> Result<()> {
    // Neither `level` nor `label` is marked `#[walk]` on `Heading`
    let heading = Heading::new(3, vec![t("Hello")]);
    let reported = properties(&heading);
    assert!(reported.contains(&NodeProperty::Level));
    assert!(reported.contains(&NodeProperty::Label));

    Ok(())
}

/// Properties declared on a flattened `*Options` struct are reported as properties
/// of the owning type
#[test]
fn options_properties_are_flattened() -> Result<()> {
    let table = Table::new(Vec::new());
    let reported = properties(&table);

    // Declared directly on `Table`
    assert!(reported.contains(&NodeProperty::Rows));
    // Declared on `TableOptions`
    assert!(reported.contains(&NodeProperty::AlternateNames));

    // The `options` field itself is not a property; `NodeProperty` has no `Options`
    // variant, so check by name
    assert!(
        !reported
            .iter()
            .any(|property| property.to_string() == "Options")
    );

    Ok(())
}

/// `uid` and the `type` discriminator are not properties
#[test]
fn intrinsic_machinery_is_not_reported() -> Result<()> {
    for reported in [
        properties(&Paragraph::new(vec![t("Hello")])),
        properties(&Table::new(Vec::new())),
        properties(&Article::new(Vec::new())),
    ] {
        assert!(!reported.iter().any(|property| {
            let name = property.to_string();
            name == "Uid" || name == "Type"
        }));
    }

    Ok(())
}

/// Union enums are transparent wrappers around their selected branch
#[test]
fn unions_report_their_selected_branch() -> Result<()> {
    let block = Block::Paragraph(Paragraph::new(vec![t("Hello")]));
    assert_eq!(InspectNode::node_type(&block), Some(NodeType::Paragraph));
    assert_eq!(
        properties(&block),
        properties(&Paragraph::new(vec![t("Hello")]))
    );

    let inline = Inline::Text(Text::from("Hello"));
    assert_eq!(InspectNode::node_type(&inline), Some(NodeType::Text));

    let node = Node::Heading(Heading::new(1, Vec::new()));
    assert_eq!(InspectNode::node_type(&node), Some(NodeType::Heading));

    // A union branch that is a scalar is reported as a scalar, with no node type
    let node = Node::Integer(42);
    assert_eq!(InspectNode::node_type(&node), None);
    assert_eq!(node.scalar(), Some(ScalarRef::Integer(42)));

    Ok(())
}

/// Flattened `*Options` containers are not occurrences of their own
#[test]
fn options_are_not_occurrences() -> Result<()> {
    let table = Table::new(Vec::new());
    assert_eq!(InspectNode::node_type(&table), Some(NodeType::Table));
    assert_eq!(InspectNode::node_type(&table.options), None);
    assert_eq!(
        <stencila_schema::TableOptions as InspectType>::VALUE_KIND,
        ValueKind::Flattened
    );

    Ok(())
}

/// Schema enums are reported as scalars carrying their type and variant
#[test]
fn schema_enums_are_scalars() -> Result<()> {
    let mut heading = Heading::new(1, Vec::new());
    heading.label_type = Some(stencila_schema::LabelType::AppendixLabel);

    let InspectValue::One(label_type) = value(&heading, NodeProperty::LabelType)? else {
        bail!("Expected a singular value")
    };
    assert_eq!(
        label_type.scalar(),
        Some(ScalarRef::Enum {
            schema_type: "LabelType",
            variant: "AppendixLabel"
        })
    );

    Ok(())
}

/// Values are borrowed rather than cloned
#[test]
fn values_are_borrowed() -> Result<()> {
    let text = Text::from("Hello");

    let InspectValue::One(cord) = value(&text, NodeProperty::Value)? else {
        bail!("Expected a singular value")
    };
    let Some(ScalarRef::Cord(cord)) = cord.scalar() else {
        bail!("Expected a cord")
    };
    assert!(std::ptr::eq(cord, &text.value));

    Ok(())
}

/// A `Cord` is reported whole: applying any string-only policy is a consumer's job
#[test]
fn cord_is_reported_whole() -> Result<()> {
    let mut text = Text::from("Hello");
    text.value = Cord::from("Hello");

    let InspectValue::One(cord) = value(&text, NodeProperty::Value)? else {
        bail!("Expected a singular value")
    };
    let Some(ScalarRef::Cord(cord)) = cord.scalar() else {
        bail!("Expected a cord")
    };
    assert_eq!(cord.string, "Hello");

    Ok(())
}
