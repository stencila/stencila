use std::{fs::read_to_string, path::PathBuf};

use codec::{
    Codec,
    eyre::{OptionExt, Result},
};

use codec_cff::CffCodec;
use insta::assert_json_snapshot;

/// Decode CFF files into Stencila nodes
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.cff";

    for path in glob::glob(&pattern)?.flatten() {
        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".cff").map(String::from))
            .ok_or_eyre("should have .cff suffix")?;

        let yaml = read_to_string(&path)?;
        let (node, ..) = CffCodec.from_str(&yaml, None).await?;

        assert_json_snapshot!(id, node);
    }

    Ok(())
}
