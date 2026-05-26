use eyre::Result;
use stencila_schema::{
    Article, CodeChunk, Datatable, DatatableColumn, Directory, Figure, File, Function, Graph,
    GraphEdge, GraphEdgeKind, GraphEvidence, GraphEvidenceConfidence, GraphEvidenceKind, GraphNode,
    Node, Reference, SoftwareApplication, SoftwareSourceCode, Table, Variable,
};

use super::*;

#[test]
fn selects_data_flow_projection_automatically() {
    let view = project_graph(&duplicate_edge_graph(), &GraphProjectionOptions::default());

    assert_eq!(view.preset, GraphProjectionPreset::Flow);
    assert_eq!(
        view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
        vec![GraphEdgeKind::ReadBy]
    );
    assert_eq!(
        view.nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        vec!["code:analysis.py", "file:data.csv"]
    );
}

#[test]
fn prefers_citation_projection_over_generic_flow_edges() {
    let view = project_graph(&graph(), &GraphProjectionOptions::default());

    assert_eq!(view.preset, GraphProjectionPreset::Cite);
    assert_eq!(
        view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
        vec![GraphEdgeKind::CitedBy]
    );
}

#[test]
fn selects_dependency_projection_for_declared_environments() {
    let view = project_graph(&environment_graph(), &GraphProjectionOptions::default());

    assert_eq!(view.preset, GraphProjectionPreset::Deps);
    assert_eq!(view.containment, GraphContainmentMode::Clusters);
    assert_eq!(
        view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
        vec![
            GraphEdgeKind::Declares,
            GraphEdgeKind::Pins,
            GraphEdgeKind::RequiredBy
        ]
    );
}

#[test]
fn collapses_citations_to_document_parent() {
    let view = project_graph(
        &graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Cite,
            ..Default::default()
        },
    );

    assert_eq!(view.edges.len(), 1);
    assert_eq!(view.edges[0].source, "reference:paper");
    assert_eq!(view.edges[0].target, "node:document#article");
    assert_eq!(view.edges[0].edges.len(), 1);
    assert_eq!(
        view.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![GraphViewNodeKind::Document, GraphViewNodeKind::Reference]
    );
}

#[test]
fn cite_projection_keeps_external_links_only() {
    let mut graph = graph();
    graph.nodes.push(graph_node(
        "resource:https%3A//example.org/archive",
        Node::String("https://example.org/archive".to_string()),
    ));
    graph.edges.extend([
        GraphEdge::new(
            "resource:https%3A//example.org/archive".to_string(),
            "node:document#article".to_string(),
            GraphEdgeKind::LinkedBy,
        ),
        GraphEdge::new(
            "file:data.csv".to_string(),
            "node:document#article".to_string(),
            GraphEdgeKind::LinkedBy,
        ),
    ]);

    let view = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Cite,
            ..Default::default()
        },
    );

    assert!(view.edges.iter().any(|edge| {
        edge.kind == GraphEdgeKind::LinkedBy
            && edge.source == "resource:https%3A//example.org/archive"
    }));
    assert!(
        !view
            .edges
            .iter()
            .any(|edge| { edge.kind == GraphEdgeKind::LinkedBy && edge.source == "file:data.csv" }),
        "local file links should stay out of cite projection"
    );
}

#[test]
fn filters_low_confidence_edges() {
    let view = project_graph(
        &graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            include_low_confidence_edges: false,
            ..Default::default()
        },
    );

    assert_eq!(view.edges.len(), 1);
    assert_eq!(view.edges[0].kind, GraphEdgeKind::ReadBy);
}

#[test]
fn adds_structural_ancestors_for_projected_nodes_only() {
    let view = project_graph(
        &graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            include_structure_edges: Some(true),
            ..Default::default()
        },
    );

    assert_eq!(
        view.nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        vec!["code:analysis.py", "file:data.csv", "node:document#article"]
    );
    assert_eq!(
        view.edges
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec![
            "PartOf:code:analysis.py->node:document#article",
            "ReadBy:file:data.csv->code:analysis.py"
        ]
    );
}

