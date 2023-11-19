use codec::{
    common::tokio,
    schema::shortcuts::{art, aud, img, p, sti, vid},
};
use common_dev::pretty_assertions::assert_eq;

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

    let (jats, _) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        jats,
        r#"<article dtd-version="1.3" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:mml="http://www.w3.org/1998/Math/MathML"><body><p><inline-media xlink:href="http://example.org/audio.mp3" mimetype="audio"></inline-media><inline-graphic xlink:href="http://example.org/image.png"></inline-graphic><inline-media xlink:href="http://example.org/video.mp4" mimetype="video"></inline-media></p></body></article>"#
    );

    let (doc2, _) = codec.from_str(&jats, None).await?;
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

    let (jats, _) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        jats,
        r#"<article dtd-version="1.3" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:mml="http://www.w3.org/1998/Math/MathML"><body><p><styled-content style="&#9;&#10;&#13;"></styled-content></p></body></article>"#
    );

    let (doc2, _) = codec.from_str(&jats, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}
