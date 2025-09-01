use std::{fs::read_to_string, path::PathBuf};

use codec::{
    Codec,
    eyre::{OptionExt, Result},
};
use codec_zenodo::ZenodoCodec;
use common_dev::insta::assert_json_snapshot;

/// Decode Zenodo API responses into Stencila schema nodes
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.json";

    for path in glob::glob(&pattern)?.flatten() {
        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".json").map(String::from))
            .ok_or_eyre("should have .json suffix")?;

        let json = read_to_string(&path)?;
        let (node, ..) = ZenodoCodec.from_str(&json, None).await?;

        assert_json_snapshot!(id, node);
    }

    Ok(())
}
