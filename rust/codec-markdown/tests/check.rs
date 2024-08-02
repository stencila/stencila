use codec::{
    common::{
        eyre::{Ok, Result},
        tokio,
    },
    Codec, DecodeInfo, Messages,
};
use codec_markdown::MarkdownCodec;
use common_dev::insta::assert_snapshot;

async fn messages(md: &str) -> Result<Messages> {
    let (.., DecodeInfo { messages, .. }) = MarkdownCodec {}.from_str(md, None).await?;
    Ok(messages)
}

#[tokio::test]
async fn no_messages() -> Result<()> {
    assert_snapshot!(
        messages(
            r#"

Self closing colon fences:

::: include something

::: call something()

::: new instruction <

::: edit instruction >

Block to be edited

Balanced colon fences

::: section

:::
"#
        )
        .await?,
        @r###""###
    );

    assert_snapshot!(
        messages(
            r#"
If blocks

::: if true

:::

::: if false

::: elif true

::: else

:::
"#
        )
        .await?,
        @r###""###
    );

    assert_snapshot!(
        messages(
            r#"
For blocks

::: for item in items

:::

::: for item in items

::: else

:::
"#
        )
        .await?,
        @r###""###
    );

    assert_snapshot!(
        messages(
            r#"
Balanced backtick fences

```
```

```py
Code
```

```python exec
Code
```
"#
        )
        .await?,
        @r###""###
    );

    assert_snapshot!(
        messages(
            r#"
Balanced MyST directives

:::{note}
The note
:::

```{code-cell} python
Code
```
"#
        )
        .await?,
        @r###""###
    );

    Ok(())
}

#[tokio::test]
async fn unbalanced_colons() -> Result<()> {
    assert_snapshot!(messages(r#"
:::

:::

::: section

::::

::: edit
"#).await?, @r###"
    1 Unpaired closing colon fence
    3 Unpaired closing colon fence
    7 Number of closing colons differs from opening colons on line 6 (4 != 3)
    9 Unpaired opening colon fence
    "###);

    assert_snapshot!(messages(r#"
::: if true

::: elif false
"#).await?, @r###"
    3 Unpaired separating colon fence
    "###);

    assert_snapshot!(messages(r#"
::: for item in items

::: else
"#).await?, @r###"
    3 Unpaired separating colon fence
    "###);

    Ok(())
}

#[tokio::test]
async fn unbalanced_backticks() -> Result<()> {
    assert_snapshot!(messages(r#"
```

```python

"#).await?, @r###"
    1 Unpaired opening backtick fence
    3 Unpaired opening backtick fence
    "###);

    Ok(())
}
