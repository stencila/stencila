use codecs::{DecodeOptions, Format};
use common::syn::Lit::Str;
use common::{eyre, eyre::Result, eyre::WrapErr, tokio};
use common_dev::insta::assert_debug_snapshot;
use glob::glob;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use node_merge::{DiffResult, MergeNode};

type MarkdownDiffs = HashMap<String, (String, String)>;

fn load_markdown(folder_path: &str) -> Result<MarkdownDiffs> {
    let pattern = format!("{}/*.md", folder_path);
    let mut files_content = HashMap::new();

    for entry in glob(&pattern).wrap_err("Failed to read glob pattern")? {
        let path = entry.wrap_err("Failed to process file entry")?;
        let file_stem = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| eyre::eyre!("Invalid file stem for {:?}", path))
            .wrap_err("Failed to extract file stem")?;

        let mut file =
            File::open(&path).wrap_err_with(|| format!("Failed to open file {:?}", path))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .wrap_err_with(|| format!("Failed to read contents of file {:?}", path))?;

        let mut parts = contents.splitn(2, "===\n").map(String::from);
        let (first, second) = (
            parts.next().unwrap_or_default(),
            parts.next().unwrap_or_default(),
        );

        files_content.insert(file_stem.to_string(), (first, second));
    }

    Ok(files_content)
}

/// Snapshot tests of the `MergeNode::diff` method
#[tokio::test]
async fn file_diff() -> Result<()> {
    async fn diff(files: &MarkdownDiffs, name: &str) -> Result<DiffResult> {
        let options = Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        });
        let (old, new) = files.get(name).ok_or(eyre::eyre!("{} not found", name))?;
        let old = codecs::from_str(old, options.clone()).await?;
        let new = codecs::from_str(new, options).await?;

        Ok(old.diff(new))
    }

    let files_content = load_markdown("tests/markdown")?;
    for name in files_content.keys() {
        assert_debug_snapshot!(name.to_string(), diff(&files_content, name).await?);
    }

    Ok(())
}
