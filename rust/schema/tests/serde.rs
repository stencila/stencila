//! Tests of serialization/deserialization for schema types
//!
//! See also tests in the `codec-json` crate.

use std::str::FromStr;

use common::{
    eyre::{bail, Result},
    itertools::Itertools,
    serde_yaml,
};
use schema::{Array, Article, Date, IntegerOrString, Primitive};

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
    assert_eq!(article.options.page_start, Some(IntegerOrString::Integer(1)));
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
