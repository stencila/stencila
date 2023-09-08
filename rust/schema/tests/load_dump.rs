use std::collections::HashMap;

use common::{eyre::Result, serde_json::json};
use common_dev::pretty_assertions::assert_eq;

use schema::{Array, Article, Block, Cord, Inline, Node, Null, Object, Paragraph, Primitive, Text};

use node_store::{ReadNode, WriteNode, WriteStore};
use node_strip::{StripNode, Targets};

fn txt(value: &str) -> Text {
    Text::new(Cord(value.to_string()))
}

/// Test loading & dumping of `Primitive` nodes
#[test]
fn primitives() -> Result<()> {
    type Root = HashMap<String, Primitive>;

    // Create base store with various primitives
    let mut base = WriteStore::new();
    let root = Root::from([
        ("null".to_string(), Primitive::Null(Null {})),
        ("bool".to_string(), Primitive::Boolean(true)),
        ("int".to_string(), Primitive::Integer(123)),
        ("uint".to_string(), Primitive::UnsignedInteger(123)),
        ("num".to_string(), Primitive::Number(1.23)),
        ("str".to_string(), Primitive::String("abc".to_string())),
        (
            "array".to_string(),
            Primitive::Array(Array::from([
                Primitive::Boolean(true),
                Primitive::Integer(456),
            ])),
        ),
        (
            "obj".to_string(),
            Primitive::Object(Object::from([("a".to_string(), Primitive::Integer(1))])),
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
        Primitive::Object(Object::from([("b".to_string(), Primitive::Number(1.23))])),
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

/// Test loading & dumping of `Option`s
#[test]
fn option() -> Result<()> {
    type Root = HashMap<String, Option<i64>>;

    // Create base store
    let mut base = WriteStore::new();
    let mut root = Root::from([("some".to_string(), Some(42)), ("none".to_string(), None)]);
    root.dump(&mut base)?;
    assert_eq!(
        Root::load(&base)?,
        // Note: key with None is not stored
        Root::from([("some".to_string(), Some(42)),])
    );

    // Change the some value
    root.insert("some".to_string(), Some(21));
    root.dump(&mut base)?;
    assert_eq!(
        Root::load(&base)?,
        Root::from([("some".to_string(), Some(21)),])
    );

    // Change some to None
    root.insert("some".to_string(), None);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?, Root::default());

    Ok(())
}

/// Test loading & dumping of `Text` nodes
#[test]
fn text() -> Result<()> {
    type Root = HashMap<String, Text>;

    // Create base store with a few text nodes
    let mut base = WriteStore::new();
    let root = Root::from([
        ("insert".to_string(), txt("abcd")),
        ("delete".to_string(), txt("abcd")),
        ("replace".to_string(), txt("abcd")),
        ("varied".to_string(), txt("abcd")),
    ]);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?.strip(&Targets::id()), &root);

    // Fork it
    let mut fork = base.fork();
    let mut root = Root::load(&fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Make modifications, merge changes back into
    // store and check both stores for consistency

    root.get_mut("insert").unwrap().value.0 = "a_bcd".to_string();
    root.get_mut("delete").unwrap().value.0 = "acd".to_string();
    root.get_mut("replace").unwrap().value.0 = "a_cd".to_string();
    root.get_mut("varied").unwrap().value.0 = "_ace".to_string();

    root.dump(&mut fork)?;
    assert_eq!(Root::load(&fork)?, root);

    base.merge(&mut fork)?;
    assert_eq!(Root::load(&base)?, root);

    // Make concurrent changes to and checked merged values are as expected

    root.get_mut("varied").unwrap().value.0 = "Space".to_string();
    let mut fork1 = base.fork();
    root.dump(&mut fork1)?;

    root.get_mut("varied").unwrap().value.0 = "ace invaders".to_string();
    let mut fork2 = base.fork();
    root.dump(&mut fork2)?;

    base.merge(&mut fork1)?;
    base.merge(&mut fork2)?;

    let actual = &Root::load(&base)?["varied"].value;
    assert_eq!(actual.0, "Space invaders");

    Ok(())
}

/// Test loading & dumping of `Vec`s
#[test]
fn vec() -> Result<()> {
    type Root = HashMap<String, Vec<Text>>;

    // Create base store
    let mut base = WriteStore::new();
    let mut root = Root::from([("vec".to_string(), vec![txt("one"), txt("two")])]);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?.strip(&Targets::id()), &root);

    // Make modifications, merge changes back into
    // store and check store for consistency

    // Add an item
    root.get_mut("vec").unwrap().push(txt("three"));
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?.strip(&Targets::id()), &root);

    // Remove an item
    root.get_mut("vec").unwrap().remove(1);
    root.dump(&mut base)?;
    assert_eq!(Root::load(&base)?.strip(&Targets::id()), &root);

    Ok(())
}

/// Test loading & dumping of `HashMap`s
#[test]
fn hash_map() -> Result<()> {
    type Root = HashMap<String, String>;

    // Create base store with map of strings
    let mut base = WriteStore::new();
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

/// Test loading & dumping of `Article`s
#[test]
fn article() -> Result<()> {
    let mut base = WriteStore::new();

    // Default, empty article
    let mut article1 = Article::default();
    article1.dump(&mut base)?;
    assert_eq!(Article::load(&base)?.strip(&Targets::id()), &article1);

    // Add an optional property
    article1.options.alternate_names = Some(vec!["some name".to_string()]);
    article1.dump(&mut base)?;
    assert_eq!(Article::load(&base)?.strip(&Targets::id()), &article1);

    // Add some content
    article1.content.push(Block::Paragraph(Paragraph {
        content: vec![Inline::Text(txt("Hello world"))],
        ..Default::default()
    }));
    article1.dump(&mut base)?;
    assert_eq!(Article::load(&base)?.strip(&Targets::id()), &article1);

    Ok(())
}

/// Test loading & dumping of `Node`s
#[test]
fn node() -> Result<()> {
    use common::serde_json::{self, json};

    let mut base = WriteStore::new();

    // Default, empty article
    let node1: Node = serde_json::from_value(json!({
        "type": "Article",
        "content": []
    }))?;
    node1.dump(&mut base)?;
    assert_eq!(Node::load(&base)?.strip(&Targets::id()), &node1);

    Ok(())
}
