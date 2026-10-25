use stencila_codec::{
    DecodeInfo, EncodeOptions,
    eyre::{OptionExt, Result, bail},
    stencila_schema::{Article, Block, Node},
};
use stencila_codec_jats::{decode, encode};
use stencila_codec_text_trait::to_text;

fn article(source: &str) -> Result<(Article, DecodeInfo)> {
    let (node, info) = decode(source, None)?;
    let Node::Article(article) = node else {
        bail!("expected an article")
    };
    Ok((article, info))
}

fn assert_roundtrip(article: &Article) -> Result<()> {
    let expected = Node::Article(article.clone());
    for compact in [true, false] {
        let (encoded, _) = encode(
            &expected,
            Some(EncodeOptions {
                compact: Some(compact),
                ..Default::default()
            }),
        )?;
        let (actual, _) = decode(&encoded, None)?;
        assert_eq!(actual, expected, "compact={compact}: {encoded}");
    }
    Ok(())
}

#[test]
fn retains_distinct_appendices() -> Result<()> {
    let (article, info) = article(
        r#"<article><back><app-group>
          <app id="a"><p>Appendix A</p></app>
          <app id="b"><p>Appendix B</p></app>
        </app-group></back></article>"#,
    )?;
    assert!(info.losses.is_empty());
    assert_eq!(article.content.len(), 2);
    for (block, id, text) in [
        (&article.content[0], "a", "Appendix A"),
        (&article.content[1], "b", "Appendix B"),
    ] {
        let Block::Section(section) = block else {
            bail!("expected an appendix")
        };
        assert_eq!(section.id.as_deref(), Some(id));
        assert_eq!(to_text(&section.content).trim(), text);
    }
    assert_roundtrip(&article)
}

#[test]
fn deduplicates_only_identical_back_matter_sections() -> Result<()> {
    let (article, _) = article(
        r#"<article>
          <body><sec sec-type="acknowledgements"><p>Thanks</p></sec></body>
          <back>
            <ack><p>Thanks</p></ack>
            <ack id="additional"><p>Thanks</p></ack>
            <ack><p>Different thanks</p></ack>
          </back>
        </article>"#,
    )?;
    assert_eq!(article.content.len(), 3);
    assert!(matches!(
        &article.content[1],
        Block::Section(section) if section.id.as_deref() == Some("additional")
    ));
    assert_eq!(to_text(&article.content[2]).trim(), "Different thanks");
    assert_roundtrip(&article)
}

#[test]
fn accumulates_reference_lists_and_ignores_empty_lists() -> Result<()> {
    let (article, info) = article(
        r#"<article><back>
          <ref-list><ref id="r1"><element-citation><article-title>First study</article-title></element-citation></ref></ref-list>
          <ref-list><ref id="r2"><element-citation><article-title>Second study</article-title></element-citation></ref></ref-list>
          <ref-list/>
        </back></article>"#,
    )?;
    assert!(info.losses.is_empty());
    let references = article
        .references
        .as_ref()
        .ok_or_eyre("missing references")?;
    assert_eq!(
        references
            .iter()
            .map(|reference| reference.id.as_deref())
            .collect::<Vec<_>>(),
        [Some("r1"), Some("r2")]
    );
    assert_roundtrip(&article)
}
