use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use itertools::Itertools;
use stencila_codec::{
    Codec, StructuringOperation, StructuringOptions,
    eyre::{OptionExt, Result},
};

use insta::assert_json_snapshot;
use stencila_codec_markdown::MarkdownCodec;
use stencila_node_structuring::structuring;

/// Structure example Markdown files using the structuring operation specified
/// in the first three parts of their name.
/// 
/// Run using:
/// 
/// cargo insta test -p stencila-node-structuring
/// 
/// and then review snapshots with:
/// 
/// cargo insta review
#[tokio::test]
async fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.md";

    for path in glob::glob(&pattern)?.flatten() {
        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".md").map(String::from))
            .ok_or_eyre("should have .md suffix")?;

        let op = id.split("-").take(3).collect_vec().join("-");
        let op = StructuringOperation::from_str(&op)?;

        let md = read_to_string(&path)?;
        let (mut node, ..) = MarkdownCodec.from_str(&md, None).await?;

        structuring(&mut node, StructuringOptions::new([op], []));

        assert_json_snapshot!(id, node);
    }

    Ok(())
}
