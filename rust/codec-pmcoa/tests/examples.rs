use std::path::PathBuf;

use glob::glob;
use insta::{assert_json_snapshot, assert_yaml_snapshot};
use stencila_codec::{
    Codec,
    eyre::{OptionExt, Result},
};

use stencila_codec_pmcoa::PmcOaCodec;

/// Decode each example of a PMC OA Package (tar.gz) and HTML and create JSON snapshots (including for losses)
#[tokio::test]
async fn examples() -> Result<()> {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?;

    // Test tar.gz files
    let tar_pattern = base_dir.to_string_lossy().to_string() + "/**/*.tar.gz";
    for path in glob::glob(&tar_pattern)?.flatten() {
        let (article, .., info) = PmcOaCodec.from_path(&path, None).await?;

        let pmcid = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".tar.gz").map(String::from))
            .ok_or_eyre("should have .tar.gz suffix")?;

        // Redact inlined media dataURIs which can be very large
        assert_json_snapshot!(format!("{pmcid}.tar.json"), article, {
            ".**.contentUrl" => "redacted"
        });
        assert_yaml_snapshot!(format!("{pmcid}.tar.decode.losses"), info.losses);
    }

    // Test HTML files
    let html_pattern = base_dir.to_string_lossy().to_string() + "/**/*.html";
    for path in glob::glob(&html_pattern)?.flatten() {
        let (article, .., info) = PmcOaCodec.from_path(&path, None).await?;

        let pmcid = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".html").map(String::from))
            .ok_or_eyre("should have .html suffix")?;

        // Redact inlined media dataURIs which can be very large
        assert_json_snapshot!(format!("{pmcid}.html.json"), article, {
            ".**.contentUrl" => "redacted"
        });
        assert_yaml_snapshot!(format!("{pmcid}.html.decode.losses"), info.losses);
    }

    Ok(())
}
