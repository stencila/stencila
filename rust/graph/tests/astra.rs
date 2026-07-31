use std::{collections::BTreeSet, fs};

use eyre::{Result, eyre};
use stencila_graph::{WorkspaceOptions, graph_from_path};
use stencila_schema::{
    GraphEdgeKind, GraphEvidenceKind, Node, PropertyValueOrString, StringOrNumber,
};
use tempfile::tempdir;

fn options() -> WorkspaceOptions {
    WorkspaceOptions {
        decode: false,
        analyze_environment: false,
        include_c2pa: false,
        source_metadata: false,
        git_file_authors: false,
        ..Default::default()
    }
}

#[tokio::test]
async fn projects_astra_contracts_without_executing_recipes() -> Result<()> {
    let dir = tempdir()?;
    fs::create_dir(dir.path().join("data"))?;
    fs::create_dir(dir.path().join("child"))?;
    fs::write(dir.path().join("data/catalog.csv"), "x\n1\n")?;
    fs::write(dir.path().join("Containerfile"), "FROM python:3.12\n")?;
    fs::write(
        dir.path().join("child/astra.yaml"),
        r#"
version: "1.0"
name: Child fit
inputs:
  - id: inherited
    from: ../catalog
decisions:
  method:
    from: ../method
outputs:
  - id: fitted
    type: table
    inputs: [inherited]
    decisions: [method]
    recipe:
      command: python fit.py --out {output}
"#,
    )?;
    fs::write(
        dir.path().join("astra.yaml"),
        r#"
version: "1.0.2"
name: Root analysis
container: Containerfile
inputs:
  - id: catalog
    type: data
    source: data/catalog.csv
  - id: registry
    type: data
    source: https://example.org/reference.csv
decisions:
  method:
    label: Fit method
    default: robust
    options:
      robust: { label: Robust }
      ordinary: { label: Ordinary }
outputs:
  - id: prepared
    type: data
    inputs: [catalog, registry]
    decisions: [method]
    when: [method.robust]
    recipe:
      command: touch SHOULD_NOT_EXIST
      resources: { cpus: 2 }
analyses:
  fit:
    path: child
"#,
    )?;

    let graph = graph_from_path(dir.path(), Some(options())).await?;

    assert!(
        !dir.path().join("SHOULD_NOT_EXIST").exists(),
        "ASTRA recipes must remain declarations"
    );
    assert!(graph.nodes.iter().any(|node| {
        node.id.starts_with("astra-analysis:")
            && matches!(node.node.as_ref(), Node::CreativeWork(work) if work.work_type == Some(stencila_schema::CreativeWorkType::Workflow))
    }));
    assert!(graph.nodes.iter().any(|node| {
        node.id.starts_with("astra-decision:")
            && matches!(node.node.as_ref(), Node::Parameter(parameter) if parameter.options.validator.is_some())
    }));
    assert!(graph.nodes.iter().any(|node| {
        node.id.starts_with("workflow-unit:") && matches!(node.node.as_ref(), Node::Function(..))
    }));
    assert!(graph.edges.iter().any(|edge| {
        edge.source == "datatable:data/catalog.csv"
            && edge.target.starts_with("workflow-unit:")
            && edge.kind == stencila_schema::GraphEdgeKind::ReadBy
    }));
    assert!(graph.edges.iter().any(|edge| {
        edge.source == "resource:https%3A//example.org/reference.csv"
            && edge.target.starts_with("workflow-unit:")
            && edge.kind == stencila_schema::GraphEdgeKind::ReceivedBy
    }));
    assert!(
        graph
            .edges
            .iter()
            .all(|edge| edge.options.actions.is_none()),
        "contract projection must not add GraphAction values"
    );

    let astra_evidence = graph
        .edges
        .iter()
        .flat_map(|edge| edge.options.evidence.iter().flatten())
        .filter(|evidence| {
            evidence
                .options
                .details
                .as_ref()
                .and_then(|details| details.get("detector"))
                == Some(&stencila_schema::Primitive::String(
                    "stencila-astra-contract".to_string(),
                ))
        })
        .collect::<Vec<_>>();
    if astra_evidence.is_empty() {
        return Err(eyre!("expected ASTRA declaration evidence"));
    }
    assert!(
        astra_evidence
            .iter()
            .all(|evidence| evidence.kind == GraphEvidenceKind::Declared)
    );
    for edge in &graph.edges {
        let evidence = edge.options.evidence.as_deref().unwrap_or_default();
        if evidence.iter().any(|item| astra_evidence.contains(&item)) {
            assert_eq!(evidence.len(), 1, "ASTRA edges have one evidence basis");
        }
    }

    Ok(())
}

