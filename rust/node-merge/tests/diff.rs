use std::{collections::HashMap, fs::read_to_string};

use codecs::{DecodeOptions, Format};
use common::{eyre, eyre::Result, eyre::WrapErr, glob::glob, tokio};
use common_dev::insta::assert_debug_snapshot;

use node_merge::DiffResult;

/// A map of fixture names to tuple of (old, new)
type Fixtures = HashMap<String, (String, String)>;

/// Loads all test fixtures
fn load_fixtures() -> Result<Fixtures> {
    let mut files_content = HashMap::new();

    for entry in glob("tests/fixtures/*.md").wrap_err("Failed to read glob pattern")? {
        let path = entry.wrap_err("Failed to process file entry")?;
        let file_stem = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| eyre::eyre!("Invalid file stem for {:?}", path))?
            .to_string();

        let contents = read_to_string(path)?;

        let mut parts = contents.splitn(2, "===\n").map(String::from);
        let (old, new) = (
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
        );

        files_content.insert(file_stem, (old, new));
    }

    Ok(files_content)
}

/// Snapshot tests of the `MergeNode::diff` method
#[tokio::test]
async fn file_diff() -> Result<()> {
    async fn diff(old: &str, new: &str) -> Result<DiffResult> {
        let options = Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        });

        let old = codecs::from_str(old, options.clone()).await?;
        let new = codecs::from_str(new, options).await?;

        Ok(node_merge::diff(&old, &new))
    }

    for (name, (old, new)) in load_fixtures()? {
        assert_debug_snapshot!(name, diff(&old, &new).await?);
    }

    Ok(())
}
