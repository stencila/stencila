use common_dev::pretty_assertions::assert_eq;
use markdown::{
    mdast::{Code, Node},
    to_mdast, ParseOptions,
};


/// Test that the `markdown` crate currently does NOT handle nested
/// backtick fenced code blocks. This test is here so that we get
/// know if this ever gets fixed (because this test will fail).
/// 
/// https://github.com/wooorm/markdown-rs/issues/119
#[test]
fn nested_code_block() {
    let options = ParseOptions::gfm();

    let mdast = to_mdast(
        r#"
```md

First code block

````python
# Nested code block
````

```
"#,
        &options,
    )
    .unwrap();

    match mdast.children().unwrap().first().unwrap() {
        Node::Code(Code { lang, value, .. }) => {
            assert_eq!(lang.as_deref(), Some("md"));
            // This currently passes but it shouldn't: the python code block is truncated
            assert_eq!(
                value,
                "
First code block

````python
# Nested code block"
            );
        }
        _ => assert!(false, "unexpected node type"),
    }
}
