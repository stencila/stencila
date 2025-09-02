use std::{fs::read_to_string, path::PathBuf, sync::LazyLock};

use glob::glob;
use regex::Regex;

use insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot};
use stencila_codec::{EncodeOptions, eyre::Result, stencila_format::Format};
use stencila_codec_lexical::{decode, encode};

/// Decode each example of a Lexical document and create JSON and Lexical snapshots
/// including snapshots for losses
#[test]
fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*";

    // Redact ids in DOM HTML since these will change between test runs
    fn redact(content: &str) -> String {
        static ID_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r" id=[a-z]{3}_\w+").expect("invalid_regex"));

        ID_REGEX.replace_all(content, " id=xxx").to_string()
    }

    for path in glob(&format!("{pattern}.lexical"))?
        .chain(glob(&format!("{pattern}.koenig"))?)
        .flatten()
    {
        let format = Format::from_path(&path);
        let ext = format.extension();

        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(&[".", &ext].concat()).map(String::from))
            .expect("should have format suffix");

        let original = read_to_string(path)?;

        let (article, info) = decode(&original, None)?;

        assert_json_snapshot!(format!("{name}.json"), article);
        assert_yaml_snapshot!(format!("{name}.decode.losses"), info.losses);

        let (lexical, info) = encode(
            &article,
            Some(EncodeOptions {
                format: Some(format.clone()),
                compact: Some(false),
                ..Default::default()
            }),
        )?;

        assert_snapshot!(format!("{name}.{ext}"), redact(&lexical));
        assert_yaml_snapshot!(format!("{name}.encode.losses"), info.losses);

        let (lexical, ..) = encode(
            &article,
            Some(EncodeOptions {
                format: Some(format),
                standalone: Some(true),
                compact: Some(false),
                ..Default::default()
            }),
        )?;
        assert_snapshot!(format!("{name}.standalone.{ext}"), redact(&lexical));
    }

    Ok(())
}
