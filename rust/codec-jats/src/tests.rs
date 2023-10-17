use codec::{
    common::tokio,
    schema::shortcuts::{article, audio, img, p, video},
};
use common_dev::pretty_assertions::assert_eq;

use super::*;

/// Roundtrip test for media objects
#[tokio::test]
async fn media_objects() -> Result<()> {
    let codec = JatsCodec {};

    let doc1 = article([p([
        audio("http://example.org/audio.mp3"),
        img("http://example.org/image.png"),
        video("http://example.org/video.mp4"),
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
