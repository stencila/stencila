use codecs::{DecodeOptions, Format};
use common::{eyre::Result, tokio};
use common_dev::insta::assert_debug_snapshot;

use node_merge::{DiffResult, MergeNode};

/// Snapshot tests of the `MergeNode::diff` method
#[tokio::test]
async fn diff() -> Result<()> {
    async fn diff(old: &str, new: &str) -> Result<DiffResult> {
        let options = Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        });
        let old = codecs::from_str(old, options.clone()).await?;
        let new = codecs::from_str(new, options).await?;

        Ok(old.diff(new))
    }

    macro_rules! diff {
        ($old:literal, $new:literal) => {
            diff($old, $new).await?
        };
    }

    assert_debug_snapshot!("same", diff!(r#"One"#, r#"One"#));

    assert_debug_snapshot!(
        "change-one-prop",
        diff!(
            r#"
```js
Code
```
"#,
            r#"
```py
Code
```
"#
        )
    );

    assert_debug_snapshot!(
        "change-all-props",
        diff!(
            r#"
```js
Code
```
"#,
            r#"
```py
Diff code
```
"#
        )
    );

    assert_debug_snapshot!(
        "change-node-type",
        diff!(
            r#"
```js
Code
```
"#,
            r#"
```py exec
Code
```
"#
        )
    );

    assert_debug_snapshot!(
        "missing-backticks",
        diff!(
            r#"
```js
Code
```
"#,
            r#"
```js
Code

more
"#
        )
    );


    Ok(())
}
