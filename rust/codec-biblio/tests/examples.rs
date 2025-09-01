use std::{fs::read_to_string, path::PathBuf};

use glob::glob;

use codec::{
    Codec, DecodeOptions, EncodeOptions,
    eyre::{OptionExt, Result},
    format::Format,
};
use codec_biblio::BiblioCodec;
use common_dev::{insta::assert_json_snapshot, pretty_assertions::assert_eq};

/// Decode bibliographic files and test round-trip conversions
#[tokio::test]
async fn examples() -> Result<()> {
    let examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?;

    let yaml_files: Vec<_> = glob(&examples_dir.join("*.yaml").to_string_lossy())?
        .flatten()
        .collect();

    for path in yaml_files {
        let file_stem = path
            .file_stem()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or_eyre("should have file stem")?;

        let yaml = read_to_string(&path)?;

        // Decode Hayagriva YAML
        let (node, ..) = BiblioCodec
            .from_str(
                &yaml,
                Some(DecodeOptions {
                    format: Some(Format::Yaml),
                    ..Default::default()
                }),
            )
            .await?;

        // Snapshot to test (and debug) decoding to Stencila schema node
        assert_json_snapshot!(file_stem.clone(), node);

        // Test round-trip Hayagriva YAML, this tests the functions in
        // conversion.rs, in particular that they do not introduce loss.
        // Hayagriva YAML is not an important encoding target. But by ensuring
        // we can at least convert back to Hayagriva entries, we have more
        // certainty that rendering to other formats is working as expected.
        let (encoded_yaml, ..) = BiblioCodec
            .to_string(
                &node,
                Some(EncodeOptions {
                    format: Some(Format::Yaml),
                    ..Default::default()
                }),
            )
            .await?;

        assert_eq!(
            encoded_yaml, yaml,
            "Round-trip test failed for Hayagriva YAML",
        );

        // Test rendering to each citation style
        for style in ["apa", "mla", "chicago", "vancouver", "ieee"] {
            // Encode to alternative formats mainly for debugging
            for format in [Format::Json, Format::Markdown, Format::Text] {
                let (encoded, ..) = BiblioCodec
                    .to_string(
                        &node,
                        Some(EncodeOptions {
                            format: Some(format.clone()),
                            theme: Some(style.into()),
                            ..Default::default()
                        }),
                    )
                    .await?;

                // Write generated files for debugging
                let output_file = format!("{file_stem}.{style}.{format}");
                let output_path = examples_dir.join(&output_file);
                std::fs::write(&output_path, &encoded)?;

                if format == Format::Text {
                    /*
                    TODO: round trip test of decoding plain text for citation style

                    // Round-trip test text
                    let (round_trip, ..) = BiblioCodec
                        .from_str(
                            &encoded,
                            Some(DecodeOptions {
                                format: Some(Format::Text),
                                ..Default::default()
                            }),
                        )
                        .await?;

                    assert_eq!(
                        round_trip, node
                        "Round-trip test failed for {output_file}",
                    );
                    */
                }
            }
        }
    }

    Ok(())
}