#[tokio::test]
async fn attributes_direct_recipe_scripts_and_canonicalizes_doi_inputs() -> Result<()> {
    let dir = tempdir()?;
    let doi = "10.6073/pasta/abc50eed9138b75f54eaada0841b9b86";
    fs::write(
        dir.path().join("download.py"),
        format!(
            "from urllib.request import urlopen\nurlopen(\"https://doi.org/{doi}\")\nopen(\"result.csv\", \"w\").write(\"value\\n1\\n\")\n"
        ),
    )?;
    fs::write(dir.path().join("result.csv"), "value\n1\n")?;
    fs::write(dir.path().join("compound.py"), "print('not attributed')\n")?;
    fs::write(
        dir.path().join("Snakefile"),
        r#"rule result:
    output: "result.csv"
    shell: "python download.py"
"#,
    )?;
    fs::write(
        dir.path().join("astra.yaml"),
        format!(
            r#"version: "1.0"
inputs:
  - id: source
    type: data
    source: https://doi.org/{doi}
outputs:
  - id: result
    type: data
    target: result.csv
    inputs: [source]
    recipe:
      command: python download.py
  - id: compound
    type: data
    recipe:
      command: python compound.py && echo done
"#
        ),
    )?;

    let graph = graph_from_path(dir.path(), Some(options())).await?;
    let script_id = "code:download.py";
    let unit_id = "workflow-unit:astra.yaml%23root:result";
    let output_id = "astra-output:astra.yaml:root:result";
    let logical_output = graph
        .nodes
        .iter()
        .find(|node| node.id == output_id)
        .and_then(|node| match node.node.as_ref() {
            Node::CreativeWork(work) => Some(work),
            _ => None,
        })
        .ok_or_else(|| eyre!("expected logical ASTRA output"))?;
    assert_eq!(logical_output.options.path.as_deref(), Some("result.csv"));

    for (target, kind) in [
        (unit_id, GraphEdgeKind::UsedBy),
        (output_id, GraphEdgeKind::Generated),
    ] {
        let edge = graph
            .edges
            .iter()
            .find(|edge| edge.source == script_id && edge.target == target && edge.kind == kind)
            .ok_or_else(|| eyre!("expected ASTRA recipe script attribution"))?;
        let evidence = edge
            .options
            .evidence
            .as_deref()
            .and_then(|evidence| evidence.first())
            .ok_or_else(|| eyre!("expected recipe script evidence"))?;
        assert_eq!(evidence.kind, GraphEvidenceKind::Declared);
        assert_eq!(
            evidence
                .options
                .details
                .as_ref()
                .and_then(|details| details.get("fieldPath")),
            Some(&stencila_schema::Primitive::String(
                "outputs[0].recipe.command".to_string()
            ))
        );
        assert_eq!(
            evidence
                .code_location
                .as_ref()
                .and_then(|location| location.start_line),
            Some(11)
        );
    }

    let concrete_id = "datatable:result.csv";
    let concrete_edge = graph
        .edges
        .iter()
        .find(|edge| {
            edge.source == script_id
                && edge.target == concrete_id
                && edge.kind == GraphEdgeKind::Generated
        })
        .ok_or_else(|| eyre!("expected concrete target generation"))?;
    let concrete_evidence = concrete_edge
        .options
        .evidence
        .as_deref()
        .unwrap_or_default();
    assert_eq!(
        concrete_evidence
            .iter()
            .filter(|evidence| evidence.kind == GraphEvidenceKind::Declared)
            .count(),
        2
    );
    assert_eq!(
        concrete_evidence
            .iter()
            .filter(|evidence| evidence.kind == GraphEvidenceKind::StaticAnalysis)
            .count(),
        1
    );
    let detectors = concrete_evidence
        .iter()
        .filter_map(|evidence| {
            evidence
                .options
                .details
                .as_ref()
                .and_then(|details| details.get("detector"))
                .and_then(|detector| match detector {
                    stencila_schema::Primitive::String(detector) => Some(detector.clone()),
                    _ => None,
                })
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        detectors,
        BTreeSet::from([
            "stencila-astra-contract".to_string(),
            "stencila-code-static-analysis".to_string(),
            "stencila-workflow-declaration".to_string(),
        ])
    );
    let astra_target_evidence = concrete_evidence
        .iter()
        .find(|evidence| {
            evidence
                .options
                .details
                .as_ref()
                .and_then(|details| details.get("extension"))
                == Some(&stencila_schema::Primitive::String(
                    "stencila-output-target".to_string(),
                ))
        })
        .ok_or_else(|| eyre!("expected ASTRA output target evidence"))?;
    assert_eq!(
        astra_target_evidence
            .options
            .details
            .as_ref()
            .and_then(|details| details.get("target")),
        Some(&stencila_schema::Primitive::String(
            "result.csv".to_string()
        ))
    );
    assert_eq!(
        astra_target_evidence
            .code_location
            .as_ref()
            .and_then(|location| location.start_line),
        Some(11)
    );
    assert!(graph.edges.iter().any(|edge| {
        edge.source == output_id
            && edge.target == concrete_id
            && edge.kind == GraphEdgeKind::WrittenTo
            && edge
                .options
                .evidence
                .as_deref()
                .and_then(|evidence| evidence.first())
                .and_then(|evidence| evidence.code_location.as_ref())
                .and_then(|location| location.start_line)
                == Some(8)
    }));

    assert!(!graph.edges.iter().any(|edge| {
        edge.source == "code:compound.py"
            && edge.target == "astra-output:astra.yaml:root:compound"
            && edge.kind == GraphEdgeKind::Generated
    }));

    let doi_id = format!("resource:doi%3A{doi}");
    assert_eq!(
        graph.nodes.iter().filter(|node| node.id == doi_id).count(),
        1
    );
    assert!(
        graph
            .nodes
            .iter()
            .all(|node| !node.id.starts_with("resource:https%3A//doi.org/"))
    );
    let doi_work = graph
        .nodes
        .iter()
        .find(|node| node.id == doi_id)
        .and_then(|node| match node.node.as_ref() {
            Node::CreativeWork(work) => Some(work),
            _ => None,
        })
        .ok_or_else(|| eyre!("expected canonical DOI resource"))?;
    assert_eq!(doi_work.doi.as_deref(), Some(doi));
    assert_eq!(
        doi_work.options.url.as_deref(),
        Some(format!("https://doi.org/{doi}").as_str())
    );

    Ok(())
}

#[tokio::test]
async fn reexported_outputs_inherit_metadata_and_targets() -> Result<()> {
    let dir = tempdir()?;
    fs::create_dir(dir.path().join("child"))?;
    fs::write(dir.path().join("child/result.csv"), "value\n1\n")?;
    fs::write(
        dir.path().join("child/astra.yaml"),
        r#"version: "1.0"
outputs:
  - id: result
    label: Child result
    type: table
    description: Materialized by the child analysis.
    target: result.csv
"#,
    )?;
    fs::write(
        dir.path().join("astra.yaml"),
        r#"version: "1.0"
outputs:
  - id: final
    from: child.result
analyses:
  child:
    path: child
"#,
    )?;

    let graph = graph_from_path(dir.path(), Some(options())).await?;
    let output_id = "astra-output:astra.yaml:root:final";
    let output = graph
        .nodes
        .iter()
        .find(|node| node.id == output_id)
        .and_then(|node| match node.node.as_ref() {
            Node::CreativeWork(work) => Some(work),
            _ => None,
        })
        .ok_or_else(|| eyre!("expected re-exported output"))?;
    assert_eq!(output.options.name.as_deref(), Some("Child result"));
    assert_eq!(
        output.options.description.as_deref(),
        Some("Materialized by the child analysis.")
    );
    assert_eq!(output.options.path.as_deref(), Some("result.csv"));
    assert_eq!(
        output.work_type,
        Some(stencila_schema::CreativeWorkType::Datatable)
    );

    let materialization = graph
        .edges
        .iter()
        .find(|edge| {
            edge.source == output_id
                && edge.target == "datatable:child/result.csv"
                && edge.kind == GraphEdgeKind::WrittenTo
        })
        .ok_or_else(|| eyre!("expected inherited output target materialization"))?;
    let evidence = materialization
        .options
        .evidence
        .as_deref()
        .and_then(|evidence| evidence.first())
        .ok_or_else(|| eyre!("expected inherited target evidence"))?;
    assert_eq!(
        evidence
            .options
            .details
            .as_ref()
            .and_then(|details| details.get("fieldPath")),
        Some(&stencila_schema::Primitive::String(
            "outputs[0].from".to_string()
        ))
    );

    Ok(())
}

#[tokio::test]
async fn skips_invalid_astra_roots_permissively_and_fails_strictly() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("astra.yaml"),
        r#"
version: "2.0"
outputs:
  - id: result
    type: metric
"#,
    )?;

    let permissive = graph_from_path(dir.path(), Some(options())).await?;
    assert!(
        permissive
            .nodes
            .iter()
            .any(|node| node.id == "file:astra.yaml")
    );
    assert!(
        permissive
            .nodes
            .iter()
            .all(|node| !node.id.starts_with("astra-analysis:"))
    );

    let error = graph_from_path(
        dir.path(),
        Some(WorkspaceOptions {
            fail_on_astra_error: true,
            ..options()
        }),
    )
    .await
    .expect_err("strict ASTRA analysis should fail");
    let message = error.to_string();
    assert!(message.contains("astra.yaml"));
    assert!(message.contains("version"));

    Ok(())
}

