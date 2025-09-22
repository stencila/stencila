use std::path::PathBuf;

use insta::{assert_json_snapshot, assert_yaml_snapshot};
use stencila_codec::{
    Codec, DecodeOptions,
    eyre::{OptionExt, Result},
    stencila_format::Format,
};

use stencila_codec_pmc::PmcCodec;
use stencila_node_structuring::structuring;

/// Decode each example of a PMC OA Package (tar.gz) and HTML and create JSON
/// snapshots
///
/// Diffing the pairs of JSON snapshots can be useful to identify areas that the
/// HTML decoding can be improved e.g.
///
/// ```sh
/// cd rust/codec-pmcoa/tests/snapshots/
/// diff examples__PMC11518443.html.json.snap examples__PMC11518443.tar.json.snap
/// ```
///
/// The standard structuring options for each format are applied to be able to
/// more test those and more easily compare article decoded from PMC OA packages
/// (tar) with those decoded from HTML.
#[tokio::test]
async fn examples() -> Result<()> {
    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?;

    // Do not embed supplements so that JSON snapshot based on HTML are more
    // easily comparable to those based on TAR package
    let options = Some(DecodeOptions {
        embed_supplements: Some(false),
        ..Default::default()
    });

    // Test tar.gz files
    let tar_pattern = base_dir.to_string_lossy().to_string() + "/**/*.tar.gz";
    for path in glob::glob(&tar_pattern)?.flatten() {
        let (mut article, .., info) = PmcCodec.from_path(&path, options.clone()).await?;

        structuring(&mut article, PmcCodec.structuring_options(&Format::PmcOa));

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
    // Do not snapshot losses for these since it is more effective to diff the JSON snapshot
    // with the one provided from the tar
    let html_pattern = base_dir.to_string_lossy().to_string() + "/**/*.html";
    for path in glob::glob(&html_pattern)?.flatten() {
        let (mut article, ..) = PmcCodec.from_path(&path, options.clone()).await?;

        structuring(&mut article, PmcCodec.structuring_options(&Format::Html));

        let pmcid = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".html").map(String::from))
            .ok_or_eyre("should have .html suffix")?;

        // Redact inlined media dataURIs which can be very large
        assert_json_snapshot!(format!("{pmcid}.html.json"), article, {
            ".**.contentUrl" => "redacted"
        });
    }

    Ok(())
}