#[test]
fn uses_clusters_for_focused_containment_by_default() {
    let view = project_graph(
        &graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    assert_eq!(view.containment, GraphContainmentMode::Clusters);
    assert_eq!(
        view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
        vec![GraphEdgeKind::ReadBy]
    );
    assert_eq!(
        view.containments
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec!["PartOf:code:analysis.py->node:document#article"]
    );
}

#[test]
fn flow_seeds_document_nodes_from_policy() {
    let view = project_graph(
        &document_flow_seed_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let node_ids = view
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    assert!(node_ids.contains(&"node:document#figure"));
    assert!(node_ids.contains(&"node:document#figure-code"));
    assert!(node_ids.contains(&"node:document#figure-image"));
    assert!(node_ids.contains(&"node:document#nested-table"));
    assert!(node_ids.contains(&"node:document#setup"));

    let containments = view
        .containments
        .iter()
        .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
        .collect::<Vec<_>>();
    assert!(
        containments.contains(&"PartOf:node:document#figure-code->node:document#figure".into())
    );
    assert!(
        containments.contains(&"PartOf:node:document#figure-image->node:document#figure".into())
    );
    assert!(
        containments
            .contains(&"PartOf:node:document#nested-table->node:document#figure-image".into())
    );
    assert!(containments.contains(&"PartOf:node:document#setup->node:document#article".into()));
}

#[test]
fn flow_does_not_seed_workspace_nodes_from_document_policy() {
    let view = project_graph(
        &workspace_datatable_seed_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let node_ids = view
        .nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(node_ids, vec!["code:analysis.py", "file:data.csv"]);
}

#[test]
fn uses_all_preset_structure_defaults_after_auto_resolution() {
    let view = project_graph(&structure_only_graph(), &GraphProjectionOptions::default());

    assert_eq!(view.preset, GraphProjectionPreset::All);
    assert_eq!(
        view.nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        vec!["node:document#article", "node:document#figure"]
    );
    assert_eq!(
        view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
        vec![GraphEdgeKind::PartOf]
    );
}

#[test]
fn uses_stable_aggregate_edge_ids_and_summaries() {
    let view = project_graph(
        &duplicate_edge_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    assert_eq!(view.edges.len(), 1);
    assert_eq!(
        view.edges[0].id,
        "edge:ReadBy:file%3Adata.csv:code%3Aanalysis.py"
    );
    assert_eq!(view.edges[0].count, 2);
    assert_eq!(view.edges[0].evidence_count, 1);
    assert!(view.edges[0].low_confidence);
}

#[test]
fn flow_detail_defaults_to_medium_without_local_symbols() {
    let view = project_graph(
        &detail_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    assert!(
        !view
            .nodes
            .iter()
            .any(|node| node.kind == GraphViewNodeKind::Symbol)
    );
    assert!(
        !view
            .nodes
            .iter()
            .any(|node| node.id == "function:analysis.py:python:read_csv")
    );
    assert!(
        view.nodes
            .iter()
            .any(|node| node.id == "column:analysis.py:data.csv:count")
    );
    assert_eq!(
        view.edges
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec![
            "DerivedInto:column:analysis.py:data.csv:count->file:plot.png",
            "Generated:code:analysis.py->file:plot.png",
            "ReadBy:file:data.csv->code:analysis.py",
        ]
    );
}

#[test]
fn flow_medium_hides_derived_into_edges_collapsed_to_code() {
    let view = project_graph(
        &collapsed_derivation_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    assert_eq!(
        view.edges
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec!["ReadBy:datatable:data.csv->code:analysis.py"]
    );
}

#[test]
fn flow_medium_keeps_collapsed_derived_into_without_redundant_read() {
    let view = project_graph(
        &collapsed_derivation_only_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    assert_eq!(
        view.edges
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec!["DerivedInto:datatable:data.csv->code:analysis.py"]
    );
}

#[test]
fn flow_medium_prefers_collapsed_derived_into_over_generated() {
    let view = project_graph(
        &collapsed_generation_and_derivation_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    assert_eq!(
        view.edges
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec!["DerivedInto:code:setup.py->code:plot.py"]
    );
}

#[test]
fn flow_high_keeps_variable_level_derived_into_edges() {
    let view = project_graph(
        &collapsed_derivation_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::High,
            ..Default::default()
        },
    );

    assert!(view.edges.iter().any(|edge| {
        edge.kind == GraphEdgeKind::DerivedInto
            && edge.source == "datatable:data.csv"
            && edge.target == "symbol:analysis.py:r:df"
    }));
}

#[test]
fn flow_detail_low_hides_datatable_columns() {
    let view = project_graph(
        &detail_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::Low,
            ..Default::default()
        },
    );

    assert!(
        !view
            .nodes
            .iter()
            .any(|node| node.kind == GraphViewNodeKind::Datatable)
    );
    assert_eq!(
        view.edges.iter().map(|edge| edge.kind).collect::<Vec<_>>(),
        vec![GraphEdgeKind::Generated, GraphEdgeKind::ReadBy]
    );
}

#[test]
fn flow_detail_high_includes_local_symbols_and_functions() {
    let view = project_graph(
        &detail_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::High,
            ..Default::default()
        },
    );

    assert!(
        view.nodes
            .iter()
            .any(|node| node.id == "symbol:analysis.py:python:df")
    );
    assert!(
        view.nodes
            .iter()
            .any(|node| node.id == "function:analysis.py:python:read_csv")
    );
    assert!(
        view.edges
            .iter()
            .any(|edge| edge.kind == GraphEdgeKind::CalledBy)
    );
}

#[test]
fn labels_document_roots_from_their_scope() {
    let node = graph_node(
        "node:manuscript/report.smd#art_",
        Node::Article(Article::new(Vec::new())),
    );

    assert_eq!(node_label(&node), "report.smd");
}

#[test]
fn connected_filter_matches_exact_node_id() -> Result<()> {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let filtered = filter_graph_view_connected_to(
        &view,
        &["code:scripts/analysis.R".into()],
        GraphConnectedMode::Directed,
    )?;

    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:scripts/analysis.R")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:data/raw.csv")
    );
    assert!(
        !filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:scripts/other.R")
    );
    Ok(())
}

#[test]
fn connected_filter_prefers_exact_text_before_substring() -> Result<()> {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let filtered = filter_graph_view_connected_to(
        &view,
        &["analysis.R".into()],
        GraphConnectedMode::Directed,
    )?;

    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:scripts/analysis.R")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:scripts/analysis.R")
    );
    assert!(
        !filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:archive/analysis.R.old")
    );
    Ok(())
}

#[test]
fn connected_filter_matches_glob_patterns() -> Result<()> {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    for pattern in ["*/analysis.R", "**/analysis.R"] {
        let filtered =
            filter_graph_view_connected_to(&view, &[pattern.into()], GraphConnectedMode::Directed)?;

        assert!(
            filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:scripts/analysis.R")
        );
        assert!(
            !filtered
                .nodes
                .iter()
                .any(|node| node.id == "code:archive/analysis.R.old")
        );
    }
    Ok(())
}

#[test]
fn connected_filter_uses_union_of_best_tier_matches() -> Result<()> {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let filtered = filter_graph_view_connected_to(
        &view,
        &["analysis.R".into()],
        GraphConnectedMode::Directed,
    )?;

    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:scripts/analysis.R")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:r-plot.png")
    );
    Ok(())
}

