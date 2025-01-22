use common_dev::pretty_assertions::assert_eq;
use markdown::{
    mdast::{Code, Node},
    to_mdast, ParseOptions,
};

/// Test that the `markdown` crate handles nested
/// backtick fenced code blocks.
///
/// Note that a descending number of backticks is needed.
/// See https://github.com/wooorm/markdown-rs/issues/119
#[test]
#[allow(clippy::unwrap_used)]
fn nested_code_block() {
    let options = ParseOptions::gfm();

    let mdast = to_mdast(
        r#"
````md

First code block

```python
# Nested code block
```

````
"#,
        &options,
    )
    .unwrap();

    match mdast.children().unwrap().first().unwrap() {
        Node::Code(Code { lang, value, .. }) => {
            assert_eq!(lang.as_deref(), Some("md"));
            assert_eq!(
                value,
                "
First code block

```python
# Nested code block
```
"
            );
        }
        _ => panic!("unexpected node type"),
    }
}
