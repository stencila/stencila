//! Tests on examples of Stencila documents

use std::{collections::BTreeMap, fs::File, path::PathBuf};

use codec::{
    common::{
        eyre::{Context, Result},
        glob::glob,
        itertools::Itertools,
        once_cell::sync::Lazy,
        serde::{Deserialize, Serialize},
        serde_json, serde_yaml,
        smart_default::SmartDefault,
        tokio::{
            self,
            fs::{read_to_string, remove_file, write},
        },
    },
    format::Format,
    DecodeOptions, EncodeOptions,
};
use common_dev::pretty_assertions::assert_eq;
use json_value_merge::Merge;
use node_strip::{StripNode, StripTargets};

type Config = BTreeMap<String, FormatConfig>;

/// Config for a format which can be read from file
#[derive(Debug, SmartDefault, Clone, Serialize, Deserialize)]
#[serde(crate = "codec::common::serde")]
struct FormatConfig {
    #[default(Format::Json)]
    format: Format,
    encode: EncodeConfig,
    decode: DecodeConfig,
}

/// Config for testing the encoding of a format
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(crate = "codec::common::serde")]
struct EncodeConfig {
    skip: bool,
    #[serde(flatten)]
    options: EncodeOptions,
}

/// Config for testing the decoding of a format
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(crate = "codec::common::serde")]
struct DecodeConfig {
    skip: bool,
    #[serde(flatten)]
    options: DecodeOptions,
}

/// Default config
static CONFIG: Lazy<Config> = Lazy::new(|| {
    BTreeMap::from([
        (
            String::from("html"),
            FormatConfig {
                format: Format::Html,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        compact: Some(false),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                decode: DecodeConfig {
                    // TODO
                    skip: true,
                    ..Default::default()
                },
            },
        ),
        (
            String::from("compact.html"),
            FormatConfig {
                format: Format::Html,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        compact: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                decode: DecodeConfig {
                    // TODO
                    skip: true,
                    ..Default::default()
                },
            },
        ),
        (
            String::from("standalone.html"),
            FormatConfig {
                format: Format::Html,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        standalone: Some(true),
                        compact: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                decode: DecodeConfig {
                    skip: true,
                    ..Default::default()
                },
            },
        ),
        (
            String::from("jats.xml"),
            FormatConfig {
                format: Format::Jats,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        standalone: Some(true),
                        compact: Some(false),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                decode: DecodeConfig {
                    // Do not test decoding since it is tested on
                    // compact.jats.xml and prettifying can affect whitespace
                    skip: true,
                    ..Default::default()
                },
            },
        ),
        (
            String::from("compact.jats.xml"),
            FormatConfig {
                format: Format::Jats,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        standalone: Some(true),
                        compact: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            String::from("json5"),
            FormatConfig {
                format: Format::Json5,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        compact: Some(false),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            String::from("compact.json5"),
            FormatConfig {
                format: Format::Json5,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        compact: Some(true),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            String::from("md"),
            FormatConfig {
                format: Format::Markdown,
                ..Default::default()
            },
        ),
        (
            String::from("txt"),
            FormatConfig {
                format: Format::Text,
                decode: DecodeConfig {
                    skip: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            String::from("yaml"),
            FormatConfig {
                format: Format::Yaml,
                ..Default::default()
            },
        ),
    ])
});

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
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/nodes")
        .canonicalize()?;

    let pattern = dir.join("*/*.json");
    let pattern = pattern.to_str().unwrap_or_default();

    let examples = glob(pattern)?.flatten().collect_vec();

    for path in examples {
        let name = path.file_name().unwrap().to_string_lossy();
        eprintln!("{name}");

        let node = codecs::from_path(&path, None).await?;

        // If the folder has a config.yaml file then read it in and merge into the
        // default config.
        let config = path.parent().unwrap().join("config.yaml");
        let config: Config = if config.exists() {
            let overrides: serde_json::Value = serde_yaml::from_reader(File::open(&config)?)?;
            let mut config: serde_json::Value = serde_json::to_value(CONFIG.clone())?;
            config.merge(&overrides);
            serde_json::from_value(config)?
        } else {
            CONFIG.clone()
        };

        for (extension, config) in config {
            eprintln!("  - {extension}");

            let mut file = path.clone();
            file.set_extension(extension.as_str());

            let codec = codecs::get(None, Some(config.format), None)?;

            let mut original = node.clone();

            if !config.encode.skip {
                // Encoding: encode to string, rather than direct to file, if possible
                // for better comparison of differences

                // Apply encode strip options
                let targets = StripTargets {
                    scopes: config.encode.options.strip_scopes.clone(),
                    types: config.encode.options.strip_types.clone(),
                    properties: config.encode.options.strip_props.clone(),
                };
                original.strip(&targets);

                if codec.supports_to_string() {
                    // Encode to string
                    let (actual, losses) = codec
                        .to_string(&original, Some(config.encode.options.clone()))
                        .await?;

                    if file.exists() {
                        // Existing file: compare string content of files
                        let expected = read_to_string(&file).await?;
                        if actual != expected {
                            if std::env::var("UPDATE_EXAMPLES").unwrap_or_default() == "true" {
                                write(&file, actual).await?;
                            } else {
                                assert_eq!(
                                    actual,
                                    expected,
                                    "Encoded file differs\nConfig:{config}",
                                    config = serde_json::to_string_pretty(&config)?
                                );
                            }
                        }
                    } else if !actual.is_empty() {
                        // No existing file: write a new one
                        write(&file, actual).await?;
                    }

                    // Write any losses to file
                    let mut losses_file = path.clone();
                    losses_file.set_extension([extension.as_str(), ".encode.losses"].concat());
                    if losses.is_empty() {
                        remove_file(losses_file).await.ok();
                    } else {
                        write(losses_file, serde_yaml::to_string(&losses)?).await?;
                    }
                } else {
                    // Just encode to file if it does not yet exist. At present not attempting
                    // to compared binary files (e.g. may include timestamps and change each run)
                    if !file.exists() {
                        codec
                            .to_path(&original, &file, Some(config.encode.options.clone()))
                            .await?;
                    }
                }
            }

            if file.exists() && !config.decode.skip {
                // Decode from file
                let (mut decoded, losses) = codec
                    .from_path(&file, Some(config.decode.options.clone()))
                    .await
                    .wrap_err_with(|| format!("while decoding {}", file.display()))?;

                // Write any losses to file
                let mut losses_file = path.clone();
                losses_file.set_extension([&extension, ".decode.losses"].concat());
                if losses.is_empty() {
                    remove_file(losses_file).await.ok();
                } else {
                    write(losses_file, serde_yaml::to_string(&losses)?).await?;
                }

                // Apply decode strip options to both original and decoded value for fair valid comparison
                let targets = StripTargets {
                    scopes: config.decode.options.strip_scopes.clone(),
                    types: config.decode.options.strip_types.clone(),
                    properties: config.decode.options.strip_props.clone(),
                };
                decoded.strip(&targets);
                original.strip(&targets);

                assert_eq!(
                    decoded,
                    original,
                    "Decoded node differs\nConfig:{config}",
                    config = serde_json::to_string_pretty(&config)?
                );
            }
        }
    }

    Ok(())
}