#[test]
fn connected_filter_does_not_traverse_structure_edges() -> Result<()> {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let filtered = filter_graph_view_connected_to(
        &view,
        &["analysis.R".into()],
        GraphConnectedMode::Directed,
    )?;

    assert!(filtered.nodes.iter().any(|node| node.id == "dir:scripts"));
    assert!(
        !filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:scripts/other.R")
    );
    assert!(
        !filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:scripts/other.R")
    );
    Ok(())
}

#[test]
fn connected_filter_starts_from_contained_descendants() -> Result<()> {
    let view = project_graph(
        &contained_symbol_flow_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::High,
            ..Default::default()
        },
    );

    let filtered = filter_graph_view_connected_to(
        &view,
        &["analysis.py".into()],
        GraphConnectedMode::Directed,
    )?;

    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:analysis.py")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "function:analysis.py:python:summarize")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "symbol:analysis.py:python:summarize:table")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:data/samples.tsv")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:results/python-summary.tsv")
    );
    Ok(())
}

#[test]
fn flow_medium_collapses_contained_symbol_io_to_code() {
    let view = project_graph(
        &contained_symbol_flow_graph(),
        &GraphProjectionOptions::default(),
    );

    assert_eq!(view.preset, GraphProjectionPreset::Flow);
    assert!(
        !view
            .nodes
            .iter()
            .any(|node| node.kind == GraphViewNodeKind::Symbol)
    );
    assert!(
        !view
            .nodes
            .iter()
            .any(|node| node.id == "function:analysis.py:python:summarize")
    );
    assert_eq!(
        view.edges
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec![
            "ReadBy:file:data/samples.tsv->code:analysis.py",
            "WrittenTo:code:analysis.py->file:results/python-summary.tsv",
        ]
    );
}

#[test]
fn flow_medium_hides_local_workflow_execution_calls() {
    let graph = workflow_execution_call_graph();
    let view = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::Medium,
            ..Default::default()
        },
    );

    assert!(
        view.edges
            .iter()
            .all(|edge| edge.kind != GraphEdgeKind::CalledBy)
    );
    assert!(
        !view
            .edges
            .iter()
            .any(|edge| edge.source == "code:main.nf" && edge.target == "workflow-unit:main.nf:qc")
    );
    assert!(
        view.edges
            .iter()
            .any(|edge| edge.source == "file:data/reads.fastq"
                && edge.target == "workflow-unit:main.nf:qc"
                && edge.kind == GraphEdgeKind::ReadBy)
    );
    assert!(
        view.edges
            .iter()
            .any(|edge| edge.source == "workflow-unit:main.nf:qc"
                && edge.target == "file:results/qc/M1-qc.txt"
                && edge.kind == GraphEdgeKind::Generated)
    );

    let high = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::High,
            ..Default::default()
        },
    );

    assert!(high.edges.iter().any(|edge| {
        edge.source == "function:main.nf:nextflow:script"
            && edge.target == "workflow-unit:main.nf:qc"
            && edge.kind == GraphEdgeKind::CalledBy
    }));
}

#[test]
fn flow_medium_hides_redundant_workflow_script_io() {
    let graph = workflow_script_io_graph(false);
    let view = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::Medium,
            ..Default::default()
        },
    );

    assert!(
        !view
            .nodes
            .iter()
            .any(|node| node.id == "code:workflow/scripts/download.py")
    );
    assert!(
        view.edges
            .iter()
            .all(|edge| edge.kind != GraphEdgeKind::UsedBy)
    );
    assert!(!view.edges.iter().any(|edge| {
        edge.source == "code:workflow/scripts/download.py"
            && edge.target == "file:data/raw/S1.fastq"
            && edge.kind == GraphEdgeKind::Generated
    }));
    assert!(view.edges.iter().any(|edge| {
        edge.source == "workflow-unit:Snakefile:download"
            && edge.target == "file:data/raw/S1.fastq"
            && edge.kind == GraphEdgeKind::Generated
    }));

    let high = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::High,
            ..Default::default()
        },
    );

    assert!(high.edges.iter().any(|edge| {
        edge.source == "code:workflow/scripts/download.py"
            && edge.target == "workflow-unit:Snakefile:download"
            && edge.kind == GraphEdgeKind::UsedBy
    }));
    assert!(high.edges.iter().any(|edge| {
        edge.source == "code:workflow/scripts/download.py"
            && edge.target == "file:data/raw/S1.fastq"
            && edge.kind == GraphEdgeKind::Generated
    }));
}

