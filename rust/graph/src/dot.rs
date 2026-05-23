//! Graphviz DOT rendering for projected graph views.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    GraphProjectionDetail, GraphProjectionPreset, GraphView, GraphViewEdge, GraphViewNode,
    GraphViewNodeKind,
};
use stencila_schema::GraphEdgeKind;

/// Render a projected graph view as Graphviz DOT.
pub fn to_dot(view: &GraphView) -> String {
    let mut dot = String::new();

    dot.push_str("digraph stencila_graph {\n");
    dot.push_str("  graph [rankdir=\"LR\", overlap=\"false\", splines=\"true\"];\n");
    dot.push_str("  node [fontname=\"Arial\", fontsize=\"10\", margin=\"0.08,0.05\"];\n");
    dot.push_str("  edge [fontname=\"Arial\", fontsize=\"9\", arrowsize=\"0.7\"];\n");
    dot.push_str(&format!(
        "  graph [preset=\"{}\"];\n",
        dot_escape(view.preset.as_str())
    ));
    dot.push_str(&format!(
        "  graph [detail=\"{}\"];\n",
        dot_escape(view.detail.as_str())
    ));

    render_nodes(&mut dot, view);

    for edge in &view.edges {
        render_edge(&mut dot, edge, view.preset);
    }

    dot.push_str("}\n");
    dot
}

fn render_nodes(dot: &mut String, view: &GraphView) {
    if !view.containment.uses_clusters() {
        for node in &view.nodes {
            render_node(dot, node);
        }
        return;
    }

    let tree = ContainmentTree::new(view);
    let mut rendered = BTreeSet::new();

    for id in tree.root_containers() {
        render_cluster(dot, &tree, id, &mut rendered);
    }

    for node in &view.nodes {
        if !rendered.contains(node.id.as_str()) {
            render_node(dot, node);
        }
    }
}

fn render_cluster(
    dot: &mut String,
    tree: &ContainmentTree,
    id: &str,
    rendered: &mut BTreeSet<String>,
) {
    let Some(node) = tree.nodes.get(id) else {
        return;
    };

    if rendered.contains(id) {
        return;
    }

    let style = cluster_style(node.kind);
    dot.push_str(&format!(
        "  subgraph \"{}\" {{\n",
        dot_escape(&format!("cluster_{id}"))
    ));
    dot.push_str(&format!(
        "    label=\"{}\"; style=\"rounded,dashed\"; color=\"{}\"; bgcolor=\"{}\"; fontname=\"Arial\"; fontsize=\"11\";\n",
        dot_escape(&cluster_label(node)),
        style.color,
        style.background_color
    ));

    if node.kind != GraphViewNodeKind::Workspace {
        render_node(dot, node);
    }
    rendered.insert(id.to_string());

    for child in tree.children.get(id).into_iter().flatten() {
        if child == id {
            continue;
        }

        if tree.containers.contains(child.as_str()) {
            render_cluster(dot, tree, child, rendered);
        } else if let Some(child_node) = tree.nodes.get(child.as_str()) {
            render_node(dot, child_node);
            rendered.insert(child.clone());
        }
    }

    dot.push_str("  }\n");
}

fn cluster_label(node: &GraphViewNode) -> String {
    if node.kind == GraphViewNodeKind::Workspace && !node.label.ends_with('/') {
        format!("{}/", node.label)
    } else {
        node.label.clone()
    }
}

fn render_node(dot: &mut String, node: &GraphViewNode) {
    let style = node_style(node.kind);
    dot.push_str(&format!(
        "  \"{}\" [label=\"{}\", kind=\"{}\", shape=\"{}\", style=\"filled\", fillcolor=\"{}\", color=\"{}\"];\n",
        dot_escape(&node.id),
        dot_escape(&node.label),
        node.kind.as_str(),
        style.shape,
        style.fill_color,
        style.color
    ));
}

