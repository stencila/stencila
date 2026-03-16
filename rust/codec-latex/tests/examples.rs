use std::path::PathBuf;

use glob::glob;

use stencila_codec::{Codec, DecodeOptions, EncodeOptions, eyre::Result};

use insta::{assert_json_snapshot, assert_snapshot};
use stencila_codec_latex::LatexCodec;

/// Decode each example of a LaTeX document and create JSON and LaTeX snapshots
/// including snapshots for losses
///
/// Currently testing decoding with `--coarse` options and built-in, rather than
/// Pandoc-based encoding.
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/*.tex";

    let have_pandoc = which::which("pandoc").is_ok();

    for path in glob(&pattern)?.flatten() {
        let name = path
            .file_stem()
            .expect("should have file stem")
            .to_string_lossy();

        // Skip island-wrap-* fixtures — they require specific `island_wrap`
        // decode options and are tested by dedicated tests below.
        if name.starts_with("island-wrap-") {
            continue;
        }

        // Using default `--coarse` decoding
        let (article, ..) = LatexCodec.from_path(&path, None).await?;
        assert_json_snapshot!(format!("{name}.coarse.json"), article, {".commit" => "redacted"});

        // Using default encoding of coarse article
        let (latex, info) = LatexCodec.to_string(&article, None).await?;
        assert_snapshot!(format!("{name}.coarse.tex"), &latex);
        assert_snapshot!(format!("{name}.coarse.encode.map"), info.mapping);

        // Only run the following tests if Pandoc is installed
        if !have_pandoc {
            continue;
        }

        // Using `--fine` decoding
        let (article, ..) = LatexCodec
            .from_path(
                &path,
                Some(DecodeOptions {
                    coarse: Some(false),
                    ..Default::default()
                }),
            )
            .await?;
        assert_json_snapshot!(format!("{name}.fine.json"), article, {".commit" => "redacted"});

        // Using built-in encoding of fine article
        let (latex, ..) = LatexCodec
            .to_string(&article, Some(EncodeOptions::default()))
            .await?;
        assert_snapshot!(format!("{name}.fine.tex"), &latex);
    }

    Ok(())
}

/// Test that island wrapping preserves explicit manual islands
#[tokio::test]
async fn island_wrap_respects_manual_islands() -> Result<()> {
    let examples = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/examples");

    let wrap_opts = Some(DecodeOptions {
        island_wrap: vec!["figure".to_string()],
        ..Default::default()
    });

    // Manual island only: figures inside a manual island are not re-wrapped
    let path = examples.join("island-wrap-manual.tex").canonicalize()?;
    let (article, ..) = LatexCodec.from_path(&path, wrap_opts.clone()).await?;
    assert_json_snapshot!(
        "island_wrap_manual.coarse.json",
        article,
        {".commit" => "redacted"}
    );

    // Mixed: manual island is preserved AND a bare figure outside is auto-wrapped
    let path = examples.join("island-wrap-mixed.tex").canonicalize()?;
    let (article, ..) = LatexCodec.from_path(&path, wrap_opts.clone()).await?;
    assert_json_snapshot!(
        "island_wrap_mixed.coarse.json",
        article,
        {".commit" => "redacted"}
    );

    // ContinuedFloat: consecutive figures with \ContinuedFloat produce
    // separate islands but the continuation island has is_continuation=true
    let path = examples
        .join("island-wrap-continued-float.tex")
        .canonicalize()?;
    let (article, ..) = LatexCodec.from_path(&path, wrap_opts).await?;
    assert_json_snapshot!(
        "island_wrap_continued_float.coarse.json",
        article,
        {".commit" => "redacted"}
    );

    Ok(())
}