#[test]
fn flow_medium_keeps_workflow_script_fallback_io() {
    let graph = workflow_script_io_graph(true);
    let view = project_graph(
        &graph,
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::Medium,
            ..Default::default()
        },
    );

    assert!(
        view.nodes
            .iter()
            .any(|node| node.id == "code:workflow/scripts/download.py")
    );
    assert!(view.edges.iter().any(|edge| {
        edge.source == "code:workflow/scripts/download.py"
            && edge.target == "workflow-unit:Snakefile:download"
            && edge.kind == GraphEdgeKind::UsedBy
    }));
    assert!(!view.edges.iter().any(|edge| {
        edge.source == "code:workflow/scripts/download.py"
            && edge.target == "file:data/raw/S1.fastq"
            && edge.kind == GraphEdgeKind::Generated
    }));
    assert!(view.edges.iter().any(|edge| {
        edge.source == "code:workflow/scripts/download.py"
            && edge.target == "file:logs/download.log"
            && edge.kind == GraphEdgeKind::Generated
    }));
    assert!(view.edges.iter().any(|edge| {
        edge.source == "workflow-unit:Snakefile:download"
            && edge.target == "file:data/raw/S1.fastq"
            && edge.kind == GraphEdgeKind::Generated
    }));
}

#[test]
fn flow_hides_document_conversion_edges() {
    let view = project_graph(
        &converted_document_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            detail: GraphProjectionDetail::High,
            ..Default::default()
        },
    );

    assert!(
        view.edges
            .iter()
            .all(|edge| edge.kind != GraphEdgeKind::ConvertedInto)
    );
    assert!(
        !view
            .nodes
            .iter()
            .any(|node| node.id == "node:report.html#art_")
    );
    assert!(
        view.edges
            .iter()
            .any(|edge| edge.source == "code:analysis.py"
                && edge.target == "file:report.html"
                && edge.kind == GraphEdgeKind::Generated)
    );

    let all = project_graph(
        &converted_document_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::All,
            ..Default::default()
        },
    );

    assert!(
        all.edges
            .iter()
            .any(|edge| edge.kind == GraphEdgeKind::ConvertedInto)
    );
}

#[test]
fn flow_containment_uses_decoded_document_structure() {
    let view = project_graph(
        &converted_document_containment_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    assert!(
        view.edges
            .iter()
            .all(|edge| edge.kind != GraphEdgeKind::ConvertedInto)
    );
    assert_eq!(
        view.containments
            .iter()
            .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
            .collect::<Vec<_>>(),
        vec![
            "PartOf:dir:docs->dir:.",
            "PartOf:node:docs/notebook.json#art_->dir:docs",
            "PartOf:node:docs/notebook.json#cdc_setup->node:docs/notebook.json#art_",
        ]
    );
}

#[test]
fn flow_low_and_medium_keep_datatable_resources() {
    for detail in [GraphProjectionDetail::Low, GraphProjectionDetail::Medium] {
        let view = project_graph(
            &contained_datatable_symbol_flow_graph(),
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail,
                ..Default::default()
            },
        );

        assert!(
            !view
                .nodes
                .iter()
                .any(|node| node.kind == GraphViewNodeKind::Symbol)
        );
        assert_eq!(
            view.edges
                .iter()
                .map(|edge| format!("{}:{}->{}", edge.kind, edge.source, edge.target))
                .collect::<Vec<_>>(),
            vec![
                "ReadBy:datatable:data/samples.tsv->code:analysis.py",
                "WrittenTo:code:analysis.py->datatable:results/python-summary.tsv",
            ]
        );
    }
}

#[test]
fn connected_filter_does_not_cross_shared_inputs_to_sibling_consumers() -> Result<()> {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let filtered = filter_graph_view_connected_to(
        &view,
        &["code:scripts/analysis.R".into()],
        GraphConnectedMode::Directed,
    )?;

    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:data/raw.csv")
    );
    assert!(
        !filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:scripts/other.R")
    );
    assert!(!filtered.edges.iter().any(
        |edge| edge.source == "file:data/raw.csv" && edge.target == "code:scripts/other.R"
    ));
    Ok(())
}

#[test]
fn connected_filter_undirected_crosses_shared_inputs_to_sibling_consumers() -> Result<()> {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let filtered = filter_graph_view_connected_to(
        &view,
        &["code:scripts/analysis.R".into()],
        GraphConnectedMode::Undirected,
    )?;

    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:scripts/analysis.R")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:data/raw.csv")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "code:scripts/other.R")
    );
    assert!(
        filtered
            .nodes
            .iter()
            .any(|node| node.id == "file:other-plot.png")
    );
    assert!(filtered.edges.iter().any(
        |edge| edge.source == "file:data/raw.csv" && edge.target == "code:scripts/other.R"
    ));
    Ok(())
}

#[test]
fn connected_filter_reports_invalid_and_unmatched_patterns() {
    let view = project_graph(
        &connected_pattern_graph(),
        &GraphProjectionOptions {
            preset: GraphProjectionPreset::Flow,
            ..Default::default()
        },
    );

    let invalid =
        filter_graph_view_connected_to(&view, &["[".into()], GraphConnectedMode::Directed)
            .expect_err("invalid glob should error");
    assert!(
        invalid
            .to_string()
            .contains("invalid connected-to glob pattern")
    );

    let unmatched =
        filter_graph_view_connected_to(&view, &["missing.R".into()], GraphConnectedMode::Directed)
            .expect_err("unmatched pattern should error");
    assert!(
        unmatched
            .to_string()
            .contains("no projected graph nodes match")
    );
}

