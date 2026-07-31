//! Ingestion of opt-in Python runtime dependency evidence.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use stencila_schema::{
    CodeLocation, CreativeWork, File, GraphEdgeKind, GraphEvidence, GraphEvidenceConfidence,
    GraphEvidenceKind, Node, NodeId, Object, Primitive, SoftwareSourceCode,
};

use crate::{
    GraphBuilder,
    code::is_python_stdlib,
    ids::{LocalGraphId, WorkspaceRelPath},
    package::{package_id, package_node},
    reference::{bare_doi, doi_url},
};

/// Whether graph construction should ingest previously cached runtime evidence.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RuntimeEvidenceMode {
    /// Do not read runtime evidence.
    #[default]
    None,

    /// Read valid evidence from `.stencila/cache/runtime`.
    Cached,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeTrace {
    version: u8,
    identity: String,
    code_digest: String,
    events: Vec<RuntimeEvent>,
}

#[derive(Debug, Deserialize)]
struct RuntimeEvent {
    operation: RuntimeOperation,
    resource: String,
    location: RuntimeLocation,
    count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RuntimeOperation {
    FileRead,
    FileWrite,
    RemoteReceive,
    RemoteSend,
    Import,
}

#[derive(Debug, Deserialize)]
struct RuntimeLocation {
    source: String,
    line: u64,
}

pub(crate) fn add_cached_runtime_evidence(builder: &mut GraphBuilder, graph_root: &Path) {
    let workspace_root = graph_root
        .ancestors()
        .find(|ancestor| ancestor.join(".stencila").is_dir())
        .unwrap_or(graph_root);
    let cache_dir = workspace_root.join(".stencila/cache/runtime");
    let Ok(entries) = fs::read_dir(cache_dir) else {
        return;
    };

    let mut paths = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Ok(trace) = serde_json::from_slice::<RuntimeTrace>(&bytes) else {
            continue;
        };
        if trace.version != 1 {
            continue;
        }
        let Some(consumer) = runtime_consumer(builder, graph_root, workspace_root, &trace.identity)
        else {
            continue;
        };
        if !runtime_digest_matches(builder, workspace_root, &consumer, &trace) {
            continue;
        }

        for event in trace.events {
            add_event(builder, graph_root, workspace_root, &consumer, event);
        }
    }
}

fn runtime_digest_matches(
    builder: &GraphBuilder,
    root: &Path,
    consumer: &str,
    trace: &RuntimeTrace,
) -> bool {
    if let Some(path) = trace.identity.strip_prefix("script:") {
        let path = path
            .strip_prefix("workspace:")
            .map_or_else(|| PathBuf::from(path), |path| root.join(path));
        let path = path.as_path();
        if WorkspaceRelPath::from_workspace_path(root, path).is_err() {
            return false;
        }
        let Ok(code) = fs::read(path) else {
            return false;
        };
        let digest = Sha256::digest(code)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        return digest == trace.code_digest;
    }

    builder.runtime_code_digest_matches(consumer, &trace.code_digest)
}

fn runtime_consumer(
    builder: &GraphBuilder,
    graph_root: &Path,
    workspace_root: &Path,
    identity: &str,
) -> Option<String> {
    if let Some(path) = identity.strip_prefix("script:") {
        let path = path
            .strip_prefix("workspace:")
            .map_or_else(|| PathBuf::from(path), |path| workspace_root.join(path));
        let path = path.as_path();
        let rel = WorkspaceRelPath::from_workspace_path(graph_root, path).ok()?;
        let id = LocalGraphId::code(rel.as_str());
        return builder.contains_node(&id).then_some(id);
    }

    if let Some(identity) = identity.strip_prefix("document:")
        && let Some((scope, node_id)) = identity.rsplit_once('#')
        && let Ok(node_id) = node_id.parse::<NodeId>()
    {
        let graph_id = LocalGraphId::document_node(scope, &node_id);
        return builder.contains_node(&graph_id).then_some(graph_id);
    }

    builder.unique_graph_id_for_schema_node_id(identity)
}

