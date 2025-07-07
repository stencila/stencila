use std::path::PathBuf;

use codec::{
    common::{eyre::Result, glob::glob, tokio},
    Codec, DecodeOptions, EncodeOptions,
};

use codec_latex::LatexCodec;
use common_dev::insta::{assert_json_snapshot, assert_snapshot};

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