#[tokio::test]
async fn rejects_astra_output_cycles_and_outside_child_paths() -> Result<()> {
    let cycles = tempdir()?;
    fs::write(
        cycles.path().join("astra.yaml"),
        r#"
outputs:
  - id: first
    type: data
    inputs: [second]
  - id: second
    type: data
    inputs: [first]
"#,
    )?;
    let cycle_error = graph_from_path(
        cycles.path(),
        Some(WorkspaceOptions {
            fail_on_astra_error: true,
            ..options()
        }),
    )
    .await
    .expect_err("output cycles should fail");
    assert!(cycle_error.to_string().contains("cycle"));

    let outside = tempdir()?;
    fs::write(
        outside.path().join("astra.yaml"),
        "analyses:\n  child:\n    path: ../../outside\n",
    )?;
    let outside_error = graph_from_path(
        outside.path(),
        Some(WorkspaceOptions {
            fail_on_astra_error: true,
            ..options()
        }),
    )
    .await
    .expect_err("outside child paths should fail");
    assert!(outside_error.to_string().contains("analyses.child.path"));

    Ok(())
}

#[tokio::test]
async fn rejects_recursive_external_astra_manifests_without_a_root() -> Result<()> {
    let dir = tempdir()?;
    fs::create_dir(dir.path().join("a"))?;
    fs::create_dir(dir.path().join("b"))?;
    fs::write(
        dir.path().join("a/astra.yaml"),
        "analyses:\n  b:\n    path: ../b\n",
    )?;
    fs::write(
        dir.path().join("b/astra.yaml"),
        "analyses:\n  a:\n    path: ../a\n",
    )?;

    let error = graph_from_path(
        dir.path(),
        Some(WorkspaceOptions {
            fail_on_astra_error: true,
            ..options()
        }),
    )
    .await
    .expect_err("recursive manifests should fail even without an unreferenced root");
    assert!(error.to_string().contains("recursive"));
    assert!(error.to_string().contains("analyses"));
    Ok(())
}