fn add_event(
    builder: &mut GraphBuilder,
    graph_root: &Path,
    workspace_root: &Path,
    consumer: &str,
    event: RuntimeEvent,
) {
    let evidence = runtime_evidence(&event);
    match event.operation {
        RuntimeOperation::FileRead | RuntimeOperation::FileWrite => {
            let Some(resource) =
                file_resource(builder, graph_root, workspace_root, &event.resource)
            else {
                return;
            };
            let kind = if matches!(event.operation, RuntimeOperation::FileRead) {
                GraphEdgeKind::ReadBy
            } else {
                GraphEdgeKind::Generated
            };
            let (source, target) = if matches!(event.operation, RuntimeOperation::FileRead) {
                (resource, consumer.to_string())
            } else {
                (consumer.to_string(), resource)
            };
            builder.add_edge_with_evidence(source, target, kind, [evidence]);
        }
        RuntimeOperation::RemoteReceive | RuntimeOperation::RemoteSend => {
            if event.resource.is_empty() {
                return;
            }
            let mut node = CreativeWork::new();
            let resource = if let Some(doi) = bare_doi(&event.resource) {
                let resource = LocalGraphId::resource(&format!("doi:{doi}"));
                node.doi = Some(doi.to_string());
                node.options.url = Some(doi_url(doi));
                resource
            } else {
                let resource = LocalGraphId::resource(&event.resource);
                node.options.url = Some(event.resource);
                resource
            };
            builder.add_schema_node(resource.clone(), Node::CreativeWork(node));
            let (source, target, kind) =
                if matches!(event.operation, RuntimeOperation::RemoteReceive) {
                    (resource, consumer.to_string(), GraphEdgeKind::ReceivedBy)
                } else {
                    (consumer.to_string(), resource, GraphEdgeKind::SentTo)
                };
            builder.add_edge_with_evidence(source, target, kind, [evidence]);
        }
        RuntimeOperation::Import => {
            let (name, module_path) = event
                .resource
                .split_once('|')
                .map_or((event.resource.as_str(), None), |(name, path)| {
                    (name, Some(path))
                });
            let resource = if let Some(resource) = module_path
                .and_then(|path| local_module_resource(builder, graph_root, workspace_root, path))
            {
                resource
            } else {
                if is_python_stdlib(name) {
                    return;
                }
                let id = package_id("pypi", name, &[]);
                builder.add_schema_node(
                    id.clone(),
                    Node::SoftwareSourceCode(package_node("pypi", name, &[])),
                );
                id
            };
            builder.add_edge_with_evidence(
                resource,
                consumer.to_string(),
                GraphEdgeKind::ImportedBy,
                [evidence],
            );
        }
    }
}

fn file_resource(
    builder: &mut GraphBuilder,
    graph_root: &Path,
    workspace_root: &Path,
    path: &str,
) -> Option<String> {
    let absolute = path
        .strip_prefix("workspace:")
        .map_or_else(|| PathBuf::from(path), |path| workspace_root.join(path));
    let rel = WorkspaceRelPath::from_workspace_path(graph_root, &absolute).ok()?;
    if is_python_environment_path(workspace_root, &absolute) {
        return None;
    }
    if let Some(id) = builder.graph_id_for_file_path(rel.as_str()) {
        return Some(id);
    }
    let candidates = [
        LocalGraphId::file(&rel),
        LocalGraphId::datatable(&rel),
        LocalGraphId::image(&rel),
        LocalGraphId::audio(&rel),
        LocalGraphId::video(&rel),
        LocalGraphId::code(rel.as_str()),
    ];
    if let Some(id) = candidates
        .into_iter()
        .find(|candidate| builder.contains_node(candidate))
    {
        return Some(id);
    }

    let name = absolute
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string();
    let id = LocalGraphId::file(&rel);
    let mut file = File::new(name, rel.as_str().to_string());
    file.id = Some(id.clone());
    builder.add_schema_node(id.clone(), Node::File(file));
    Some(id)
}

fn local_module_resource(
    builder: &mut GraphBuilder,
    graph_root: &Path,
    workspace_root: &Path,
    path: &str,
) -> Option<String> {
    let path = path
        .strip_prefix("workspace:")
        .map_or_else(|| PathBuf::from(path), |path| workspace_root.join(path));
    let rel = WorkspaceRelPath::from_workspace_path(graph_root, &path).ok()?;
    if is_python_environment_path(workspace_root, &path) {
        return None;
    }
    let id = LocalGraphId::code(rel.as_str());
    if builder.contains_node(&id) {
        return Some(id);
    }
    if !path.is_file() {
        return None;
    }

    let name = path.file_name().and_then(|name| name.to_str())?.to_string();
    let mut node = SoftwareSourceCode::new(name, "Python".to_string());
    node.id = Some(id.clone());
    node.path = Some(rel.as_str().to_string());
    builder.add_schema_node(id.clone(), Node::SoftwareSourceCode(node));
    Some(id)
}

