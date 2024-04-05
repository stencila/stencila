use codecs::{DecodeOptions, Format};
use common::{eyre, eyre::Result, eyre::WrapErr, tokio};
use common_dev::insta::assert_debug_snapshot;
use glob::glob;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use node_merge::{DiffResult, MergeNode};

fn load_markdown(folder_path: &str) -> Result<HashMap<String, String>> {
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

        files_content.insert(file_stem.to_string(), contents);
    }

    Ok(files_content)
}

/// Snapshot tests of the `MergeNode::diff` method
#[tokio::test]
async fn file_diff() -> Result<()> {
    let files_content = load_markdown("tests/markdown")?;

    async fn diff(files: &HashMap<String, String>, old: &str, new: &str) -> Result<DiffResult> {
        let options = Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        });
        let old = files
            .get(old)
            .ok_or(eyre::eyre!("{} not found", old))?
            .to_string();
        let new = files
            .get(new)
            .ok_or(eyre::eyre!("{} not found", new))?
            .to_string();
        let old = codecs::from_str(&old, options.clone()).await?;
        let new = codecs::from_str(&new, options).await?;

        Ok(old.diff(new))
    }

    macro_rules! assert_diff {
        ($old:literal, $new:literal) => {
            // diff(files_content, $old, $new).await?;
            let nm = format!("{}={}", $old, $new);
            assert_debug_snapshot!(nm, diff(&files_content, $old, $new).await?);
        };
    }

    // Begin Test cases -----------------------
    assert_diff!("text1", "text1");
    assert_diff!("text1", "text2");
    assert_diff!("text1", "text3");

    Ok(())
}
