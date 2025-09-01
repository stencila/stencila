//! Tests of serialization/deserialization for schema types
//!
//! See also tests in the `codec-json` crate.

use std::str::FromStr;

use eyre::{Result, bail};
use indexmap::IndexMap;
use itertools::Itertools;

use schema::{Array, Article, Date, IntegerOrString, Node, Object, Primitive};

/// Test that unrecognized keys are collected into the `Article.extra`
/// primitive [`Object`].
#[test]
fn article_extra() -> Result<()> {
    let yaml = r"
type: Article

# Schema properties with aliases
date: 2025-01-01
pageStart: 1
page-end: 2

# Other properties collected into extra
extra-string: Hello world
extraint: 42
extra_array: [1.23, 456, ABC]
extraObject:
    a:
        a1: 123
    b: [1, 2, 3]

content: []
";

    let article: Article = serde_yaml::from_str(yaml)?;

    assert_eq!(article.date_published, Date::from_str("2025-01-01").ok());
    assert_eq!(
        article.options.page_start,
        Some(IntegerOrString::Integer(1))
    );
    assert_eq!(article.options.page_end, Some(IntegerOrString::Integer(2)));

    let Some(extra) = article.options.extra else {
        bail!("expected an extra object")
    };
    assert_eq!(
        extra.keys().join(","),
        "extra-string,extraint,extra_array,extraObject"
    );
    assert_eq!(
        extra.get("extra-string"),
        Some(&Primitive::String("Hello world".into()))
    );
    assert_eq!(extra.get("extraint"), Some(&Primitive::Integer(42)));
    assert_eq!(
        extra.get("extra_array"),
        Some(&Primitive::Array(Array(vec![
            Primitive::Number(1.23),
            Primitive::Integer(456),
            Primitive::String("ABC".into())
        ])))
    );

    // Using pattern matching is the most ergonomic way to access deeply nested objects
    let Some(Primitive::Object(object)) = extra.get("extraObject") else {
        bail!("expected an object")
    };
    let Some(Primitive::Array(array)) = object.get("b") else {
        bail!("expected an array")
    };
    let Some(Primitive::Integer(integer)) = array.get(1) else {
        bail!("expected an integer")
    };
    assert_eq!(*integer, 2);

    Ok(())
}

/// Test deserialization of potentially ambiguous JSON string to [`Node`] enum
#[test]
fn ambiguous_value() -> Result<()> {
    // Regression tests for issue https://github.com/stencila/stencila/issues/2616
    // which was due to arrays of string previously being deserialized as a Cord
    assert_eq!(
        serde_json::from_str::<Node>(r#""one""#)?,
        Node::String("one".into())
    );
    assert_eq!(
        serde_json::from_str::<Node>(r#"["one"]"#)?,
        Node::Array(Array(vec![Primitive::String("one".into())]))
    );
    assert_eq!(
        serde_json::from_str::<Node>(r#"["one", "two"]"#)?,
        Node::Array(Array(vec![
            Primitive::String("one".into()),
            Primitive::String("two".into())
        ]))
    );

    // Should be deserialized to Object because does not have a "type" key
    assert_eq!(
        serde_json::from_str::<Node>(r#"{"key": 123}"#)?,
        Node::Object(Object(IndexMap::from([(
            "key".into(),
            Primitive::Integer(123),
        )])))
    );

    // Should be deserialized to Object, not Cord, because has "number" key (in addition to "string")
    assert_eq!(
        serde_json::from_str::<Node>(r#"{"number": 1.23, "string": "one"}"#)?,
        Node::Object(Object(IndexMap::from([
            ("number".into(), Primitive::Number(1.23),),
            ("string".into(), Primitive::String("one".into()),)
        ])))
    );

    Ok(())
}
