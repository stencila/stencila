use pretty_assertions::assert_eq;
use stencila_codec::stencila_schema::{
    Author, Node,
    shortcuts::{art, aud, img, p, sti, vid},
};

use super::*;

/// Roundtrip test for media objects
#[tokio::test]
async fn media_objects() -> Result<()> {
    let codec = JatsCodec {};

    let doc1 = art([p([
        aud("http://example.org/audio.mp3"),
        img("http://example.org/image.png"),
        vid("http://example.org/video.mp4"),
    ])]);

    let (jats, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        jats,
        r#"<article dtd-version="1.3" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:mml="http://www.w3.org/1998/Math/MathML"><body><p><inline-media xlink:href="http://example.org/audio.mp3" mimetype="audio"></inline-media><inline-graphic xlink:href="http://example.org/image.png"></inline-graphic><inline-media xlink:href="http://example.org/video.mp4" mimetype="video"></inline-media></p></body></article>"#
    );

    let (doc2, ..) = codec.from_str(&jats, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}

/// Roundtrip test for `Span`
///
/// This is a regression test for a bug found in roundtrip proptests.
#[tokio::test]
async fn spans() -> Result<()> {
    let codec = JatsCodec {};

    // Code contains whitespace characters that need to be escaped
    let doc1 = art([p([sti("\t\n\r", [])])]);

    let (jats, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        jats,
        r#"<article dtd-version="1.3" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:mml="http://www.w3.org/1998/Math/MathML"><body><p><styled-content style="&#9;&#10;&#13;"></styled-content></p></body></article>"#
    );

    let (doc2, ..) = codec.from_str(&jats, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}

/// Correspondence notes referenced from a contributor should be retained on the person.
#[tokio::test]
async fn contributor_correspondence_email() -> Result<()> {
    let jats = r#"
        <article>
          <front>
            <article-meta>
              <contrib-group>
                <contrib contrib-type="author" corresp="yes">
                  <name><surname>Doe</surname><given-names>Jane</given-names></name>
                  <xref ref-type="corresp" rid="cor1">*</xref>
                </contrib>
              </contrib-group>
              <author-notes>
                <corresp id="cor1">Correspondence: <email>jane@example.org</email></corresp>
              </author-notes>
            </article-meta>
          </front>
        </article>
    "#;

    let (node, ..) = JatsCodec.from_str(jats, None).await?;
    let Node::Article(article) = node else {
        return Err(stencila_codec::eyre::eyre!("expected an article"));
    };
    let Some(Author::Person(person)) = article
        .authors
        .and_then(|authors| authors.into_iter().next())
    else {
        return Err(stencila_codec::eyre::eyre!("expected a person author"));
    };

    assert_eq!(person.options.emails, Some(vec!["jane@example.org".into()]));

    Ok(())
}
