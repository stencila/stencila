//! Tests on examples of Stencila documents

use std::path::PathBuf;

use codecs::{DecodeOptions, EncodeOptions};
use common::{
    eyre::Result,
    glob::glob,
    itertools::Itertools,
    tokio::{
        self,
        fs::{read_to_string, write},
    },
};
use common_dev::pretty_assertions::assert_eq;
use format::Format;

/// Get a list of JSON files in the `examples` folder
fn examples() -> Result<Vec<PathBuf>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .canonicalize()?;

    let pattern = dir.join("**/*.json");
    let pattern = pattern.to_str().unwrap_or_default();

    let files = glob(pattern)?.flatten().collect_vec();

    Ok(files)
}

/// Test the encoding/decoding of examples to/from various formats
///
/// For each `examples/*.json` file, load it as a `Node`, and then for
/// each format:
///
/// 1. Encode to the format and compare any existing file
/// with the corresponding file extension. If no such file exists then
/// write the file.
///
/// 2. Decode the existing file to a `Node` and compare it to the original
/// node (from the JSON example).
///
/// Use the `UPDATE_EXAMPLES` environment variable to overwrite any existing
/// files e.g.
///
///   UPDATE_EXAMPLES=true cargo test -p tests examples_encode_decode
#[tokio::test]
async fn examples_encode_decode() -> Result<()> {
    // Formats to encode examples to
    //
    // Excludes developer focussed and/or unstable formats e.g. `Debug`
    let formats: &[(&str, Format, Option<EncodeOptions>, Option<DecodeOptions>)] = &[
        ("html", Format::Html, Some(EncodeOptions::default()), None),
        (
            "compact.html",
            Format::Html,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
            None,
        ),
        (
            "json5",
            Format::Json5,
            Some(EncodeOptions::default()),
            Some(DecodeOptions::default()),
        ),
        (
            "compact.json5",
            Format::Json5,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
            Some(DecodeOptions::default()),
        ),
        ("text", Format::Text, Some(EncodeOptions::default()), None),
        (
            "yaml",
            Format::Yaml,
            Some(EncodeOptions::default()),
            Some(DecodeOptions::default()),
        ),
    ];

    let examples = examples()?;

    for path in examples {
        let name = path.file_name().unwrap().to_string_lossy();

        let node = codecs::from_path(&path, None).await?;

        for (extension, format, encode_options, decode_options) in formats {
            let mut file = path.clone();
            file.set_extension(extension);

            let codec = codecs::spec(&format.to_string())?;

            if let Some(options) = encode_options {
                // Encoding: encode to string, rather than direct to file, if possible
                // for better comparison of differences

                let options = EncodeOptions {
                    format: Some(*format),
                    ..options.clone()
                };

                if codec.supports_to_string {
                    let actual = codecs::to_string(&node, Some(options)).await?;

                    if file.exists() {
                        // Existing file: compare string content of files
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
                    // Just encode to file if it does not yet exist. At present not attempting
                    // to compared binary files (e.g. may include timestamps and change each run)
                    if !file.exists() {
                        codecs::to_path(&node, &file, Some(options)).await?;
                    }
                }
            }

            if let Some(options) = decode_options {
                // Decoding: always from the file

                let options = DecodeOptions {
                    format: Some(*format),
                    ..options.clone()
                };
                let actual = codecs::from_path(&file, Some(options)).await?;
                assert_eq!(
                    actual, node,
                    "Example `{name}`, format `{format}`: decoded node differs"
                );
            }
        }
    }

    Ok(())
}