fn render_edge(dot: &mut String, edge: &GraphViewEdge, _preset: GraphProjectionPreset) {
    let style = edge_style(edge);
    let (source, target, label) = (&edge.source, &edge.target, edge.label.as_str());

    dot.push_str(&format!(
        "  \"{}\" -> \"{}\" [label=\"{}\", kind=\"{}\", count=\"{}\", evidence_count=\"{}\", action_count=\"{}\", low_confidence=\"{}\", style=\"{}\", color=\"{}\"];\n",
        dot_escape(source),
        dot_escape(target),
        dot_escape(label),
        edge.kind,
        edge.count,
        edge.evidence_count,
        edge.action_count,
        edge.low_confidence,
        style.line_style,
        style.color
    ));
}

#[derive(Debug)]
struct ContainmentTree<'a> {
    nodes: BTreeMap<&'a str, &'a GraphViewNode>,
    parents: BTreeMap<String, String>,
    children: BTreeMap<String, Vec<String>>,
    containers: BTreeSet<String>,
}

impl<'a> ContainmentTree<'a> {
    fn new(view: &'a GraphView) -> Self {
        let nodes = view
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        let raw_parents = view
            .containments
            .iter()
            .filter(|edge| {
                nodes.contains_key(edge.source.as_str()) && nodes.contains_key(edge.target.as_str())
            })
            .map(|edge| (edge.source.as_str(), edge.target.as_str()))
            .collect::<BTreeMap<_, _>>();
        let mut parents = BTreeMap::new();
        let mut children = BTreeMap::<String, Vec<String>>::new();
        let mut containers = BTreeSet::new();

        for node in &view.nodes {
            let Some(parent) =
                nearest_container_parent(&node.id, &nodes, &raw_parents, view.detail)
            else {
                continue;
            };

            parents
                .entry(node.id.clone())
                .or_insert_with(|| parent.clone());
            children
                .entry(parent.clone())
                .or_default()
                .push(node.id.clone());
            containers.insert(parent);
        }

        for children in children.values_mut() {
            children.sort();
            children.dedup();
        }

        Self {
            nodes,
            parents,
            children,
            containers,
        }
    }

    fn root_containers(&self) -> Vec<&str> {
        self.containers
            .iter()
            .filter(|id| {
                self.parents
                    .get(id.as_str())
                    .is_none_or(|parent| !self.containers.contains(parent))
            })
            .map(String::as_str)
            .collect()
    }
}

fn nearest_container_parent(
    id: &str,
    nodes: &BTreeMap<&str, &GraphViewNode>,
    raw_parents: &BTreeMap<&str, &str>,
    detail: GraphProjectionDetail,
) -> Option<String> {
    let mut current = id;
    let mut visited = BTreeSet::new();

    while visited.insert(current.to_string()) {
        let parent = raw_parents.get(current)?;
        let parent_node = nodes.get(*parent)?;
        if is_container(parent_node.kind, detail) {
            return Some((*parent).to_string());
        }
        current = parent;
    }

    None
}

fn is_container(kind: GraphViewNodeKind, detail: GraphProjectionDetail) -> bool {
    matches!(
        kind,
        GraphViewNodeKind::Workspace | GraphViewNodeKind::Document | GraphViewNodeKind::Environment
    ) || (detail == GraphProjectionDetail::High && kind == GraphViewNodeKind::Code)
}

#[derive(Debug, Clone, Copy)]
struct NodeStyle {
    shape: &'static str,
    fill_color: &'static str,
    color: &'static str,
}