#[tokio::test]
async fn rejects_undeclared_recipe_placeholders_and_invalid_conditions() -> Result<()> {
    for (manifest, expected) in [
        (
            r#"
version: "1.0"
inputs:
  - id: data
    type: data
outputs:
  - id: result
    type: data
    inputs: [data]
    recipe:
      command: tool {inputs.missing} {output}
"#,
            "placeholder",
        ),
        (
            r#"
version: "1.0"
outputs:
  - id: result
    type: data
    when: [missing.enabled]
"#,
            "undeclared decision",
        ),
        (
            r#"
version: "1.0"
outputs:
  - id: result
    type: data
    target: ../../outside.csv
"#,
            "output target must stay within the workspace",
        ),
    ] {
        let dir = tempdir()?;
        fs::write(dir.path().join("astra.yaml"), manifest)?;
        let error = graph_from_path(
            dir.path(),
            Some(WorkspaceOptions {
                fail_on_astra_error: true,
                ..options()
            }),
        )
        .await
        .expect_err("strict ASTRA analysis should reject invalid semantics");
        assert!(error.to_string().contains(expected));
    }
    Ok(())
}

#[tokio::test]
async fn reuses_remote_inputs_and_preserves_transfer_semantics() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("astra.yaml"),
        r#"
version: "1.0"
inputs:
  - id: primary
    label: Primary
    type: data
    source: ftps://example.org/shared.csv
  - id: backup
    label: Backup
    type: data
    source: ftps://example.org/shared.csv
