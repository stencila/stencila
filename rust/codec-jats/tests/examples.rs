use std::{fs::read_to_string, path::PathBuf};

use codec::{
    common::{eyre::Result, glob::glob},
    EncodeOptions,
};

use codec_jats::{decode, encode};
use common_dev::insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot};

/// Decode each example of a JATS article and create JSON and JATS snapshots
/// including for losses
#[test]
fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.xml";

    for path in glob(&pattern)?.flatten() {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".xml").map(String::from))
            .expect("should have .xml suffix");

        let original = read_to_string(path)?;

        let (article, info) = decode(&original, None)?;

        assert_json_snapshot!(format!("{name}.json"), article);
        assert_yaml_snapshot!(format!("{name}.decode.losses"), info.losses);

        let (jats, info) = encode(
            &article,
            Some(EncodeOptions {
                compact: Some(false),
                ..Default::default()
            }),
        )?;

        assert_snapshot!(format!("{name}.jats"), jats);
        assert_yaml_snapshot!(format!("{name}.encode.losses"), info.losses);
    }

    Ok(())
}
