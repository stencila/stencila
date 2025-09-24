use std::path::PathBuf;

use glob::glob;
use insta::assert_json_snapshot;

use stencila_codec::{
    Codec,
    eyre::{Context, OptionExt, Result, eyre},
};
use stencila_codec_openalex::OpenAlexCodec;

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

        let (node, ..) = OpenAlexCodec
            .from_path(&path, None)
            .await
            .wrap_err_with(|| eyre!("Unable to deserialize {id}.json"))?;

        assert_json_snapshot!(id, node, {
            ".commit" => "redacted"
        });
    }

    Ok(())
}
