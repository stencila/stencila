//! Snapshot unresolved-I/O diagnostics across the fixture corpus.
//!
//! The count and content of these diagnostics is the regression metric for
//! static analysis permissiveness. Widening resolution should move operations
//! out of this snapshot; it must never move an operation in without a
//! corresponding negative fixture explaining why resolution declined.

use std::{fs::read_dir, path::PathBuf};

use eyre::Result;
use stencila_graph::{
    StaticAnalysisDiagnostic, WorkspaceOptions, graph_from_path_with_diagnostics,
};

#[tokio::test]
async fn fixture_diagnostics() -> Result<()> {
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

    let mut report = Vec::new();
    let mut total = 0;

    for fixture in fixtures {
        let name = fixture.file_name().to_string_lossy().to_string();
        let path = fixture.path();

        if read_dir(&path)?.next().is_none() {
            continue;
        }

        #[cfg(not(unix))]
        if name == "workspace-symlinks-publication-assets" {
            continue;
        }

        let analysis = graph_from_path_with_diagnostics(
            path,
            Some(WorkspaceOptions {
                subject: Some(format!("fixture:{name}")),
                ..Default::default()
            }),
        )
        .await?;

        total += analysis.diagnostics.len();
        report.push(format!(
            "# {name}: {} unresolved\n\n{}",
            analysis.diagnostics.len(),
            analysis
                .diagnostics
                .iter()
                .map(StaticAnalysisDiagnostic::to_string)
                .collect::<Vec<_>>()
                .join("\n\n")
        ));
    }

    report.push(format!("# corpus total: {total} unresolved"));

    insta::assert_snapshot!("fixture-diagnostics", report.join("\n\n"));

    Ok(())
}