fn graph() -> Graph {
    let mut graph = Graph::new(
        "test:graph".to_string(),
        vec![
            graph_node(
                "file:data.csv",
                Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
            ),
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node("reference:paper", Node::Reference(reference("Paper"))),
            graph_node(
                "node:document#citation-1",
                Node::String("paper".to_string()),
            ),
            graph_node(
                "node:document#article",
                Node::Article(Article::new(Vec::new())),
            ),
        ],
        vec![
            GraphEdge::new(
                "file:data.csv".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            low_confidence_edge(
                "code:analysis.py",
                "file:plot.png",
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "reference:paper".to_string(),
                "node:document#citation-1".to_string(),
                GraphEdgeKind::CitedBy,
            ),
            GraphEdge::new(
                "node:document#citation-1".to_string(),
                "node:document#article".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "code:analysis.py".to_string(),
                "node:document#article".to_string(),
                GraphEdgeKind::PartOf,
            ),
        ],
    );
    graph.nodes[3].node = Box::new(Node::Citation(Default::default()));
    graph
}

fn connected_pattern_graph() -> Graph {
    Graph::new(
        "test:connected-pattern".to_string(),
        vec![
            graph_node(
                "dir:.",
                Node::Directory(Directory::new("workspace".to_string(), ".".to_string())),
            ),
            graph_node(
                "dir:scripts",
                Node::Directory(Directory::new("scripts".to_string(), "scripts".to_string())),
            ),
            graph_node(
                "code:scripts/analysis.R",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.R".to_string(),
                    path: Some("scripts/analysis.R".to_string()),
                    programming_language: "r".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "file:scripts/analysis.R",
                Node::File(File::new(
                    "analysis.R".to_string(),
                    "scripts/analysis.R".to_string(),
                )),
            ),
            graph_node(
                "file:data/raw.csv",
                Node::File(File::new("raw.csv".to_string(), "data/raw.csv".to_string())),
            ),
            graph_node(
                "file:r-plot.png",
                Node::File(File::new(
                    "r-plot.png".to_string(),
                    "r-plot.png".to_string(),
                )),
            ),
            graph_node(
                "code:scripts/other.R",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "other.R".to_string(),
                    path: Some("scripts/other.R".to_string()),
                    programming_language: "r".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "file:scripts/other.R",
                Node::File(File::new(
                    "other.R".to_string(),
                    "scripts/other.R".to_string(),
                )),
            ),
            graph_node(
                "file:other-data.csv",
                Node::File(File::new(
                    "other-data.csv".to_string(),
                    "other-data.csv".to_string(),
                )),
            ),
            graph_node(
                "file:other-plot.png",
                Node::File(File::new(
                    "other-plot.png".to_string(),
                    "other-plot.png".to_string(),
                )),
            ),
            graph_node(
                "code:archive/analysis.R.old",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.R.old".to_string(),
                    path: Some("archive/analysis.R.old".to_string()),
                    programming_language: "r".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "file:archive-data.csv",
                Node::File(File::new(
                    "archive-data.csv".to_string(),
                    "archive-data.csv".to_string(),
                )),
            ),
            graph_node(
                "file:archive-plot.png",
                Node::File(File::new(
                    "archive-plot.png".to_string(),
                    "archive-plot.png".to_string(),
                )),
            ),
        ],
        vec![
            GraphEdge::new(
                "file:data/raw.csv".to_string(),
                "code:scripts/analysis.R".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "code:scripts/analysis.R".to_string(),
                "file:r-plot.png".to_string(),
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "code:scripts/analysis.R".to_string(),
                "file:scripts/analysis.R".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "file:scripts/analysis.R".to_string(),
                "dir:scripts".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "dir:scripts".to_string(),
                "dir:.".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "file:data/raw.csv".to_string(),
                "code:scripts/other.R".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "code:scripts/other.R".to_string(),
                "file:other-plot.png".to_string(),
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "code:scripts/other.R".to_string(),
                "file:scripts/other.R".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "file:scripts/other.R".to_string(),
                "dir:scripts".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "file:archive-data.csv".to_string(),
                "code:archive/analysis.R.old".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "code:archive/analysis.R.old".to_string(),
                "file:archive-plot.png".to_string(),
                GraphEdgeKind::Generated,
            ),
        ],
    )
}

fn contained_symbol_flow_graph() -> Graph {
    Graph::new(
        "test:contained-symbol-flow".to_string(),
        vec![
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "function:analysis.py:python:summarize",
                Node::Function(Function::new("summarize".to_string(), Vec::new())),
            ),
            graph_node(
                "symbol:analysis.py:python:summarize:table",
                Node::Variable(Variable::new("table".to_string())),
            ),
            graph_node(
                "file:data/samples.tsv",
                Node::File(File::new(
                    "samples.tsv".to_string(),
                    "data/samples.tsv".to_string(),
                )),
            ),
            graph_node(
                "file:results/python-summary.tsv",
                Node::File(File::new(
                    "python-summary.tsv".to_string(),
                    "results/python-summary.tsv".to_string(),
                )),
            ),
        ],
        vec![
            GraphEdge::new(
                "file:data/samples.tsv".to_string(),
                "symbol:analysis.py:python:summarize:table".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "symbol:analysis.py:python:summarize:table".to_string(),
                "file:results/python-summary.tsv".to_string(),
                GraphEdgeKind::WrittenTo,
            ),
            GraphEdge::new(
                "symbol:analysis.py:python:summarize:table".to_string(),
                "function:analysis.py:python:summarize".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "function:analysis.py:python:summarize".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::PartOf,
            ),
        ],
    )
}

