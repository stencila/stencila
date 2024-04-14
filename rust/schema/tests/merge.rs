//! Snapshot based tests of merging

use std::fs::read_to_string;

use codec::{format::Format, Codec, DecodeOptions};
use codec_markdown::MarkdownCodec;
use common::{eyre::Result, glob::glob, serde::Serialize, tokio};
use common_dev::{insta::assert_yaml_snapshot, pretty_assertions::assert_eq};
use schema::{diff, patch, Node, PatchOp, PatchPath};

/// A fixture
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
struct Fixture {
    // The old node read from the fixture file
    old: Node,

    // The new node read from the fixture file
    new: Node,

    // The operations required to got from old to new
    ops: Vec<(PatchPath, PatchOp)>,
}

/// Snapshot tests of the `MergeNode::diff` method
#[tokio::test]
async fn fixtures() -> Result<()> {
    for path in glob("tests/fixtures/*.md")?.flatten() {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap()
            .to_string();

        let content = read_to_string(path)?;
        let mut parts = content.splitn(2, "===\n").map(String::from);
        let (old, new) = (
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
        );

        let options = Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        });
        let codec = MarkdownCodec {};
        let (old, ..) = codec.from_str(&old, options.clone()).await?;
        let (new, ..) = codec.from_str(&new, options).await?;

        // Calculate the ops
        let ops = diff(&old, &new)?;

        // Apply ops and assert that get new node
        let mut merged = old.clone();
        patch(&mut merged, ops.clone())?;
        assert_eq!(merged, new, "{name}\n{ops:#?}");

        assert_yaml_snapshot!(name, Fixture { old, new, ops });
    }

    Ok(())
}
