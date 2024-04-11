use codecs::{DecodeOptions, Format};
use common::{eyre::Result, tokio};
use common_dev::insta::assert_debug_snapshot;
use schema::{CondenseContext, PatchNode};

/// Snapshot tests of the `MergeNode::condense` method
///
/// Mainly intended to check that properties that should be merged are included
/// in the condense context.
#[tokio::test]
async fn condense() -> Result<()> {
    async fn condense(md: &str) -> Result<CondenseContext> {
        let node = codecs::from_str(
            md,
            Some(DecodeOptions {
                format: Some(Format::Markdown),
                ..Default::default()
            }),
        )
        .await?;

        let mut context = CondenseContext::new();
        node.condense(&mut context);

        Ok(context)
    }

    macro_rules! condense {
        ($md:literal) => {
            condense($md).await?
        };
    }

    assert_debug_snapshot!(
        "inlines_marks",
        condense!(
            r#"
One _emphasis_, **strong**, ~~strikeout~~, ~subscript~, ^superscript^.

Two <u>underline</u>, <q>quote</q>.

Three `code`, `R code`{r}, $math$, `am`{asciimath}.
"#
        )
    );

    assert_debug_snapshot!(
        "inlines_links",
        condense!(
            r#"
One [](image1.png) and https://example.org.

Two [alt](https://example.org/image2.png "title").
"#
        )
    );

    assert_debug_snapshot!(
        "inlines_media",
        condense!(
            r#"
One ![](image1.png), ![](audio1.mp3), ![](video1.mp4).

Two ![alt](https://example.org/image2.png "title"), ![alt](audio1.mp3 "title"), ![alt](video1.mp4 "title").
"#
        )
    );

    assert_debug_snapshot!(
        "inlines_exec",
        condense!(
            r#"
One `1 + 2`{exec}, {{3 + 4}}, &[par1]{bool}.

Two `5 * 6`{py exec}, &[par2]{min=0 max=5}.
"#
        )
    );

    assert_debug_snapshot!(
        "inlines_edits",
        condense!(
            r#"
One [[insert add]], [[delete remove]], [[replace this>>that]].
"#
        )
    );

    assert_debug_snapshot!(
        "headings",
        condense!(
            r#"
# One

## Two

### Three
"#
        )
    );

    assert_debug_snapshot!(
        "lists",
        condense!(
            r#"
1. one
2. two

- fruit
  - apple
  - pear
- veges
  - carrots
"#
        )
    );

    assert_debug_snapshot!(
        "lists",
        condense!(
            r#"
1. one
2. two

- fruit
  - apple
  - pear
- veges
  - carrots
"#
        )
    );

    assert_debug_snapshot!(
        "lists_checked",
        condense!(
            r#"
- [ ] Todo
- [x] Done
"#
        )
    );

    assert_debug_snapshot!(
        "tables",
        condense!(
            r#"
| A | B |
|---|---|
|1  | 2 |
"#
        )
    );

    assert_debug_snapshot!(
        "sections",
        condense!(
            r#"
::: Introduction

One.

:::

::: Methods

Two.

:::

::: Discussion

Three.

:::
"#
        )
    );

    assert_debug_snapshot!(
        "claims",
        condense!(
            r#"
::: Lemma 1

Lemma.

:::

::: Theorem 1

Theorem.

:::
"#
        )
    );

    assert_debug_snapshot!(
        "thematic_breaks",
        condense!(
            r#"
Before.

***

After.
"#
        )
    );

    assert_debug_snapshot!(
        "code_blocks",
        condense!(
            r#"
```
# Some code
```

```r
# With lang
```
"#
        )
    );

    assert_debug_snapshot!(
        "code_chunks",
        condense!(
            r#"
```exec
# Some code
```

```r exec
# With lang
```

```r exec auto=never
# With auto exec set
```
"#
        )
    );

    assert_debug_snapshot!(
        "math_blocks",
        condense!(
            r#"
$$
no lang, TeX assumed
$$

```asciimath
with lang specified
```
"#
        )
    );

    assert_debug_snapshot!(
        "styled_blocks",
        condense!(
            r#"
::: { color: red }

Styled

:::
"#
        )
    );

    assert_debug_snapshot!(
        "if_blocks",
        condense!(
            r#"
::: if a < 1

Clause 1.

::: elif a > 2 {python}

Clause 2.

::: else

Clause 3.

:::
"#
        )
    );

    assert_debug_snapshot!(
        "for_blocks",
        condense!(
            r#"
::: for item in [1,2,3]

Content.

::: else

No items.

:::
"#
        )
    );

    assert_debug_snapshot!(
        "include_blocks",
        condense!(
            r#"
::: include file.md
"#
        )
    );

    assert_debug_snapshot!(
        "call_blocks",
        condense!(
            r#"
::: call file.md (par1=1, par2=`1+1`, par3=4)
"#
        )
    );

    Ok(())
}
