use std::path::PathBuf;

use codec::{
    common::{
        eyre::{OptionExt, Result},
        glob::glob,
        tokio,
    },
    Codec,
};
use common_dev::insta::{assert_json_snapshot, assert_yaml_snapshot};

use codec_pmcoap::PmcOapCodec;

/// Decode each example of a PMC OA Package and create JSON snapshots (including for losses)
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.tar.gz";

    for path in glob(&pattern)?.flatten() {
        let (article, .., info) = PmcOapCodec.from_path(&path, None).await?;

        let pmcid = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".tar.gz").map(String::from))
            .ok_or_eyre("should have .tar.gz suffix")?;

        assert_json_snapshot!(format!("{pmcid}.json"), article);
        assert_yaml_snapshot!(format!("{pmcid}.decode.losses"), info.losses);
    }

    Ok(())
}