fn node_style(kind: GraphViewNodeKind) -> NodeStyle {
    match kind {
        GraphViewNodeKind::Code => NodeStyle {
            shape: "box",
            fill_color: "#eef6f4",
            color: "#187b6b",
        },
        GraphViewNodeKind::Symbol => NodeStyle {
            shape: "ellipse",
            fill_color: "#eef6f4",
            color: "#187b6b",
        },
        GraphViewNodeKind::Function => NodeStyle {
            shape: "diamond",
            fill_color: "#eaf4fb",
            color: "#2870a6",
        },
        GraphViewNodeKind::Datatable => NodeStyle {
            shape: "box",
            fill_color: "#edf7ee",
            color: "#3f7f48",
        },
        GraphViewNodeKind::Package => NodeStyle {
            shape: "hexagon",
            fill_color: "#f8f0dd",
            color: "#a06d15",
        },
        GraphViewNodeKind::Reference => NodeStyle {
            shape: "note",
            fill_color: "#f1eef8",
            color: "#6f4fb0",
        },
        GraphViewNodeKind::Citation => NodeStyle {
            shape: "ellipse",
            fill_color: "#fff4f1",
            color: "#b64d38",
        },
        GraphViewNodeKind::Content => NodeStyle {
            shape: "box",
            fill_color: "#f7f4ea",
            color: "#8a6f2a",
        },
        GraphViewNodeKind::Workspace | GraphViewNodeKind::Environment => NodeStyle {
            shape: "folder",
            fill_color: "#f2f5f8",
            color: "#647486",
        },
        GraphViewNodeKind::Resource | GraphViewNodeKind::Output => NodeStyle {
            shape: "component",
            fill_color: "#eef2f7",
            color: "#516981",
        },
        GraphViewNodeKind::Document | GraphViewNodeKind::Other => NodeStyle {
            shape: "box",
            fill_color: "#ffffff",
            color: "#647486",
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct ClusterStyle {
    color: &'static str,
    background_color: &'static str,
}

fn cluster_style(kind: GraphViewNodeKind) -> ClusterStyle {
    match kind {
        GraphViewNodeKind::Environment => ClusterStyle {
            color: "#8fb36f",
            background_color: "#f6fbf2",
        },
        GraphViewNodeKind::Document => ClusterStyle {
            color: "#b8c2cc",
            background_color: "#fbfcfe",
        },
        _ => ClusterStyle {
            color: "#c3ccd6",
            background_color: "#f8fafc",
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct EdgeStyle {
    line_style: &'static str,
    color: &'static str,
}

fn edge_style(edge: &GraphViewEdge) -> EdgeStyle {
    if edge.kind == GraphEdgeKind::PartOf {
        return EdgeStyle {
            line_style: "dotted",
            color: "#8a94a3",
        };
    }

    if edge.low_confidence {
        return EdgeStyle {
            line_style: "dashed",
            color: "#8a94a3",
        };
    }

    EdgeStyle {
        line_style: "solid",
        color: "#4b5563",
    }
}

fn dot_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|char| match char {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => Vec::new(),
            _ => vec![char],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use stencila_schema::{
        Directory, File, Graph, GraphEdge, GraphEdgeKind, GraphNode, Node, SoftwareSourceCode,
    };

    use crate::{
        GraphContainmentMode, GraphProjectionDetail, GraphProjectionOptions, GraphProjectionPreset,
        project_graph,
    };

    use super::*;

    #[test]
    fn renders_projected_graph_as_dot() {
        let graph = Graph::new(
            "test:graph".to_string(),
            vec![
                GraphNode::new(
                    "file:data.csv".to_string(),
                    Box::new(Node::File(File::new(
                        "data.csv".to_string(),
                        "data.csv".to_string(),
                    ))),
                ),
                GraphNode::new(
                    "file:summary.csv".to_string(),
                    Box::new(Node::File(File::new(
                        "summary.csv".to_string(),
                        "summary.csv".to_string(),
                    ))),
                ),
            ],
            vec![GraphEdge::new(
                "file:data.csv".to_string(),
                "file:summary.csv".to_string(),
                GraphEdgeKind::DerivedInto,
            )],
        );
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let dot = to_dot(&view);

        assert!(dot.starts_with("digraph stencila_graph"));
        assert!(dot.contains("preset=\"flow\""));
        assert!(dot.contains("\"file:data.csv\""));
        assert!(dot.contains("label=\"Derived Into\""));
        assert!(dot.contains("evidence_count=\"0\""));
    }

    #[test]
    fn renders_containment_clusters_without_part_of_edges() {
        let graph = Graph::new(
            "test:cluster".to_string(),
            vec![
                graph_node(
                    "dir:scripts",
                    Node::Directory(Directory::new("scripts".to_string(), "scripts".to_string())),
                ),
                graph_node(
                    "file:scripts/analysis.py",
                    Node::File(File::new(
                        "analysis.py".to_string(),
                        "scripts/analysis.py".to_string(),
                    )),
                ),
                graph_node(
                    "code:scripts/analysis.py",
                    Node::SoftwareSourceCode(SoftwareSourceCode {
                        name: "analysis.py".to_string(),
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
                    "file:data.csv".to_string(),
                    "code:scripts/analysis.py".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "code:scripts/analysis.py".to_string(),
                    "file:scripts/analysis.py".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "file:scripts/analysis.py".to_string(),
                    "dir:scripts".to_string(),
                    GraphEdgeKind::PartOf,
                ),
            ],
        );
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
        );

        let dot = to_dot(&view);

        assert!(dot.contains("subgraph \"cluster_dir:scripts\""));
        assert!(dot.contains("label=\"scripts/\""));
        assert!(!dot.contains("\"dir:scripts\" ["));
        assert!(dot.contains("\"code:scripts/analysis.py\""));
        assert!(!dot.contains("label=\"Part Of\""));
    }

    #[test]
    fn renders_code_clusters_for_high_detail_containment() {
        let graph = Graph::new(
            "test:code-cluster".to_string(),
            vec![
                graph_node(
                    "dir:scripts",
                    Node::Directory(Directory::new("scripts".to_string(), "scripts".to_string())),
                ),
                graph_node("code:scripts/analysis.py", code_node("analysis.py")),
                graph_node(
                    "symbol:scripts/analysis.py:python:df",
                    Node::Variable(stencila_schema::Variable::new("df".to_string())),
                ),
                graph_node(
                    "file:data.csv",
                    Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:data.csv".to_string(),
                    "code:scripts/analysis.py".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "code:scripts/analysis.py".to_string(),
                    "symbol:scripts/analysis.py:python:df".to_string(),
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "symbol:scripts/analysis.py:python:df".to_string(),
                    "code:scripts/analysis.py".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "code:scripts/analysis.py".to_string(),
                    "dir:scripts".to_string(),
                    GraphEdgeKind::PartOf,
                ),
            ],
        );
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                detail: GraphProjectionDetail::High,
                ..Default::default()
            },
        );

        let dot = to_dot(&view);

        assert!(dot.contains("subgraph \"cluster_code:scripts/analysis.py\""));
        assert!(dot.contains("\"code:scripts/analysis.py\" ["));
        assert!(dot.contains("\"symbol:scripts/analysis.py:python:df\" ["));
        assert!(dot.contains("\"file:data.csv\" -> \"code:scripts/analysis.py\""));
    }

    #[test]
    fn renders_reactivity_in_update_order() {
        let graph = Graph::new(
            "test:react".to_string(),
            vec![
                graph_node("code:setup.py", code_node("setup.py")),
                graph_node(
                    "symbol:setup.py:x",
                    Node::Variable(stencila_schema::Variable::new("x".to_string())),
                ),
                graph_node("code:analysis.py", code_node("analysis.py")),
            ],
            vec![
                GraphEdge::new(
                    "code:setup.py".to_string(),
                    "symbol:setup.py:x".to_string(),
                    GraphEdgeKind::Generated,
                ),
                GraphEdge::new(
                    "symbol:setup.py:x".to_string(),
                    "code:analysis.py".to_string(),
                    GraphEdgeKind::UsedBy,
                ),
            ],
        );
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::React,
                containment: Some(GraphContainmentMode::None),
                ..Default::default()
            },
        );

        let dot = to_dot(&view);

        assert!(dot.contains("\"code:setup.py\" -> \"symbol:setup.py:x\""));
        assert!(dot.contains("\"symbol:setup.py:x\" -> \"code:analysis.py\""));
        assert!(dot.contains("label=\"Generated\""));
        assert!(dot.contains("label=\"Used By\""));
    }

    fn graph_node(id: &str, node: Node) -> GraphNode {
        GraphNode::new(id.to_string(), Box::new(node))
    }

    fn code_node(name: &str) -> Node {
        Node::SoftwareSourceCode(SoftwareSourceCode {
            name: name.to_string(),
            programming_language: "python".to_string(),
            ..Default::default()
        })
    }
}
