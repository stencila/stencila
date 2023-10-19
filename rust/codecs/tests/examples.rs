//! Tests on examples of Stencila documents

use std::{collections::HashMap, fs::File, path::PathBuf};

use codec::{
    common::{
        eyre::{Context, Result},
        glob::glob,
        itertools::Itertools,
        serde::Deserialize,
        serde_yaml,
        tokio::{
            self,
            fs::{read_to_string, remove_file, write},
        },
    },
    format::Format,
    DecodeOptions, EncodeOptions,
};
use common_dev::pretty_assertions::assert_eq;
use node_strip::{StripNode, Targets};

/// Spec for what to tests etc
struct Spec {
    extension: String,
    format: Format,
    encode_options: Option<EncodeOptions>,
    decode_options: Option<DecodeOptions>,
    write_losses: bool,
}

impl Spec {
    fn new(
        extension: &str,
        format: Format,
        encode_options: Option<EncodeOptions>,
        decode_options: Option<DecodeOptions>,
        write_losses: bool,
    ) -> Self {
        Self {
            extension: extension.to_string(),
            format,
            encode_options,
            decode_options,
            write_losses,
        }
    }
}

/// Config for a format which can be read from file
/// TODO: consider merging with `Spec` to allow per folder overrides
/// of everything
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(crate = "codec::common::serde")]
struct Config {
    decode: DecodeConfig,
}

/// Config for testing the decoding of a format
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(crate = "codec::common::serde")]
struct DecodeConfig {
    skip: bool,
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
///   UPDATE_EXAMPLES=true cargo test -p codecs examples
#[tokio::test]
async fn examples() -> Result<()> {
    // Formats to encode examples to
    //
    // Excludes developer focussed and/or unstable formats e.g. `Debug`
    let formats: &[Spec] = &[
        // HTML
        Spec::new(
            "html",
            Format::Html,
            Some(EncodeOptions::default()),
            None,
            true,
        ),
        Spec::new(
            "compact.html",
            Format::Html,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
            None,
            false,
        ),
        Spec::new(
            "standalone.html",
            Format::Html,
            Some(EncodeOptions {
                standalone: Some(true),
                compact: true,
                ..Default::default()
            }),
            None,
            false,
        ),
        // JATS
        Spec::new(
            "jats.xml",
            Format::Jats,
            Some(EncodeOptions {
                standalone: Some(true),
                ..Default::default()
            }),
            // Do not test decoding since it is tested on
            // compact.jats.xml and prettifying can affect whitespace
            None,
            true,
        ),
        Spec::new(
            "compact.jats.xml",
            Format::Jats,
            Some(EncodeOptions {
                standalone: Some(true),
                compact: true,
                ..Default::default()
            }),
            Some(DecodeOptions::default()),
            false,
        ),
        // JSON5
        Spec::new(
            "json5",
            Format::Json5,
            Some(EncodeOptions::default()),
            Some(DecodeOptions::default()),
            true,
        ),
        Spec::new(
            "compact.json5",
            Format::Json5,
            Some(EncodeOptions {
                compact: true,
                ..Default::default()
            }),
            Some(DecodeOptions::default()),
            false,
        ),
        // Markdown
        Spec::new(
            "md",
            Format::Markdown,
            Some(EncodeOptions::default()),
            None,
            true,
        ),
        // Plain text
        Spec::new(
            "txt",
            Format::Text,
            Some(EncodeOptions::default()),
            None,
            true,
        ),
        // YAML
        Spec::new(
            "yaml",
            Format::Yaml,
            Some(EncodeOptions::default()),
            Some(DecodeOptions::default()),
            true,
        ),
    ];

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .canonicalize()?;

    let pattern = dir.join("**/*.json");
    let pattern = pattern.to_str().unwrap_or_default();

    let examples = glob(pattern)?.flatten().collect_vec();

    for path in examples {
        let name = path.file_name().unwrap().to_string_lossy();
        eprintln!("> {name}");

        let config = path.parent().unwrap().join("config.yaml");
        let config: HashMap<String, Config> = if config.exists() {
            let config = File::open(&config)?;
            serde_yaml::from_reader(config)?
        } else {
            HashMap::new()
        };

        let node = codecs::from_path(&path, None).await?;

        for Spec {
            extension,
            format,
            encode_options,
            decode_options,
            write_losses,
        } in formats
        {
            let mut file = path.clone();
            file.set_extension(extension);

            if let Some(options) = encode_options {
                // Encoding: encode to string, rather than direct to file, if possible
                // for better comparison of differences

                let codec = codecs::get(None, Some(*format), None)?;

                let options = EncodeOptions {
                    format: Some(*format),
                    ..options.clone()
                };

                if codec.supports_to_string() {
                    let (actual, losses) = codec.to_string(&node, Some(options)).await?;

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
                    } else if !actual.is_empty() {
                        // No existing file: write a new one
                        write(&file, actual).await?;
                    }

                    let mut losses_file = path.clone();
                    losses_file.set_extension([extension, ".encode.losses"].concat());
                    if losses.is_empty() {
                        remove_file(losses_file).await.ok();
                    } else if *write_losses {
                        write(losses_file, serde_yaml::to_string(&losses)?).await?;
                    }
                } else {
                    // Just encode to file if it does not yet exist. At present not attempting
                    // to compared binary files (e.g. may include timestamps and change each run)
                    if !file.exists() {
                        codec.to_path(&node, &file, Some(options)).await?;
                    }
                }
            }

            if let (true, Some(options)) = (file.exists(), decode_options) {
                // Decoding: always from the file

                let config = config.get(&format.to_string()).cloned().unwrap_or_default();
                if config.decode.skip {
                    continue;
                }

                let codec = codecs::get(None, Some(*format), None)?;
                let lossy_types = codec
                    .lossy_types(None)
                    .iter()
                    .map(|node_type| node_type.to_string())
                    .collect_vec();

                let options = DecodeOptions {
                    format: Some(*format),
                    ..options.clone()
                };
                let (mut decoded, losses) = codec
                    .from_path(&file, Some(options))
                    .await
                    .wrap_err_with(|| format!("while decoding {}", file.display()))?;

                let mut losses_file = path.clone();
                losses_file.set_extension([extension, ".decode.losses"].concat());
                if losses.is_empty() {
                    remove_file(losses_file).await.ok();
                } else if *write_losses {
                    write(losses_file, serde_yaml::to_string(&losses)?).await?;
                }

                // Strip types that the codec is lossy for from both the decoded
                // and original node
                let targets = Targets {
                    types: lossy_types,
                    ..Default::default()
                };
                decoded.strip(&targets);

                let mut stripped = node.clone();
                stripped.strip(&targets);

                assert_eq!(
                    decoded, stripped,
                    "Example `{name}`, format `{format}`: decoded node differs"
                );
            }
        }
    }

    Ok(())
}
