use insta::assert_snapshot;

use stencila_codec_markdown::preprocess;

/// Check of preprocessing for ensuring empty lines between fenced divs
#[rustfmt::skip]
#[test]
fn colon_divs() {
    // Does not add unnecessary empty lines for lines that are not colon divs
    assert_snapshot!(
        preprocess(""),
        @""
    );
    assert_snapshot!(
        preprocess("Paragraph 1\n\nParagraph 2"),
        @r"
    Paragraph 1

    Paragraph 2
    "
    );
    assert_snapshot!(
        preprocess("# Heading\n\nParagraph 1"),
        @r"
    # Heading

    Paragraph 1
    "
    );

    // Does not add unnecessary empty lines for lines that are not colon divs
    assert_snapshot!(
        preprocess("::: theorem

Abc

:::"),
        @r"
    ::: theorem

    Abc

    :::
    "
    );

    // Adds empty lines where needed
    assert_snapshot!(
        preprocess("::: theorem
Abc
:::"),
        @r"
    ::: theorem

    Abc

    :::
    "
    );
    assert_snapshot!(
        preprocess("::: edit >>
Abc"),
        @r"
    ::: edit >>

    Abc
    "
    );
}
