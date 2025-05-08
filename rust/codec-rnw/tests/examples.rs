use std::path::PathBuf;

use codec::{
    Codec, EncodeOptions,
    common::{eyre::Result, glob::glob, tokio},
};

use codec_rnw::RnwCodec;
use common_dev::insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot};

/// Decode each example of a Noweb document and create JSON and Rnw snapshots
/// including snapshots for losses
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/*.rnw";

    for path in glob(&pattern)?.flatten() {
        let name = path
            .file_stem()
            .expect("should have file stem")
            .to_string_lossy();

        let (article, info) = RnwCodec.from_path(&path, None).await?;

        assert_json_snapshot!(format!("{name}.json"), article);
        assert_yaml_snapshot!(format!("{name}.decode.losses"), info.losses);

        let (noweb, info) = RnwCodec
            .to_string(
                &article,
                Some(EncodeOptions {
                    tool_args: vec!["--builtin".into()],
                    ..Default::default()
                }),
            )
            .await?;

        assert_snapshot!(format!("{name}.rnw"), &noweb);
        assert_yaml_snapshot!(format!("{name}.encode.losses"), info.losses);
    }

    Ok(())
}
