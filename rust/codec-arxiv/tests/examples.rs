use std::path::PathBuf;

use glob::glob;
use insta::{assert_json_snapshot, assert_yaml_snapshot};

use stencila_codec::eyre::{OptionExt, Result};
use stencila_codec_arxiv::decode_html::decode_arxiv_html;

/// Decode each example of an arXiv HTML page and create JSON snapshots (including for losses)
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.html";

    for path in glob(&pattern)?.flatten() {
        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".html").map(String::from))
            .ok_or_eyre("should have .html suffix")?;

        let (article, .., info) = decode_arxiv_html(&id, &path, None).await?;

        // Redact inlined image dataURIs and mathml which can be very large
        assert_json_snapshot!(format!("{id}.json"), article, {
            ".**.contentUrl" => "redacted",
            ".**.mathml" => "redacted",
            ".commit" => "redacted"
        });

        assert_yaml_snapshot!(format!("{id}.decode.losses"), info.losses);
    }

    Ok(())
}
