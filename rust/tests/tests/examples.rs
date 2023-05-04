//! Tests on examples of Stencila documents

use std::{fs::read_dir, path::PathBuf};

use codecs::{DecodeOptions, EncodeOptions};
use common::{
    eyre::Result,
    itertools::Itertools,
    tokio::{
        self,
        fs::{read_to_string, write},
    },
};
use common_dev::pretty_assertions::assert_eq;
use format::Format;

/// Get a list of all files in the `examples` folder
fn examples() -> Result<Vec<PathBuf>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .canonicalize()?;

    let files = read_dir(dir)?
        .flatten()
        .map(|path| path.path())
        .collect_vec();

    Ok(files)
}

/// Test the encoding/decoding of examples to/from various formats
///
/// For each `examples/*.json` file, load it as a `Node`, and then for
/// each format:
///
/// 1. Encode to the format and compare the current file
/// with the corresponding file extension. If no such file exists then
/// write the file.
///
/// 2. Decode the existing file to a `Node` and compare it to the orginal
/// node (from the JSON example).
///
/// Use the `UPDATE_EXAMPLES` environment vaiable to overwrite any existing
/// files e.g.
///
///   UPDATE_EXAMPLES=true cargo test -p tests examples_encode_decode
#[tokio::test]
async fn examples_encode_decode() -> Result<()> {
    // Excludes developer focussed and/or unstable formats `Debug` and `Ron`
    // as well as those under development.
    const FORMATS: &[Format] = &[Format::Json5, Format::Yaml];

    let examples = examples()?;

    for path in examples
        .iter()
        .filter(|path| path.to_string_lossy().ends_with(".json"))
    {
        let name = path.file_name().unwrap().to_string_lossy();

        let node = codecs::from_path(path, None).await?;

        for format in FORMATS {
            let mut file = path.clone();
            file.set_extension(&format.get_extension());

            let codec = codecs::spec(&format.to_string())?;

            // Encoding: encode to string, rather than direct to file, if possible
            // for better comparison of differences

            let encode_options = EncodeOptions {
                format: Some(*format),
                ..Default::default()
            };

            if codec.supports_to_string {
                let actual = codecs::to_string(&node, Some(encode_options)).await?;

                if file.exists() {
                    // Exisiting file: compare string content of files
                    let expected = read_to_string(&file).await?;
                    if actual != expected {
                        if std::env::var("UPDATE_EXAMPLES").unwrap_or_default() == "true" {
                            write(&file, actual).await?;
                        } else {
                            assert_eq!(
                                actual, expected,
                                "Example `{name}`, format `{format}`: encoded file differs",
                            );
                        }
                    }
                } else {
                    // No existing file: write a new one
                    write(&file, actual).await?;
                }
            } else {
                // Just file if it does not yet exists. At present not attempting
                // to compared binary files (e.g. may include timestamps and change each run)
                if !file.exists() {
                    codecs::to_path(&node, &file, Some(encode_options)).await?;
                }
            }

            // Decoding: always from the file

            let decode_options = DecodeOptions {
                format: Some(*format),
                ..Default::default()
            };
            let actual = codecs::from_path(&file, Some(decode_options)).await?;
            assert_eq!(
                actual, node,
                "Example `{name}`, format `{format}`: decoded node differs"
            );
        }
    }

    Ok(())
}
