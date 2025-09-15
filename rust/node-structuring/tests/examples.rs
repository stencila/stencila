use std::{
    fs::{read_dir, read_to_string},
    path::PathBuf,
    str::FromStr,
};

use eyre::{OptionExt, Result, bail};
use insta::assert_json_snapshot;
use itertools::Itertools;

use stencila_codec::{Codec, StructuringOperation, StructuringOptions, stencila_format::Format};
use stencila_codec_json::JsonCodec;
use stencila_codec_markdown::MarkdownCodec;
use stencila_node_structuring::structuring;
use stencila_schema::{Node, Primitive};

/// Structure example Markdown or JSON files using the structuring operation
/// specified in the first three parts of their name, or multiple operations for
/// files with numeric prefixes.
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
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/examples");

    for entry in read_dir(&dir)?.flatten() {
        let path = entry.path();
        let format = Format::from_path(&path);

        let content = read_to_string(&path)?;

        let (mut node, ..) = match format {
            Format::Markdown => MarkdownCodec.from_str(&content, None).await?,
            Format::Json => JsonCodec.from_str(&content, None).await?,
            _ => bail!("Unsupported formatted"),
        };

        let id = path
            .file_stem()
            .map(|name| name.to_string_lossy().to_string())
            .ok_or_eyre("No file stem")?;

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
                bail!("Unable to determine structuring operations for {}", path.display());
            }
        };

        structuring(&mut node, StructuringOptions::new(operations, []));

        assert_json_snapshot!(id, node);
    }

    Ok(())
}