fn contained_datatable_symbol_flow_graph() -> Graph {
    Graph::new(
        "test:contained-datatable-symbol-flow".to_string(),
        vec![
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "function:analysis.py:python:summarize",
                Node::Function(Function::new("summarize".to_string(), Vec::new())),
            ),
            graph_node(
                "symbol:analysis.py:python:summarize:table",
                Node::Variable(Variable::new("table".to_string())),
            ),
            graph_node(
                "datatable:data/samples.tsv",
                Node::Datatable(Datatable::new(Vec::new())),
            ),
            graph_node(
                "datatable:results/python-summary.tsv",
                Node::Datatable(Datatable::new(Vec::new())),
            ),
        ],
        vec![
            GraphEdge::new(
                "datatable:data/samples.tsv".to_string(),
                "symbol:analysis.py:python:summarize:table".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "symbol:analysis.py:python:summarize:table".to_string(),
                "datatable:results/python-summary.tsv".to_string(),
                GraphEdgeKind::WrittenTo,
            ),
            GraphEdge::new(
                "symbol:analysis.py:python:summarize:table".to_string(),
                "function:analysis.py:python:summarize".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "function:analysis.py:python:summarize".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::PartOf,
            ),
        ],
    )
}

fn workflow_execution_call_graph() -> Graph {
    Graph::new(
        "test:workflow-execution-call".to_string(),
        vec![
            graph_node(
                "code:main.nf",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "main.nf".to_string(),
                    programming_language: "nextflow".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "function:main.nf:nextflow:script",
                Node::Function(Function::new("script".to_string(), Vec::new())),
            ),
            graph_node(
                "workflow-unit:main.nf:qc",
                Node::Function(Function::new("qc".to_string(), Vec::new())),
            ),
            graph_node(
                "file:data/reads.fastq",
                Node::File(File::new(
                    "reads.fastq".to_string(),
                    "data/reads.fastq".to_string(),
                )),
            ),
            graph_node(
                "file:results/qc/M1-qc.txt",
                Node::File(File::new(
                    "M1-qc.txt".to_string(),
                    "results/qc/M1-qc.txt".to_string(),
                )),
            ),
        ],
        vec![
            GraphEdge::new(
                "code:main.nf".to_string(),
                "workflow-unit:main.nf:qc".to_string(),
                GraphEdgeKind::Declares,
            ),
            GraphEdge::new(
                "function:main.nf:nextflow:script".to_string(),
                "code:main.nf".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "function:main.nf:nextflow:script".to_string(),
                "workflow-unit:main.nf:qc".to_string(),
                GraphEdgeKind::CalledBy,
            ),
            GraphEdge::new(
                "workflow-unit:main.nf:qc".to_string(),
                "code:main.nf".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "file:data/reads.fastq".to_string(),
                "workflow-unit:main.nf:qc".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "workflow-unit:main.nf:qc".to_string(),
                "file:results/qc/M1-qc.txt".to_string(),
                GraphEdgeKind::Generated,
            ),
        ],
    )
}

fn workflow_script_io_graph(include_extra_output: bool) -> Graph {
    let mut nodes = vec![
        graph_node(
            "code:Snakefile",
            Node::SoftwareSourceCode(SoftwareSourceCode {
                name: "Snakefile".to_string(),
                programming_language: "snakemake".to_string(),
                ..Default::default()
            }),
        ),
        graph_node(
            "workflow-unit:Snakefile:download",
            Node::Function(Function::new("download".to_string(), Vec::new())),
        ),
        graph_node(
            "code:workflow/scripts/download.py",
            Node::SoftwareSourceCode(SoftwareSourceCode {
                name: "download.py".to_string(),
                path: Some("workflow/scripts/download.py".to_string()),
                programming_language: "python".to_string(),
                ..Default::default()
            }),
        ),
        graph_node(
            "file:data/raw/S1.fastq",
            Node::File(File::new(
                "S1.fastq".to_string(),
                "data/raw/S1.fastq".to_string(),
            )),
        ),
    ];
    let mut edges = vec![
        GraphEdge::new(
            "code:Snakefile".to_string(),
            "workflow-unit:Snakefile:download".to_string(),
            GraphEdgeKind::Declares,
        ),
        GraphEdge::new(
            "workflow-unit:Snakefile:download".to_string(),
            "code:Snakefile".to_string(),
            GraphEdgeKind::PartOf,
        ),
        GraphEdge::new(
            "code:workflow/scripts/download.py".to_string(),
            "workflow-unit:Snakefile:download".to_string(),
            GraphEdgeKind::UsedBy,
        ),
        GraphEdge::new(
            "code:workflow/scripts/download.py".to_string(),
            "file:data/raw/S1.fastq".to_string(),
            GraphEdgeKind::Generated,
        ),
        GraphEdge::new(
            "workflow-unit:Snakefile:download".to_string(),
            "file:data/raw/S1.fastq".to_string(),
            GraphEdgeKind::Generated,
        ),
    ];

    if include_extra_output {
        nodes.push(graph_node(
            "file:logs/download.log",
            Node::File(File::new(
                "download.log".to_string(),
                "logs/download.log".to_string(),
            )),
        ));
        edges.push(GraphEdge::new(
            "code:workflow/scripts/download.py".to_string(),
            "file:logs/download.log".to_string(),
            GraphEdgeKind::Generated,
        ));
    }

    Graph::new("test:workflow-script-io".to_string(), nodes, edges)
}

fn converted_document_graph() -> Graph {
    Graph::new(
        "test:converted-document".to_string(),
        vec![
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "file:report.html",
                Node::File(File::new(
                    "report.html".to_string(),
                    "report.html".to_string(),
                )),
            ),
            graph_node(
                "node:report.html#art_",
                Node::Article(Article::new(Vec::new())),
            ),
        ],
        vec![
            GraphEdge::new(
                "code:analysis.py".to_string(),
                "file:report.html".to_string(),
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "file:report.html".to_string(),
                "node:report.html#art_".to_string(),
                GraphEdgeKind::ConvertedInto,
            ),
        ],
    )
}

