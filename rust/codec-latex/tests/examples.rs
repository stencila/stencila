use std::path::PathBuf;

use codec::{
    common::{eyre::Result, glob::glob, tokio},
    Codec, DecodeOptions, EncodeOptions,
};

use codec_latex::LatexCodec;
use common_dev::insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot};

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

        let (article, info) = LatexCodec
            .from_path(
                &path,
                Some(DecodeOptions {
                    coarse: Some(true),
                    ..Default::default()
                }),
            )
            .await?;

        assert_json_snapshot!(format!("{name}.json"), article);
        assert_yaml_snapshot!(format!("{name}.decode.losses"), info.losses);

        let (latex, info) = LatexCodec
            .to_string(
                &article,
                Some(EncodeOptions {
                    passthrough_args: vec!["--builtin".into()],
                    ..Default::default()
                }),
            )
            .await?;

        assert_snapshot!(format!("{name}.tex"), &latex);
        assert_yaml_snapshot!(format!("{name}.encode.losses"), info.losses);
    }

    Ok(())
}
