use indexmap::IndexMap;

use common::eyre::Result;
use common_dev::pretty_assertions::assert_eq;

use schema::{
    store::Store,
    traits::Node,
    types::{Null, Object, Primitive, Text},
};

/// Test loading & dumping of `Primitive` nodes
#[test]
fn primitives() -> Result<()> {
    type Root = Object;

    // Create base store with various primitives
    let mut base = Store::new();
    let root = Root::from([
        ("null".to_string(), Primitive::Null(Null {})),
        ("bool".to_string(), Primitive::Boolean(true)),
        ("int".to_string(), Primitive::Integer(123)),
        ("uint".to_string(), Primitive::UnsignedInteger(123)),
        ("num".to_string(), Primitive::Number(1.23)),
        ("str".to_string(), Primitive::String("abc".to_string())),
        (
            "array".to_string(),
            Primitive::Array(vec![Primitive::Boolean(true), Primitive::Integer(456)]),
        ),
        (
            "obj".to_string(),
            Primitive::Object(IndexMap::from([("a".to_string(), Primitive::Integer(1))])),
        ),
    ]);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?, root);

    // Fork it
    let mut fork = base.fork();
    let mut root = Root::load(&fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Make modifications, each time merging changes back into
    // store and checking both stores for consistency

    // Change values
    root.insert("null".to_string(), Primitive::Null(Null {}));
    root.insert("bool".to_string(), Primitive::Boolean(false));
    root.insert("int".to_string(), Primitive::Integer(456));
    root.insert("int".to_string(), Primitive::UnsignedInteger(456));
    root.insert("num".to_string(), Primitive::Number(4.56));
    root.insert("str".to_string(), Primitive::String("def".to_string()));
    root.insert(
        "obj".to_string(),
        Primitive::Object(IndexMap::from([("b".to_string(), Primitive::Number(1.23))])),
    );
    root.dump(&mut fork)?;
    assert_eq!(Root::load(&fork)?, root);
    base.merge(&mut fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Change types
    root.insert("null".to_string(), Primitive::String("null".to_string()));
    root.insert("bool".to_string(), Primitive::Number(1.23));
    root.insert("int".to_string(), Primitive::String("abc".to_string()));
    root.insert("num".to_string(), Primitive::Integer(123));
    root.insert("str".to_string(), Primitive::Null(Null {}));
    root.dump(&mut fork)?;
    assert_eq!(Root::load(&fork)?, root);
    base.merge(&mut fork)?;
    assert_eq!(Root::load(&base)?, root);

    Ok(())
}

/// Test loading & dumping of `Text` nodes
#[test]
fn text() -> Result<()> {
    type Root = IndexMap<String, Text>;

    // Create base store with a few text nodes
    let mut base = Store::new();
    let root = Root::from([
        ("insert".to_string(), Text::from("abcd")),
        ("delete".to_string(), Text::from("abcd")),
        ("replace".to_string(), Text::from("abcd")),
        ("varied".to_string(), Text::from("abcd")),
    ]);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?.strip_ids(), &root);

    // Fork it
    let mut fork = base.fork();
    let mut root = Root::load(&fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Make modifications, merge changes back into
    // store and check both stores for consistency

    root.get_mut("insert").unwrap().value = "a_bcd".to_string();
    root.get_mut("delete").unwrap().value = "acd".to_string();
    root.get_mut("replace").unwrap().value = "a_cd".to_string();
    root.get_mut("varied").unwrap().value = "_ace".to_string();

    root.dump(&mut fork)?;
    assert_eq!(Root::load(&fork)?, root);

    base.merge(&mut fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Make concurrent changes to and checked merged values are as expected

    root.get_mut("varied").unwrap().value = "Space".to_string();
    let mut fork1 = base.fork();
    root.dump(&mut fork1)?;

    root.get_mut("varied").unwrap().value = "ace invaders".to_string();
    let mut fork2 = base.fork();
    root.dump(&mut fork2)?;

    base.merge(&mut fork1)?;
    base.merge(&mut fork2)?;

    let actual = &Root::load(&base)?["varied"].value;
    assert_eq!(actual, "Space invaders");

    Ok(())
}

/// Test loading & dumping of `Vec`s
#[test]
fn vec() -> Result<()> {
    type Root = IndexMap<String, Vec<Text>>;

    // Create base store
    let mut base = Store::new();
    let mut root = Root::from([(
        "vec".to_string(),
        vec![Text::from("one"), Text::from("two")],
    )]);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?.strip_ids(), &root);

    // Make modifications, merge changes back into
    // store and check store for consistency

    // Add an item
    root.get_mut("vec").unwrap().push(Text::from("three"));
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?.strip_ids(), &root);

    // Remove an item
    root.get_mut("vec").unwrap().remove(1);
    root.dump(&mut base)?;
    //assert_eq!(Root::load(&base)?.strip_ids(), &root);

    Ok(())
}

/// Test loading & dumping of `IndexMap`s
#[test]
fn indexmap() -> Result<()> {
    type Root = IndexMap<String, String>;

    // Create base store with map of strings
    let mut base = Store::new();
    let root = Root::from([
        ("a".to_string(), "one".to_string()),
        ("b".to_string(), "two".to_string()),
        ("c".to_string(), "three".to_string()),
    ]);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?, root);

    // Fork it
    let mut fork = base.fork();
    let mut root = Root::load(&fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Make modifications, each time merging changes back into
    // store and checking both stores for consistency

    // Change an item
    root.insert("a".to_string(), "first".to_string());
    root.dump(&mut fork)?;
    assert_eq!(Root::load(&fork)?, root);
    base.merge(&mut fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Insert an item
    root.insert("d".to_string(), "four".to_string());
    root.dump(&mut fork)?;
    assert_eq!(Root::load(&fork)?, root);
    base.merge(&mut fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Remove an item
    root.remove("b");
    root.dump(&mut fork)?;
    assert_eq!(Root::load(&fork)?, root);
    base.merge(&mut fork)?;
    assert_eq!(Root::load(&base)?, root);

    Ok(())
}
