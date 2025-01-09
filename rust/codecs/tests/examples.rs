//! Tests on examples of Stencila documents

use std::{collections::BTreeMap, env, fs::File, path::PathBuf};

use codec::{
    common::{
        eyre::{Context, Result},
        glob::glob,
        itertools::Itertools,
        once_cell::sync::Lazy,
        regex::Regex,
        serde::{Deserialize, Serialize},
        serde_json, serde_yaml,
        smart_default::SmartDefault,
        tokio::{
            self,
            fs::{read_to_string, remove_file, write},
        },
    },
    format::Format,
    schema::NodeType,
    DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions,
};
use common_dev::pretty_assertions::assert_eq;
use json_value_merge::Merge;
use node_strip::{StripNode, StripTargets};

type Config = BTreeMap<String, FormatConfig>;

/// Config for a format which can be read from file
#[derive(Debug, SmartDefault, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, crate = "codec::common::serde")]
struct FormatConfig {
    #[default(Format::Json)]
    format: Format,
    canonical: bool,
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
            "cbor".into(),
            FormatConfig {
                format: Format::Cbor,
                ..Default::default()
            },
        ),
        (
            "cbor.zst".into(),
            FormatConfig {
                format: Format::CborZst,
                ..Default::default()
            },
        ),
        (
            "dom.html".into(),
            FormatConfig {
                format: Format::Dom,
                encode: EncodeConfig {
                    options: EncodeOptions {
                        compact: Some(false),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                decode: DecodeConfig {
                    skip: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "html".into(),
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
                ..Default::default()
            },
        ),
        (
            "compact.html".into(),
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
                ..Default::default()
            },
        ),
        (
            "standalone.html".into(),
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
                ..Default::default()
            },
        ),
        (
            "json".into(),
            FormatConfig {
                format: Format::Json,
                ..Default::default()
            },
        ),
        (
            "json.zip".into(),
            FormatConfig {
                format: Format::JsonZip,
                ..Default::default()
            },
        ),
        (
            "jats.xml".into(),
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
                ..Default::default()
            },
        ),
        (
            "compact.jats.xml".into(),
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
            "json5".into(),
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
            "compact.json5".into(),
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
            "jsonld".into(),
            FormatConfig {
                format: Format::JsonLd,
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
            "compact.jsonld".into(),
            FormatConfig {
                format: Format::JsonLd,
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
            "llmd".into(),
            FormatConfig {
                format: Format::Llmd,
                decode: DecodeConfig {
                    skip: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "smd".into(),
            FormatConfig {
                format: Format::Smd,
                ..Default::default()
            },
        ),
        (
            "myst".into(),
            FormatConfig {
                format: Format::Myst,
                ..Default::default()
            },
        ),
        (
            "qmd".into(),
            FormatConfig {
                format: Format::Qmd,
                ..Default::default()
            },
        ),
        (
            "txt".into(),
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
            "yaml".into(),
            FormatConfig {
                format: Format::Yaml,
                ..Default::default()
            },
        ),
        (
            "pandoc".into(),
            FormatConfig {
                format: Format::Pandoc,
                ..Default::default()
            },
        ),
    ])
});