fn converted_document_containment_graph() -> Graph {
    Graph::new(
        "test:converted-document-containment".to_string(),
        vec![
            graph_node(
                "dir:.",
                Node::Directory(Directory::new("workspace".to_string(), ".".to_string())),
            ),
            graph_node(
                "dir:docs",
                Node::Directory(Directory::new("docs".to_string(), "docs".to_string())),
            ),
            graph_node(
                "file:docs/notebook.json",
                Node::File(File::new(
                    "notebook.json".to_string(),
                    "docs/notebook.json".to_string(),
                )),
            ),
            graph_node(
                "node:docs/notebook.json#art_",
                Node::Article(Article::new(Vec::new())),
            ),
            graph_node(
                "node:docs/notebook.json#cdc_setup",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "setup".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "file:data.csv",
                Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
            ),
        ],
        vec![
            GraphEdge::new(
                "dir:docs".to_string(),
                "dir:.".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "file:docs/notebook.json".to_string(),
                "dir:docs".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "node:docs/notebook.json#art_".to_string(),
                "dir:docs".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "node:docs/notebook.json#cdc_setup".to_string(),
                "node:docs/notebook.json#art_".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "file:data.csv".to_string(),
                "node:docs/notebook.json#cdc_setup".to_string(),
                GraphEdgeKind::ReadBy,
            ),
        ],
    )
}

fn structure_only_graph() -> Graph {
    Graph::new(
        "test:structure".to_string(),
        vec![
            graph_node(
                "node:document#article",
                Node::Article(Article::new(Vec::new())),
            ),
            graph_node("node:document#figure", Node::String("figure".to_string())),
        ],
        vec![GraphEdge::new(
            "node:document#figure".to_string(),
            "node:document#article".to_string(),
            GraphEdgeKind::PartOf,
        )],
    )
}

fn document_flow_seed_graph() -> Graph {
    Graph::new(
        "test:document-flow-seeds".to_string(),
        vec![
            graph_node(
                "file:ai-panel.png",
                Node::File(File::new(
                    "ai-panel.png".to_string(),
                    "ai-panel.png".to_string(),
                )),
            ),
            graph_node(
                "node:document#article",
                Node::Article(Article::new(Vec::new())),
            ),
            graph_node(
                "node:document#figure",
                Node::Figure(Figure::new(Vec::new())),
            ),
            graph_node(
                "node:document#figure-code",
                Node::CodeChunk(CodeChunk::new("plot()".into())),
            ),
            graph_node(
                "node:document#figure-image",
                Node::Figure(Figure::new(Vec::new())),
            ),
            graph_node(
                "node:document#nested-table",
                Node::Table(Table::new(Vec::new())),
            ),
            graph_node(
                "node:document#setup",
                Node::CodeChunk(CodeChunk::new("setup()".into())),
            ),
        ],
        vec![
            GraphEdge::new(
                "file:ai-panel.png".to_string(),
                "node:document#figure-image".to_string(),
                GraphEdgeKind::LinkedBy,
            ),
            GraphEdge::new(
                "node:document#figure-code".to_string(),
                "node:document#figure".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "node:document#figure-image".to_string(),
                "node:document#figure".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "node:document#nested-table".to_string(),
                "node:document#figure-image".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "node:document#setup".to_string(),
                "node:document#article".to_string(),
                GraphEdgeKind::PartOf,
            ),
            GraphEdge::new(
                "node:document#figure".to_string(),
                "node:document#article".to_string(),
                GraphEdgeKind::PartOf,
            ),
        ],
    )
}

fn workspace_datatable_seed_graph() -> Graph {
    Graph::new(
        "test:workspace-datatable-flow-seeds".to_string(),
        vec![
            graph_node(
                "datatable:samplesheet.csv",
                Node::Datatable(Datatable::new(Vec::new())),
            ),
            graph_node(
                "file:data.csv",
                Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
            ),
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
        ],
        vec![GraphEdge::new(
            "file:data.csv".to_string(),
            "code:analysis.py".to_string(),
            GraphEdgeKind::ReadBy,
        )],
    )
}

fn collapsed_derivation_graph() -> Graph {
    Graph::new(
        "test:collapsed-derivation".to_string(),
        vec![
            graph_node(
                "datatable:data.csv",
                Node::Datatable(Datatable::new(Vec::new())),
            ),
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "r".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "symbol:analysis.py:r:df",
                Node::Variable(Variable::new("df".to_string())),
            ),
        ],
        vec![
            GraphEdge::new(
                "datatable:data.csv".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "datatable:data.csv".to_string(),
                "symbol:analysis.py:r:df".to_string(),
                GraphEdgeKind::DerivedInto,
            ),
            GraphEdge::new(
                "symbol:analysis.py:r:df".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::PartOf,
            ),
        ],
    )
}

fn collapsed_derivation_only_graph() -> Graph {
    Graph::new(
        "test:collapsed-derivation-only".to_string(),
        vec![
            graph_node(
                "datatable:data.csv",
                Node::Datatable(Datatable::new(Vec::new())),
            ),
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "r".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "symbol:analysis.py:r:df",
                Node::Variable(Variable::new("df".to_string())),
            ),
        ],
        vec![
            GraphEdge::new(
                "datatable:data.csv".to_string(),
                "symbol:analysis.py:r:df".to_string(),
                GraphEdgeKind::DerivedInto,
            ),
            GraphEdge::new(
                "symbol:analysis.py:r:df".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::PartOf,
            ),
        ],
    )
}