/// Whether a path is part of a Python environment nested in the workspace.
///
/// Checking both installation directory names and `pyvenv.cfg` supports legacy
/// trace caches without assuming that virtual environments are named `.venv`.
fn is_python_environment_path(workspace_root: &Path, path: &Path) -> bool {
    if path.components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some("site-packages" | "dist-packages")
        )
    }) {
        return true;
    }

    path.ancestors()
        .take_while(|ancestor| ancestor.starts_with(workspace_root))
        .any(|ancestor| ancestor.join("pyvenv.cfg").is_file())
}

fn runtime_evidence(event: &RuntimeEvent) -> GraphEvidence {
    let mut evidence = GraphEvidence::new(GraphEvidenceKind::RuntimeAnalysis);
    evidence.confidence = Some(GraphEvidenceConfidence::Certain);
    if !event.location.source.is_empty() {
        evidence.code_location = Some(CodeLocation {
            source: Some(event.location.source.clone()),
            start_line: Some(event.location.line),
            ..Default::default()
        });
    }
    evidence.options.details = Some(Object::from([(
        "observations",
        Primitive::UnsignedInteger(event.count),
    )]));
    evidence
}

#[cfg(test)]
mod tests {
    use eyre::Result;
    use stencila_schema::{Article, Block, CodeChunk, GraphEvidenceKind, SoftwareSourceCode};

    use super::*;
    use crate::evidence;

    #[test]
    fn cached_runtime_evidence_matches_stabilized_document_identity() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let root = temp_dir.path().canonicalize()?;
        fs::write(root.join("input.txt"), "value")?;

        let chunk = CodeChunk::new("open('input.txt').read()\n".into());
        let code_digest = Sha256::digest(chunk.code.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let document = Node::Article(Article::new(vec![Block::CodeChunk(chunk)]));
        let mut stable = document.clone();
        stencila_node_stabilize::stabilize(&mut stable);
        let Node::Article(article) = stable else {
            eyre::bail!("expected article")
        };
        let Some(Block::CodeChunk(chunk)) = article.content.first() else {
            eyre::bail!("expected code chunk")
        };
        let node_id = chunk.node_id();

        let cache_dir = root.join(".stencila/cache/runtime");
        fs::create_dir_all(&cache_dir)?;
        fs::write(
            cache_dir.join("trace.json"),
            serde_json::json!({
                "version": 1,
                "identity": format!("document:report.smd#{node_id}"),
                "codeDigest": code_digest,
                "events": [{
                    "operation": "file_read",
                    "resource": "workspace:input.txt",
                    "location": {"source": "report.smd", "line": 0},
                    "count": 1
                }],
                "diagnostics": []
            })
            .to_string(),
        )?;

        let analysis = crate::graph_from_node_with_runtime_evidence(
            "test",
            "report.smd",
            &document,
            &root,
            RuntimeEvidenceMode::Cached,
        )?;
        let consumer = LocalGraphId::document_node("report.smd", &node_id);
        assert!(analysis.graph.edges.iter().any(|edge| {
            edge.source == "file:input.txt"
                && edge.target == consumer
                && edge.kind == GraphEdgeKind::ReadBy
        }));

        Ok(())
    }

    #[test]
    fn cached_runtime_evidence_from_parent_workspace_merges_with_static_edges() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let root = temp_dir.path().canonicalize()?;
        let graph_root = root.join("project");
        fs::create_dir(&graph_root)?;
        let script_path = graph_root.join("analysis.py");
        let input_path = graph_root.join("dynamic.csv");
        fs::write(&script_path, "open('dynamic.csv').read()\n")?;
        fs::write(&input_path, "value\n1\n")?;