/// Test the encoding/decoding of examples to/from various formats
///
/// For each `examples/conversion/**/*.json` file, load it as a `Node`, and then for
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
#[allow(clippy::print_stderr)]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/conversion")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/*";

    let update = env::var("UPDATE_EXAMPLES").unwrap_or_default() == "true";
    let include = env::var("INCLUDE_EXAMPLES")
        .ok()
        .map(|var| var.split(',').map(String::from).collect_vec());

    let examples = glob(&pattern)?.flatten().collect_vec();

    for dir in examples {
        if !dir.is_dir() {
            continue;
        }

        let Some(name) = dir.file_name() else {
            continue;
        };
        let name = name.to_string_lossy().to_string();

        if let Some(include) = include.as_ref() {
            if !include.contains(&name) {
                continue;
            }
        }

        eprintln!("{name}");

        // If the folder has a config.yaml file then read it in and merge into the
        // default config.
        let config = dir.join("config.yaml");
        let config: Config = if config.exists() {
            let overrides: serde_json::Value = serde_yaml::from_reader(File::open(&config)?)?;
            let mut config: serde_json::Value = serde_json::to_value(CONFIG.clone())?;
            config.merge(&overrides);
            serde_json::from_value(config)?
        } else {
            CONFIG.clone()
        };

        let canon = config
            .iter()
            .find_map(|(extension, config)| config.canonical.then_some(extension.as_ref()))
            .unwrap_or("json");
        let path = dir.join(format!("{name}.{canon}"));

        let node = codecs::from_path(&path, None).await?;

        for (extension, config) in &config {
            if extension == canon {
                continue;
            }

            eprintln!("  - {extension}");

            let prefix = path
                .file_name()
                .expect("should have name")
                .to_string_lossy();
            let prefix = prefix[..prefix.find('.').expect("should have dot")].to_string();
            let mut file = path.parent().expect("should have parent").join(prefix);
            file.set_extension(extension.as_str());

            let codec = codecs::get(None, Some(&config.format), None)?;

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

                let encode_options = Some(EncodeOptions {
                    format: Some(config.format.clone()),
                    ..config.encode.options.clone()
                });

                if codec.supports_to_string() && !matches!(config.format, Format::JsonZip) {
                    // Encode to string
                    let (mut actual, EncodeInfo { losses, mapping }) =
                        codec.to_string(&original, encode_options).await?;

                    // If DOM HTML redact ids since these will change between test runs
                    if config.format == Format::Dom {
                        static ID_REGEX: Lazy<Regex> =
                            Lazy::new(|| Regex::new(r" id=[a-z]{3}_\w+").expect("invalid_regex"));
                        static HEADING_REGEX: Lazy<Regex> = Lazy::new(|| {
                            Regex::new(r" heading=[a-z]{3}_\w+").expect("invalid_regex")
                        });

                        actual = ID_REGEX.replace_all(&actual, " id=xxx").to_string();
                        actual = HEADING_REGEX
                            .replace_all(&actual, " heading=xxx")
                            .to_string();
                    }

                    if file.exists() {
                        // Existing file: compare string content of files
                        let expected = read_to_string(&file).await?;
                        if actual != expected {
                            if update {
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

                    // Write any mapping to file
                    let mut mapping_file = path.clone();
                    mapping_file.set_extension([extension.as_str(), ".encode.map"].concat());
                    let actual = mapping.to_string();
                    if mapping_file.exists() {
                        let expected = read_to_string(&mapping_file).await?;
                        if actual != expected {
                            if update {
                                if mapping.entries().is_empty() {
                                    remove_file(mapping_file).await.ok();
                                } else {
                                    write(mapping_file, actual).await?;
                                }
                            } else {
                                assert_eq!(actual, expected, "Encode mapping differs");
                            }
                        }
                    } else if !mapping.entries().is_empty() {
                        write(mapping_file, actual).await?;
                    }

                    // Check that mapping ends with one of the supported creative works
                    if let Some(entry) = mapping.entries().last() {
                        assert!(matches!(
                            entry.node_type,
                            NodeType::Article | NodeType::Prompt | NodeType::Chat
                        ));
                    }
                } else {
                    // Just encode to file if it does not yet exist or updating. At present not attempting
                    // to compared binary files (e.g. may include timestamps and change each run)
                    if !file.exists() || update {
                        codec.to_path(&original, &file, encode_options).await?;
                    }
                }
            }

            if file.exists() && !config.decode.skip {
                let decode_options = Some(DecodeOptions {
                    format: Some(config.format.clone()),
                    ..config.decode.options.clone()
                });

                // Decode from file
                let (
                    mut decoded,
                    DecodeInfo {
                        losses, mapping, ..
                    },
                ) = codec
                    .from_path(&file, decode_options)
                    .await
                    .wrap_err_with(|| format!("while decoding {}", file.display()))?;

                // Write any losses to file
                let mut losses_file = path.clone();
                losses_file.set_extension([extension, ".decode.losses"].concat());
                if losses.is_empty() {
                    remove_file(losses_file).await.ok();
                } else {
                    write(losses_file, serde_yaml::to_string(&losses)?).await?;
                }

                // Write any mapping to file
                let mut mapping_file = path.clone();
                mapping_file.set_extension([extension.as_str(), ".decode.map"].concat());
                let actual = mapping.to_string();
                if mapping_file.exists() {
                    let expected = read_to_string(&mapping_file).await?;
                    if actual != expected {
                        if update {
                            if mapping.entries().is_empty() {
                                remove_file(mapping_file).await.ok();
                            } else {
                                write(mapping_file, actual).await?;
                            }
                        } else {
                            assert_eq!(actual, expected, "Decode mapping differs");
                        }
                    }
                } else if !mapping.entries().is_empty() {
                    write(mapping_file, actual).await?;
                }

                // Check that mapping ends with one of the supported creative works
                if let Some(entry) = mapping.entries().last() {
                    assert!(matches!(
                        entry.node_type,
                        NodeType::Article | NodeType::Prompt | NodeType::Chat
                    ));
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