outputs:
  - id: result
    type: data
    inputs: [primary, backup]
    recipe:
      command: tool {inputs} {output}
"#,
    )?;

    let graph = graph_from_path(dir.path(), Some(options())).await?;
    let resource_id = "resource:ftps%3A//example.org/shared.csv";
    assert_eq!(
        graph
            .nodes
            .iter()
            .filter(|node| node.id == resource_id)
            .count(),
        1
    );
    assert!(graph.edges.iter().any(|edge| {
        edge.source == resource_id
            && edge.kind == GraphEdgeKind::ReceivedBy
            && edge.target.starts_with("workflow-unit:")
    }));
    assert!(!graph.edges.iter().any(|edge| {
        edge.source == resource_id
            && edge.kind == GraphEdgeKind::ReadBy
            && edge.target.starts_with("workflow-unit:")
    }));
    Ok(())
}

#[tokio::test]
async fn preserves_unresolved_input_and_analysis_reference_locators() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("astra.yaml"),
        r#"
version: "1.0"
inputs:
  - id: catalog
    type: data
    source: sklearn.datasets.load_iris
  - id: prior
    type: analysis
    ref: analyses/baseline_fit
    ref_version: "1.2"
outputs:
  - id: result
    type: data
    inputs: [catalog, prior]
"#,
    )?;

    let graph = graph_from_path(dir.path(), Some(options())).await?;
    let catalog = graph
        .nodes
        .iter()
        .find(|node| node.id.starts_with("astra-input:") && node.id.ends_with(":catalog"))
        .and_then(|node| match node.node.as_ref() {
            Node::CreativeWork(work) => Some(work),
            _ => None,
        })
        .ok_or_else(|| eyre!("expected ASTRA catalog input"))?;
    assert_eq!(
        catalog.options.identifiers.as_deref(),
        Some(
            [PropertyValueOrString::String(
                "sklearn.datasets.load_iris".to_string()
            )]
            .as_slice()
        )
    );

    let prior = graph
        .nodes
        .iter()
        .find(|node| node.id.starts_with("astra-analysis-ref:") && node.id.ends_with(":prior"))
        .and_then(|node| match node.node.as_ref() {
            Node::CreativeWork(work) => Some(work),
            _ => None,
        })
        .ok_or_else(|| eyre!("expected referenced ASTRA analysis"))?;
    assert_eq!(prior.options.path.as_deref(), Some("analyses/baseline_fit"));
    assert_eq!(
        prior.options.version,
        Some(StringOrNumber::String("1.2".to_string()))
    );
    Ok(())
}

#[tokio::test]
async fn scopes_evidence_locations_and_projects_conditional_decisions() -> Result<()> {
    let dir = tempdir()?;
    fs::write(
        dir.path().join("astra.yaml"),
        r#"version: "1.0"
decisions:
  mode:
    label: Root mode
    options:
      enabled: { label: Enabled }
analyses:
  child:
    decisions:
      mode:
        label: Child mode
        options:
          enabled: { label: Enabled }
    outputs:
      - id: result
        type: data
        when: [mode.enabled]
"#,
    )?;

    let graph = graph_from_path(dir.path(), Some(options())).await?;
    let decision_id = "astra-decision:astra.yaml:root.child:mode";
    let unit_id = "workflow-unit:astra.yaml%23root.child:result";
    let containment = graph
        .edges
        .iter()
        .find(|edge| {
            edge.source == decision_id
                && edge.target == "astra-analysis:astra.yaml:root.child"
                && edge.kind == GraphEdgeKind::PartOf
        })
        .ok_or_else(|| eyre!("expected child decision containment"))?;
    let location = containment
        .options
        .evidence
        .as_deref()
        .and_then(|evidence| evidence.first())
        .and_then(|evidence| evidence.code_location.as_ref())
        .ok_or_else(|| eyre!("expected scoped decision evidence location"))?;
    assert_eq!(location.start_line, Some(9));
    assert!(graph.edges.iter().any(|edge| {
        edge.source == decision_id
            && edge.target == unit_id
            && edge.kind == GraphEdgeKind::Configures
    }));
    Ok(())
}