fn collapsed_generation_and_derivation_graph() -> Graph {
    Graph::new(
        "test:collapsed-generation-and-derivation".to_string(),
        vec![
            graph_node(
                "code:setup.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "setup.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "code:plot.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "plot.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "symbol:plot.py:python:summaries",
                Node::Variable(Variable::new("summaries".to_string())),
            ),
        ],
        vec![
            GraphEdge::new(
                "code:setup.py".to_string(),
                "symbol:plot.py:python:summaries".to_string(),
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "code:setup.py".to_string(),
                "symbol:plot.py:python:summaries".to_string(),
                GraphEdgeKind::DerivedInto,
            ),
            GraphEdge::new(
                "symbol:plot.py:python:summaries".to_string(),
                "code:plot.py".to_string(),
                GraphEdgeKind::PartOf,
            ),
        ],
    )
}

fn duplicate_edge_graph() -> Graph {
    Graph::new(
        "test:duplicate".to_string(),
        vec![
            graph_node(
                "file:data.csv",
                Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
            ),
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
        ],
        vec![
            GraphEdge::new(
                "file:data.csv".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            low_confidence_edge("file:data.csv", "code:analysis.py", GraphEdgeKind::ReadBy),
        ],
    )
}

fn environment_graph() -> Graph {
    Graph::new(
        "test:environment".to_string(),
        vec![
            graph_node(
                "file:pyproject.toml",
                Node::File(File::new(
                    "pyproject.toml".to_string(),
                    "pyproject.toml".to_string(),
                )),
            ),
            graph_node(
                "file:uv.lock",
                Node::File(File::new("uv.lock".to_string(), "uv.lock".to_string())),
            ),
            graph_node(
                "environment:python:pyproject.toml",
                Node::SoftwareApplication(SoftwareApplication::new(
                    "Python environment declared by pyproject.toml".to_string(),
                )),
            ),
            graph_node(
                "package:pypi/pandas",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "pandas".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
        ],
        vec![
            GraphEdge::new(
                "file:pyproject.toml".to_string(),
                "environment:python:pyproject.toml".to_string(),
                GraphEdgeKind::Declares,
            ),
            GraphEdge::new(
                "file:uv.lock".to_string(),
                "environment:python:pyproject.toml".to_string(),
                GraphEdgeKind::Pins,
            ),
            GraphEdge::new(
                "package:pypi/pandas".to_string(),
                "environment:python:pyproject.toml".to_string(),
                GraphEdgeKind::RequiredBy,
            ),
        ],
    )
}

fn detail_graph() -> Graph {
    Graph::new(
        "test:detail".to_string(),
        vec![
            graph_node(
                "file:data.csv",
                Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
            ),
            graph_node(
                "file:plot.png",
                Node::File(File::new("plot.png".to_string(), "plot.png".to_string())),
            ),
            graph_node(
                "code:analysis.py",
                Node::SoftwareSourceCode(SoftwareSourceCode {
                    name: "analysis.py".to_string(),
                    programming_language: "python".to_string(),
                    ..Default::default()
                }),
            ),
            graph_node(
                "symbol:analysis.py:python:df",
                Node::Variable(Variable::new("df".to_string())),
            ),
            graph_node(
                "function:analysis.py:python:read_csv",
                Node::Function(Function::new("read_csv".to_string(), Vec::new())),
            ),
            graph_node(
                "column:analysis.py:data.csv:count",
                Node::DatatableColumn(DatatableColumn::new("count".to_string(), Vec::new())),
            ),
        ],
        vec![
            GraphEdge::new(
                "file:data.csv".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::ReadBy,
            ),
            GraphEdge::new(
                "code:analysis.py".to_string(),
                "file:plot.png".to_string(),
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "code:analysis.py".to_string(),
                "symbol:analysis.py:python:df".to_string(),
                GraphEdgeKind::Generated,
            ),
            GraphEdge::new(
                "symbol:analysis.py:python:df".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::UsedBy,
            ),
            GraphEdge::new(
                "function:analysis.py:python:read_csv".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::CalledBy,
            ),
            GraphEdge::new(
                "column:analysis.py:data.csv:count".to_string(),
                "code:analysis.py".to_string(),
                GraphEdgeKind::UsedBy,
            ),
            GraphEdge::new(
                "column:analysis.py:data.csv:count".to_string(),
                "file:plot.png".to_string(),
                GraphEdgeKind::DerivedInto,
            ),
        ],
    )
}

fn graph_node(id: &str, node: Node) -> GraphNode {
    GraphNode::new(id.to_string(), Box::new(node))
}

fn reference(title: &str) -> Reference {
    Reference {
        title: Some(vec![stencila_schema::Inline::Text(
            stencila_schema::Text::new(title.into()),
        )]),
        ..Default::default()
    }
}

fn low_confidence_edge(source: &str, target: &str, kind: GraphEdgeKind) -> GraphEdge {
    let mut edge = GraphEdge::new(source.to_string(), target.to_string(), kind);
    edge.options.evidence = Some(vec![GraphEvidence {
        kind: GraphEvidenceKind::Inferred,
        confidence: Some(GraphEvidenceConfidence::Low),
        ..Default::default()
    }]);
    edge
}

#[test]
fn edge_labels_are_readable() -> Result<()> {
    assert_eq!(edge_label(GraphEdgeKind::DerivedInto), "Derived Into");
    Ok(())
}
