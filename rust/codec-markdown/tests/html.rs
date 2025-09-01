use codec::{
    eyre::{Result, bail},
    schema::{Article, Block, Inline, Node, Paragraph},
};
use insta::assert_yaml_snapshot;

/// Inline HTML is transformed to Stencila nodes
#[test]
fn inlines() -> Result<()> {
    fn decode(md: &str) -> Result<Vec<Inline>> {
        let (Node::Article(Article { content, .. }), ..) = codec_markdown::decode(md, None)? else {
            bail!("Expected an Article")
        };

        let Some(Block::Paragraph(Paragraph { content, .. })) = content.first() else {
            bail!("Expected a Paragraph")
        };

        Ok(content.clone())
    }

    assert_yaml_snapshot!(
        decode("Some <sup>super</sup> and <sub>sub</sub>.")?,
        @r#"
    - type: Text
      value:
        string: "Some "
    - type: Superscript
      content:
        - type: Text
          value:
            string: super
    - type: Text
      value:
        string: " and "
    - type: Subscript
      content:
        - type: Text
          value:
            string: sub
    - type: Text
      value:
        string: "."
    "#);

    assert_yaml_snapshot!(
        decode(r#"A <a href="http://example.org" title="title">link</a>"#)?,
        @r#"
    - type: Text
      value:
        string: "A "
    - type: Link
      content:
        - type: Text
          value:
            string: link
      target: "http://example.org"
      title: title
    "#);

    // Autolinks can not be enabled anymore because they interfere with parsing of <a> and <img>
    assert_yaml_snapshot!(
        decode(r#"Not a http://example.org link"#)?,
        @r#"
    - type: Text
      value:
        string: "Not a http://example.org link"
    "#);

    assert_yaml_snapshot!(
        decode(r#"An <img src="http://example.org/image.png">."#)?,
        @r#"
    - type: Text
      value:
        string: "An "
    - type: ImageObject
      contentUrl: "http://example.org/image.png"
    - type: Text
      value:
        string: "."
    "#);

    assert_yaml_snapshot!(
        decode("Some <em>emphasis</em>.")?,
        @r#"
    - type: Text
      value:
        string: "Some "
    - type: Emphasis
      content:
        - type: Text
          value:
            string: emphasis
    - type: Text
      value:
        string: "."
    "#);

    assert_yaml_snapshot!(
        decode("Some <strong>strong</strong>.")?,
        @r#"
    - type: Text
      value:
        string: "Some "
    - type: Strong
      content:
        - type: Text
          value:
            string: strong
    - type: Text
      value:
        string: "."
    "#);

    // Because the `commonmark` parser splits this into text + emphasis + text
    // before we parse it, it is not possible to associate the starting
    // and ending <strong> tags so they just get ignored
    assert_yaml_snapshot!(
        decode("HTML around Markdown is <strong>_not_</strong> supported.")?,
        @r#"
    - type: Text
      value:
        string: "HTML around Markdown is "
    - type: Emphasis
      content:
        - type: Text
          value:
            string: not
    - type: Text
      value:
        string: " supported."
    "#);

    assert_yaml_snapshot!(
        decode("HTML in Markdown _is supported <strong>strong</strong>_.")?,
        @r#"
    - type: Text
      value:
        string: "HTML in Markdown "
    - type: Emphasis
      content:
        - type: Text
          value:
            string: "is supported "
        - type: Strong
          content:
            - type: Text
              value:
                string: strong
    - type: Text
      value:
        string: "."
    "#);

    // Other HTML tags are ignored, but not the content between them
    assert_yaml_snapshot!(
        decode(r#"<span class="red">Hello</span> <foo>world</bar>"#)?,
        @r#"
    - type: Text
      value:
        string: Hello world
    "#);

    Ok(())
}
