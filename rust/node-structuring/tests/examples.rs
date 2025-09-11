use std::{fs::read_to_string, path::PathBuf, str::FromStr};

use eyre::{OptionExt, Result, bail};
use itertools::Itertools;
use stencila_codec::{Codec, StructuringOperation, StructuringOptions};

use insta::assert_json_snapshot;
use stencila_codec_markdown::MarkdownCodec;
use stencila_node_structuring::structuring;
use stencila_schema::{Node, Primitive};

/// Structure example Markdown files using the structuring operation specified
/// in the first three parts of their name, or multiple operations for files
/// with numeric prefixes.
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
        let md = read_to_string(&path)?;
        let (mut node, ..) = MarkdownCodec.from_str(&md, None).await?;

        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".md").map(String::from))
            .ok_or_eyre("should have .md suffix")?;

        let Node::Article(article) = &node else {
            bail!("Expected an article")
        };

        let operations = if let Some(Primitive::Array(ops)) = article
            .options
            .extra
            .as_ref()
            .and_then(|extra| extra.get("structuring").cloned())
        {
            ops.iter()
                .filter_map(|op| match op {
                    Primitive::String(op) => StructuringOperation::from_str(op).ok(),
                    _ => None,
                })
                .collect_vec()
        } else {
            // Fall back to operation from filename for single operation tests
            let op = id.split("-").take(3).collect_vec().join("-");
            if let Ok(operation) = StructuringOperation::from_str(&op) {
                vec![operation]
            } else {
                // Skip files that don't match operation pattern and have no YAML frontmatter
                continue;
            }
        };

        structuring(&mut node, StructuringOptions::new(operations, []));

        assert_json_snapshot!(id, node);
    }

    Ok(())
}
