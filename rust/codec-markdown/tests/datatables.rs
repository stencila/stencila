use insta::{assert_snapshot, assert_yaml_snapshot};
use stencila_codec::{
    Codec, DecodeOptions, EncodeOptions, StructuringOperation, eyre::Result,
    stencila_format::Format,
};
use stencila_codec_markdown::MarkdownCodec;

#[test]
fn tables_to_datatables_is_not_default_structuring() {
    let options = MarkdownCodec {}.structuring_options(&Format::Smd);

    assert!(!options.should_perform(StructuringOperation::TablesToDatatables));
}

#[tokio::test]
async fn decode_data_fence() -> Result<()> {
    let codec = MarkdownCodec {};
    let (node, ..) = codec
        .from_str(
            r#"::: data

A caption for the data

| A   | B   |
| --- | --- |
| 1   | 2   |

:::
"#,
            Some(DecodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
        )
        .await?;

    assert_yaml_snapshot!(node, @r"
    type: Article
    content:
      - type: Datatable
        caption:
          - type: Paragraph
            content:
              - type: Text
                value:
                  string: A caption for the data
        columns:
          - type: DatatableColumn
            name: A
            values:
              - 1
            validator:
              type: ArrayValidator
              itemsValidator:
                type: IntegerValidator
          - type: DatatableColumn
            name: B
            values:
              - 2
            validator:
              type: ArrayValidator
              itemsValidator:
                type: IntegerValidator
    ");

    Ok(())
}

#[tokio::test]
async fn roundtrip_data_fence() -> Result<()> {
    let codec = MarkdownCodec {};
    let source = r#"::: data

A caption for the data

| A   | B   |
| --- | --- |
| 1   | 2   |

:::
"#;

    let (node, ..) = codec
        .from_str(
            source,
            Some(DecodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
        )
        .await?;
    let (encoded, ..) = codec.to_string(&node, None).await?;

    assert_snapshot!(encoded, @r"
    ::: data

    A caption for the data

    | A   | B   |
    | --- | --- |
    | 1   | 2   |

    :::
    ");

    Ok(())
}

#[tokio::test]
async fn encode_data_fence_only_for_smd() -> Result<()> {
    let codec = MarkdownCodec {};
    let source = r#"::: data

A caption for the data

| A   | B   |
| --- | --- |
| 1   | 2   |

:::
"#;

    let (node, ..) = codec
        .from_str(
            source,
            Some(DecodeOptions {
                format: Some(Format::Smd),
                ..Default::default()
            }),
        )
        .await?;
    let (encoded, ..) = codec
        .to_string(
            &node,
            Some(EncodeOptions {
                format: Some(Format::Myst),
                ..Default::default()
            }),
        )
        .await?;

    assert_snapshot!(encoded, @r"
    A caption for the data

    | A   | B   |
    | --- | --- |
    | 1   | 2   |
    ");

    Ok(())
}
