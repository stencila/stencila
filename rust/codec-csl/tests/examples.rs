use std::{fs::read_to_string, path::PathBuf};

use codec::{
    Codec,
    common::{
        eyre::{OptionExt, Result},
        glob::glob,
        tokio,
    },
};
use codec_csl::CslCodec;
use common_dev::insta::assert_json_snapshot;

/// Decode CSL-JSON into articles
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.json";

    for path in glob(&pattern)?.flatten() {
        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".json").map(String::from))
            .ok_or_eyre("should have .json suffix")?;

        let json = read_to_string(&path)?;
        let (node, ..) = CslCodec.from_str(&json, None).await?;

        assert_json_snapshot!(id, node);
    }

    Ok(())
}
