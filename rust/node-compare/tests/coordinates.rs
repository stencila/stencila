//! Source coordinates do not change semantic comparison or artifact locations.

use eyre::Result;
use stencila_node_compare::{Difference, compare};
use stencila_node_type::NodeProperty;
use stencila_schema::{
    Article, File, Node, Person, Primitive, PropertyValue, PropertyValueOrString,
    shortcuts::{p, t},
};

fn identifier(property: &str, value: &str) -> PropertyValueOrString {
    PropertyValueOrString::PropertyValue(PropertyValue {
        property_id: Some(property.into()),
        value: Primitive::String(value.into()),
        ..Default::default()
    })
}

fn person(identifiers: Option<Vec<PropertyValueOrString>>) -> Node {
    let mut person = Person {
        family_names: Some(vec!["Doe".into()]),
        ..Default::default()
    };
    person.options.identifiers = identifiers;
    Node::Person(person)
}

#[test]
fn coordinates_are_ignored_in_both_directions() -> Result<()> {
    let plain = person(None);
    let located = person(Some(vec![identifier("coords", "1,2,3,4,5;2,6,7,8,9")]));
    let relocated = person(Some(vec![identifier("coords", "3,20,30,40,50")]));
    for (left, right) in [
        (&plain, &located),
        (&located, &plain),
        (&located, &relocated),
    ] {
        let comparison = compare(left, right)?;
        assert!(comparison.is_equal(), "{comparison:#?}");
        assert_eq!(comparison.alignment().correspondences().len(), 1);
    }
    Ok(())
}

#[test]
fn ordinary_identifiers_retain_their_original_paths() -> Result<()> {
    let plain = person(Some(vec![identifier("ISBN", "123")]));
    let located = person(Some(vec![
        identifier("coords", "1,2,3,4,5"),
        identifier("ISBN", "123"),
    ]));
    let comparison = compare(&plain, &located)?;
    assert!(comparison.is_equal(), "{comparison:#?}");
    assert!(comparison.alignment().pairs().any(|(left, right, _)| {
        left.path.to_string() == "identifiers/0" && right.path.to_string() == "identifiers/1"
    }));

    let changed = person(Some(vec![identifier("ISBN", "456")]));
    assert!(!compare(&plain, &changed)?.is_equal());
    let missing = person(None);
    assert!(!compare(&plain, &missing)?.is_equal());
    Ok(())
}

#[test]
fn scalar_identifiers_retain_their_original_indices() -> Result<()> {
    let left = person(Some(vec![
        identifier("coords", "1,2,3,4,5"),
        PropertyValueOrString::String("old identifier".into()),
    ]));
    let right = person(Some(vec![PropertyValueOrString::String(
        "new identifier".into(),
    )]));
    let forward = compare(&left, &right)?;
    let reverse = compare(&right, &left)?;
    assert_eq!(forward.clone().invert(), reverse);
    assert!(forward.differences().iter().any(|difference| matches!(
        difference,
        Difference::ValueChanged { location, .. }
            if location.property == Some(NodeProperty::Identifiers)
                && location.left_index == Some(1) && location.right_index == Some(0)
    )));
    Ok(())
}

#[test]
fn coordinates_only_and_empty_identifiers_are_equal() -> Result<()> {
    let empty = person(Some(Vec::new()));
    let located = person(Some(vec![identifier("coords", "1,2,3,4,5")]));
    for (left, right) in [(&empty, &located), (&located, &empty)] {
        let comparison = compare(left, right)?;
        assert!(comparison.is_equal(), "{comparison:#?}");
    }
    Ok(())
}

#[test]
fn singular_and_repeated_values_of_a_property_compare() -> Result<()> {
    let file = Node::File(File {
        content: Some("Some text".into()),
        ..Default::default()
    });
    let article = Node::Article(Article::new(vec![p([t("Some text")])]));
    for (left, right) in [(&file, &article), (&article, &file)] {
        assert!(!compare(left, right)?.is_equal());
    }
    Ok(())
}

#[test]
fn coordinate_values_outside_identifiers_are_compared() -> Result<()> {
    let value = |coords: &str| {
        Node::PropertyValue(PropertyValue {
            property_id: Some("coords".into()),
            value: Primitive::String(coords.into()),
            ..Default::default()
        })
    };
    assert!(!compare(&value("1,2,3,4,5"), &value("2,3,4,5,6"))?.is_equal());
    Ok(())
}
