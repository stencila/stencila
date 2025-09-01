use std::path::PathBuf;

use codec::{
    Codec,
    eyre::{OptionExt, Result},
};
use common_dev::insta::{assert_json_snapshot, assert_yaml_snapshot};

use codec_meca::MecaCodec;

/// Decode each example of a PMC OA Package and create JSON snapshots (including for losses)
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.meca";

    for path in glob::glob(&pattern)?.flatten() {
        let (article, .., info) = MecaCodec.from_path(&path, None).await?;

        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".meca").map(String::from))
            .ok_or_eyre("should have .meca suffix")?;

        // Redact inlined image dataURIs which can be very large
        assert_json_snapshot!(format!("{id}.json"), article, {
            ".content[].contentUrl" => "redacted",
            ".content[].content[].contentUrl" => "redacted",
            ".content[].content[].content[].contentUrl" => "redacted",
            ".content[].content[].content[].content[].contentUrl" => "redacted"
        });

        assert_yaml_snapshot!(format!("{id}.decode.losses"), info.losses);
    }

    Ok(())
}
