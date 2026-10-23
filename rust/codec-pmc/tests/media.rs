use flate2::{Compression, write::GzEncoder};
use std::io::Cursor;
use stencila_codec::{
    Codec, DecodeOptions,
    eyre::{OptionExt, Result},
    stencila_schema::{Block, Node},
};
use stencila_codec_pmc::PmcCodec;

#[tokio::test]
async fn original_references_outlive_pmc_archive_extraction() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let archive = dir.path().join("article.tar.gz");
    let mut tar = tar::Builder::new(GzEncoder::new(
        std::fs::File::create(&archive)?,
        Compression::fast(),
    ));
    let xml = br#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><fig><graphic xlink:href="figure"/></fig><supplementary-material><media xlink:href="data.csv"/></supplementary-material></body></article>"#;
    let image = include_bytes!("../../codec-jats/tests/media/extensionless/figure.tif");
    for (name, bytes) in [
        ("PMC1/article.nxml", xml.as_slice()),
        ("PMC1/figure.tif", image.as_slice()),
        ("PMC1/data.csv", b"x,y\n1,2\n".as_slice()),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, name, Cursor::new(bytes))?;
    }
    tar.into_inner()?.finish()?;
    let (node, _, _) = PmcCodec
        .from_path(
            &archive,
            Some(DecodeOptions {
                embed_media: Some(false),
                embed_supplements: Some(false),
                reproducible: Some(false),
                ..Default::default()
            }),
        )
        .await?;
    let Node::Article(article) = node else {
        stencila_codec::eyre::bail!("expected article")
    };
    let Block::Figure(figure) = &article.content[0] else {
        stencila_codec::eyre::bail!("expected figure")
    };
    let Block::ImageObject(found) = &figure.content[0] else {
        stencila_codec::eyre::bail!("expected image")
    };
    assert_eq!(std::fs::read(dir.path().join(&found.content_url))?, image);
    let Block::Supplement(supplement) = &article.content[1] else {
        stencila_codec::eyre::bail!("expected supplement")
    };
    assert_eq!(
        std::fs::read(
            dir.path()
                .join(supplement.target.as_deref().ok_or_eyre("target")?)
        )?,
        b"x,y\n1,2\n"
    );
    Ok(())
}
