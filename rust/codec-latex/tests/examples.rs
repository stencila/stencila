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
/// Currently testing decoding with `--coarse` options and bultin, rather than
/// Pandoc-based encoding.
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/*.tex";

    for path in glob(&pattern)?.flatten() {
        let name = path
            .file_stem()
            .expect("should have file stem")
            .to_string_lossy();

        // Using default `--coarse` decoding
        let (article, ..) = LatexCodec.from_path(&path, None).await?;
        assert_json_snapshot!(format!("{name}.coarse.json"), article);

        // Using default encoding of coarse article
        let (latex, info) = LatexCodec.to_string(&article, None).await?;
        assert_snapshot!(format!("{name}.coarse.tex"), &latex);
        assert_snapshot!(format!("{name}.coarse.encode.map"), info.mapping);

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
        assert_json_snapshot!(format!("{name}.fine.json"), article);

        // Using Pandoc encoding of fine article
        let (latex, ..) = LatexCodec
            .to_string(
                &article,
                Some(EncodeOptions {
                    tool: Some("pandoc".into()),
                    ..Default::default()
                }),
            )
            .await?;
        assert_snapshot!(format!("{name}.fine.tex"), &latex);
    }

    Ok(())
}