        let script_rel = WorkspaceRelPath::from_relative_path(Path::new("analysis.py"))?;
        let input_rel = WorkspaceRelPath::from_relative_path(Path::new("dynamic.csv"))?;
        let script_id = LocalGraphId::code(script_rel.as_str());
        let input_id = LocalGraphId::file(&input_rel);
        let doi = "10.6073/pasta/abc50";
        let doi_id = LocalGraphId::resource(&format!("doi:{doi}"));
        let mut builder = GraphBuilder::new("test");
        builder.add_schema_node(
            script_id.clone(),
            Node::SoftwareSourceCode(SoftwareSourceCode::new(
                "analysis.py".to_string(),
                "Python".to_string(),
            )),
        );
        builder.add_schema_node(
            input_id.clone(),
            Node::File(File::new(
                "dynamic.csv".to_string(),
                "dynamic.csv".to_string(),
            )),
        );
        let mut doi_node = CreativeWork::new();
        doi_node.doi = Some(doi.to_string());
        doi_node.options.url = Some(doi_url(doi));
        builder.add_schema_node(&doi_id, Node::CreativeWork(doi_node));
        builder.add_edge_with_evidence(
            &input_id,
            &script_id,
            GraphEdgeKind::ReadBy,
            [evidence::static_analysis()],
        );
        builder.add_edge_with_evidence(
            &doi_id,
            &script_id,
            GraphEdgeKind::ReceivedBy,
            [evidence::static_analysis()],
        );

        let cache_dir = root.join(".stencila/cache/runtime");
        fs::create_dir_all(&cache_dir)?;
        let code_digest = Sha256::digest(fs::read(&script_path)?)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        fs::write(
            cache_dir.join("trace.json"),
            serde_json::json!({
                "version": 1,
                "identity": "script:workspace:project/analysis.py",
                "codeDigest": code_digest,
                "events": [
                    {
                        "operation": "file_read",
                        "resource": "workspace:project/dynamic.csv",
                        "location": {"source": "analysis.py", "line": 0},
                        "count": 2
                    },
                    {
                        "operation": "remote_receive",
                        "resource": format!("https://doi.org/{doi}"),
                        "location": {"source": "analysis.py", "line": 1},
                        "count": 1
                    }
                ],
                "diagnostics": []
            })
            .to_string(),
        )?;

