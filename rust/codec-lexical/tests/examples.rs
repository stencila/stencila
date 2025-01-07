use std::{fs::read_to_string, path::PathBuf};

use codec::{
    common::{eyre::Result, glob::glob},
    format::Format,
    EncodeOptions,
};

use codec_lexical::{decode, encode};
use common_dev::insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot};

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

        assert_snapshot!(format!("{name}.{ext}"), lexical);
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
        assert_snapshot!(format!("{name}.standalone.{ext}"), lexical);
    }

    Ok(())
}
