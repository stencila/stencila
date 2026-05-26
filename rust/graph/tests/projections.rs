//! Snapshot selected workspace graph projections as Graphviz DOT.
//!
//! Raw graph fixture snapshots cover extraction. These snapshots cover the
//! projected, visualization-facing graph shape for representative presets.

use std::path::PathBuf;

use eyre::Result;
use stencila_graph::{
    GraphProjectionDetail, GraphProjectionOptions, GraphProjectionPreset, WorkspaceOptions,
    dot::to_dot, graph_from_path, project_graph,
};

#[tokio::test]
async fn projections() -> Result<()> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    for case in projection_cases() {
        let graph = graph_from_path(
            fixtures_dir.join(case.fixture),
            Some(fixture_options(case.fixture)),
        )
        .await?;
        let view = project_graph(&graph, &case.options());

        insta::assert_snapshot!(format!("{}__{}", case.fixture, case.name), to_dot(&view));
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ProjectionCase {
    fixture: &'static str,
    name: &'static str,
    preset: GraphProjectionPreset,
    detail: Option<GraphProjectionDetail>,
}

impl ProjectionCase {
    fn options(self) -> GraphProjectionOptions {
        GraphProjectionOptions {
            preset: self.preset,
            detail: self.detail.unwrap_or_default(),
            ..Default::default()
        }
    }
}

fn projection_cases() -> [ProjectionCase; 11] {
    use GraphProjectionDetail::{High, Low, Medium};
    use GraphProjectionPreset::{Cite, Deps, Flow, React};

    [
        ProjectionCase {
            fixture: "code-workflow-snakemake-rnaseq",
            name: "flow-medium",
            preset: Flow,
            detail: Some(Medium),
        },
        ProjectionCase {
            fixture: "code-workflow-snakemake-rnaseq",
            name: "flow-high",
            preset: Flow,
            detail: Some(High),
        },
        ProjectionCase {
            fixture: "code-workflow-nextflow-metagenomics",
            name: "flow-medium",
            preset: Flow,
            detail: Some(Medium),
        },
        ProjectionCase {
            fixture: "code-workflow-nextflow-metagenomics",
            name: "flow-high",
            preset: Flow,
            detail: Some(High),
        },
        ProjectionCase {
            fixture: "code-python-r-dataframe-provenance",
            name: "flow-low",
            preset: Flow,
            detail: Some(Low),
        },
        ProjectionCase {
            fixture: "code-python-r-dataframe-provenance",
            name: "flow-medium",
            preset: Flow,
            detail: Some(Medium),
        },
        ProjectionCase {
            fixture: "code-python-r-dataframe-provenance",
            name: "flow-high",
            preset: Flow,
            detail: Some(High),
        },
        ProjectionCase {
            fixture: "document-figures-code-and-ai-generated",
            name: "flow-medium",
            preset: Flow,
            detail: Some(Medium),
        },
        ProjectionCase {
            fixture: "document-report-references-citations",
            name: "cite",
            preset: Cite,
            detail: None,
        },
        ProjectionCase {
            fixture: "environment-manifests-lockfiles-polyglot",
            name: "deps",
            preset: Deps,
            detail: None,
        },
        ProjectionCase {
            fixture: "document-executable-reactivity-notebook",
            name: "react",
            preset: React,
            detail: None,
        },
    ]
}

fn fixture_options(name: &str) -> WorkspaceOptions {
    WorkspaceOptions {
        subject: Some(format!("fixture:{name}")),
        fail_on_decode_error: name.starts_with("document-"),
        source_metadata: false,
        ..Default::default()
    }
}