        add_cached_runtime_evidence(&mut builder, &graph_root);
        let graph = builder.build()?;
        let edge = graph
            .edges
            .iter()
            .find(|edge| {
                edge.source == input_id
                    && edge.target == script_id
                    && edge.kind == GraphEdgeKind::ReadBy
            })
            .ok_or_else(|| eyre::eyre!("expected merged read edge"))?;
        let kinds = edge
            .options
            .evidence
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|evidence| evidence.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                GraphEvidenceKind::StaticAnalysis,
                GraphEvidenceKind::RuntimeAnalysis
            ]
        );
        let edge = graph
            .edges
            .iter()
            .find(|edge| {
                edge.source == doi_id
                    && edge.target == script_id
                    && edge.kind == GraphEdgeKind::ReceivedBy
            })
            .ok_or_else(|| eyre::eyre!("expected merged DOI receive edge"))?;
        let kinds = edge
            .options
            .evidence
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(|evidence| evidence.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                GraphEvidenceKind::StaticAnalysis,
                GraphEvidenceKind::RuntimeAnalysis
            ]
        );

        Ok(())
    }

    #[test]
    fn cached_runtime_evidence_respects_document_scope() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let root = temp_dir.path().canonicalize()?;
        fs::write(root.join("a.csv"), "a\n")?;
        fs::write(root.join("b.csv"), "b\n")?;
        fs::write(root.join("helpers.py"), "VALUE = 1\n")?;

        let first = CodeChunk::new("open('a.csv').read()\n".into());
        let node_id = first.node_id();
        let mut second = first.clone();
        second.code = "open('b.csv').read()\n".into();
        let first_id = LocalGraphId::document_node("a.smd", &node_id);
        let second_id = LocalGraphId::document_node("b.smd", &node_id);
        let mut builder = GraphBuilder::new("test");
        builder.add_schema_node(first_id.clone(), Node::CodeChunk(first.clone()));
        builder.add_schema_node(second_id.clone(), Node::CodeChunk(second.clone()));

        let cache_dir = root.join(".stencila/cache/runtime");
        fs::create_dir_all(&cache_dir)?;
        for (index, (scope, chunk, resource)) in [
            ("a.smd", &first, "workspace:a.csv"),
            ("b.smd", &second, "workspace:b.csv"),
        ]
        .into_iter()
        .enumerate()
        {
            let code_digest = Sha256::digest(chunk.code.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            fs::write(
                cache_dir.join(format!("{index}.json")),
                serde_json::json!({
                    "version": 1,
                    "identity": format!("document:{scope}#{node_id}"),
                    "codeDigest": code_digest,
                    "events": [
                        {
                            "operation": "file_read",
                            "resource": resource,
                            "location": {"source": scope, "line": 0},
                            "count": 1
                        },
                        {
                            "operation": "import",
                            "resource": "sys",
                            "location": {"source": scope, "line": 0},
                            "count": 1
                        },
                        {
                            "operation": "import",
                            "resource": "helpers|workspace:helpers.py",
                            "location": {"source": scope, "line": 0},
                            "count": 1
                        }
                    ],
                    "diagnostics": []
                })
                .to_string(),
            )?;
        }

        add_cached_runtime_evidence(&mut builder, &root);
        let graph = builder.build()?;

        assert!(graph.edges.iter().any(|edge| {
            edge.source == "file:a.csv"
                && edge.target == first_id
                && edge.kind == GraphEdgeKind::ReadBy
        }));
        assert!(graph.edges.iter().any(|edge| {
            edge.source == "file:b.csv"
                && edge.target == second_id
                && edge.kind == GraphEdgeKind::ReadBy
        }));
        assert!(!graph.edges.iter().any(|edge| {
            (edge.source == "file:a.csv" && edge.target == second_id)
                || (edge.source == "file:b.csv" && edge.target == first_id)
        }));
        assert!(
            graph
                .nodes
                .iter()
                .all(|node| node.id != package_id("pypi", "sys", &[]))
        );
        assert!(graph.edges.iter().any(|edge| {
            edge.source == "code:helpers.py"
                && edge.target == first_id
                && edge.kind == GraphEdgeKind::ImportedBy
        }));
        assert!(
            graph
                .nodes
                .iter()
                .all(|node| node.id != package_id("pypi", "helpers", &[]))
        );

        Ok(())
    }

    #[test]
    fn cached_runtime_evidence_excludes_python_environment_files() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let root = temp_dir.path().canonicalize()?;
        let script_path = root.join("analysis.py");
        let input_path = root.join("input.csv");
        let environment = root.join("python-environment");
        let package_path = environment.join("lib/python3.13/site-packages/example");
        fs::create_dir_all(&package_path)?;
        fs::write(&script_path, "import example\nopen('input.csv').read()\n")?;
        fs::write(&input_path, "value\n1\n")?;
        fs::write(environment.join("pyvenv.cfg"), "home = /usr/bin\n")?;
        fs::write(package_path.join("__init__.py"), "VALUE = 1\n")?;
        fs::write(environment.join("environment.dat"), "internal\n")?;

        let script_rel = WorkspaceRelPath::from_relative_path(Path::new("analysis.py"))?;
        let script_id = LocalGraphId::code(script_rel.as_str());
        let mut builder = GraphBuilder::new("test");
        let mut script = SoftwareSourceCode::new("analysis.py".into(), "Python".into());
        script.path = Some("analysis.py".into());
        builder.add_schema_node(script_id.clone(), Node::SoftwareSourceCode(script));

        let cache_dir = root.join(".stencila/cache/runtime");
        fs::create_dir_all(&cache_dir)?;
        let code_digest = Sha256::digest(fs::read(&script_path)?)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        fs::write(
            cache_dir.join("trace.json"),
            serde_json::json!({
                "version": 1,
                "identity": "script:workspace:analysis.py",
                "codeDigest": code_digest,
                "events": [
                    {
                        "operation": "file_read",
                        "resource": "workspace:input.csv",
                        "location": {"source": "analysis.py", "line": 1},
                        "count": 1
                    },
                    {
                        "operation": "file_read",
                        "resource": "workspace:python-environment/environment.dat",
                        "location": {"source": "analysis.py", "line": 0},
                        "count": 1
                    },
                    {
                        "operation": "import",
                        "resource": "example|workspace:python-environment/lib/python3.13/site-packages/example/__init__.py",
                        "location": {"source": "analysis.py", "line": 0},
                        "count": 1
                    }
                ],
                "diagnostics": []
            })
            .to_string(),
        )?;

        add_cached_runtime_evidence(&mut builder, &root);
        let graph = builder.build()?;

        assert!(graph.edges.iter().any(|edge| {
            edge.source == "file:input.csv"
                && edge.target == script_id
                && edge.kind == GraphEdgeKind::ReadBy
        }));
        assert!(graph.edges.iter().any(|edge| {
            edge.source == package_id("pypi", "example", &[])
                && edge.target == script_id
                && edge.kind == GraphEdgeKind::ImportedBy
        }));
        assert!(
            graph
                .nodes
                .iter()
                .all(|node| !node.id.contains("python-environment"))
        );

        Ok(())
    }
}
