//! Snapshot workspace graph fixtures as YAML.
//!
//! Volatile timestamps are redacted through Insta so `cargo insta review`
//! can show normal structured YAML diffs.

use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use eyre::Result;
use stencila_graph::{WorkspaceOptions, graph_from_path};

#[tokio::test]
async fn fixtures() -> Result<()> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    let mut fixtures = read_dir(&fixtures_dir)?
        .flatten()
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|file_type| file_type.is_dir())
                .map(|_| entry)
        })
        .collect::<Vec<_>>();
    fixtures.sort_by_key(|entry| entry.file_name());

    for fixture in fixtures {
        let name = fixture.file_name().to_string_lossy().to_string();
        let path = fixture.path();

        if !fixture_has_content(&path)? {
            continue;
        }

        #[cfg(not(unix))]
        if name == "workspace-symlinks-publication-assets" {
            continue;
        }

        let graph = graph_from_path(path, Some(fixture_options(&name))).await?;

        const DATETIME_PLACEHOLDER: &str = "[datetime]";
        insta::assert_yaml_snapshot!(name, graph, {
            ".commit" => "[commit]",
            ".worktreeStatus" => "[worktreeStatus]",
            ".nodes[].node.startTime.value" => DATETIME_PLACEHOLDER,
            ".nodes[].node.endTime.value" => DATETIME_PLACEHOLDER,
            ".nodes[].node.authors" => "[authors]",
            ".nodes[].node.dateCreated.value" => DATETIME_PLACEHOLDER,
            ".nodes[].node.dateModified.value" => DATETIME_PLACEHOLDER,
            ".edges[].actions[].startTime.value" => DATETIME_PLACEHOLDER,
            ".edges[].actions[].endTime.value" => DATETIME_PLACEHOLDER,
            ".edges[].evidence[].details.reader" => "[c2pa-reader]",
        });
    }

    Ok(())
}

fn fixture_has_content(path: &Path) -> Result<bool> {
    Ok(read_dir(path)?.next().transpose()?.is_some())
}

fn fixture_options(name: &str) -> WorkspaceOptions {
    WorkspaceOptions {
        subject: Some(format!("fixture:{name}")),
        fail_on_decode_error: name.starts_with("document-"),
        source_metadata: true,
        ..Default::default()
    }
}
