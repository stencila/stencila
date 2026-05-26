//! Graphviz DOT rendering for projected graph views.

use std::collections::{BTreeMap, BTreeSet};

use crate::{GraphView, GraphViewEdge, GraphViewNode, GraphViewNodeKind};
use stencila_schema::GraphEdgeKind;

const INDENT: &str = "  ";

/// Render a projected graph view as Graphviz DOT.
pub fn to_dot(view: &GraphView) -> String {
    let mut dot = String::new();

    push_line(&mut dot, 0, "digraph stencila_graph {");
    push_line(
        &mut dot,
        1,
        "graph [rankdir=\"LR\", overlap=\"false\", splines=\"true\"];",
    );
    push_line(
        &mut dot,
        1,
        "graph [fontname=\"Arial\", fontsize=\"11\", style=\"solid\"];",
    );
    push_line(
        &mut dot,
        1,
        "node [fontname=\"Arial\", fontsize=\"10\", margin=\"0.08,0.05\"];",
    );
    push_line(
        &mut dot,
        1,
        "edge [fontname=\"Arial\", fontsize=\"9\", arrowsize=\"0.7\"];",
    );
    push_line(
        &mut dot,
        1,
        &format!("graph [preset=\"{}\"];", dot_escape(view.preset.as_str())),
    );
    push_line(
        &mut dot,
        1,
        &format!("graph [detail=\"{}\"];", dot_escape(view.detail.as_str())),
    );

    if !view.nodes.is_empty() {
        dot.push('\n');
        render_nodes(&mut dot, view, 1);
    }

    if !view.edges.is_empty() {
        dot.push('\n');
        for edge in &view.edges {
            render_edge(&mut dot, edge, 1);
        }
    }

    push_line(&mut dot, 0, "}");
    dot
}

fn render_nodes(dot: &mut String, view: &GraphView, indent: usize) {
    if !view.containment.uses_clusters() {
        for node in &view.nodes {
            render_node(dot, node, indent);
        }
        return;
    }

    let tree = ContainmentTree::new(view);
    let mut rendered = BTreeSet::new();

    for id in tree.root_containers() {
        render_cluster(dot, &tree, id, &mut rendered, indent);
    }

    for node in &view.nodes {
        if !rendered.contains(node.id.as_str()) {
            render_node(dot, node, indent);
        }
    }
}

fn render_cluster(
    dot: &mut String,
    tree: &ContainmentTree,
    id: &str,
    rendered: &mut BTreeSet<String>,
    indent: usize,
) {
    let Some(node) = tree.nodes.get(id) else {
        return;
    };

    if rendered.contains(id) {
        return;
    }

    let style = cluster_style(node.kind);
    push_line(
        dot,
        indent,
        &format!("subgraph \"{}\" {{", dot_escape(&format!("cluster_{id}"))),
    );
    push_line(
        dot,
        indent + 1,
        &format!(
            "label=\"{}\"; color=\"{}\"; bgcolor=\"{}\";",
            dot_escape(&cluster_label(node)),
            style.color,
            style.background_color
        ),
    );

    if tree.edge_endpoints.contains(id) {
        render_node(dot, node, indent + 1);
    }
    rendered.insert(id.to_string());

    for child in tree.children.get(id).into_iter().flatten() {
        if child == id {
            continue;
        }

        if tree.containers.contains(child.as_str()) {
            render_cluster(dot, tree, child, rendered, indent + 1);
        } else if let Some(child_node) = tree.nodes.get(child.as_str()) {
            render_node(dot, child_node, indent + 1);
            rendered.insert(child.clone());
        }
    }

    push_line(dot, indent, "}");
}

fn cluster_label(node: &GraphViewNode) -> String {
    if node.kind == GraphViewNodeKind::Workspace && !node.label.ends_with('/') {
        format!("{}/", node.label)
    } else if node.kind == GraphViewNodeKind::Function {
        format!("fn {}", node.label)
    } else {
        node.label.clone()
    }
}

fn render_node(dot: &mut String, node: &GraphViewNode, indent: usize) {
    let style = node_style(node.kind);
    push_line(
        dot,
        indent,
        &format!(
            "\"{}\" [label=\"{}\", kind=\"{}\", shape=\"{}\", style=\"filled\", fillcolor=\"{}\", color=\"{}\"];",
            dot_escape(&node.id),
            dot_escape(&node.label),
            node.kind.as_str(),
            style.shape,
            style.fill_color,
            style.color
        ),
    );
}

fn render_edge(dot: &mut String, edge: &GraphViewEdge, indent: usize) {
    let style = edge_style(edge);
    let (source, target, label) = (&edge.source, &edge.target, edge.label.as_str());

    push_line(
        dot,
        indent,
        &format!(
            "\"{}\" -> \"{}\" [label=\"{}\", kind=\"{}\", count=\"{}\", evidence_count=\"{}\", action_count=\"{}\", low_confidence=\"{}\", style=\"{}\", color=\"{}\"];",
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
        ),
    );
}

fn push_line(dot: &mut String, indent: usize, line: &str) {
    for _ in 0..indent {
        dot.push_str(INDENT);
    }

    dot.push_str(line);
    dot.push('\n');
}

#[derive(Debug)]
struct ContainmentTree<'a> {
    nodes: BTreeMap<&'a str, &'a GraphViewNode>,
    parents: BTreeMap<String, String>,
    children: BTreeMap<String, Vec<String>>,
    containers: BTreeSet<String>,
    edge_endpoints: BTreeSet<String>,
}

impl<'a> ContainmentTree<'a> {
    fn new(view: &'a GraphView) -> Self {
        let nodes = view
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect::<BTreeMap<_, _>>();
        let edge_endpoints = view
            .edges
            .iter()
            .flat_map(|edge| [edge.source.clone(), edge.target.clone()])
            .collect::<BTreeSet<_>>();
        let raw_parents = view
            .containments
            .iter()
            .filter(|edge| {
                nodes.contains_key(edge.source.as_str()) && nodes.contains_key(edge.target.as_str())
            })
            .filter(|edge| edge.source != edge.target)
            .map(|edge| (edge.source.clone(), edge.target.clone()))
            .collect::<Vec<_>>();
        let mut parents = BTreeMap::new();
        let mut children = BTreeMap::<String, Vec<String>>::new();
        let mut containers = BTreeSet::new();

        for (child, parent) in raw_parents {
            parents
                .entry(child.clone())
                .or_insert_with(|| parent.clone());
            children.entry(parent.clone()).or_default().push(child);
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
            edge_endpoints,
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
        GraphViewNodeKind::Software => NodeStyle {
            shape: "box3d",
            fill_color: "#f1f5ff",
            color: "#5261b8",
        },
        GraphViewNodeKind::Resource | GraphViewNodeKind::Output => NodeStyle {
            shape: "component",
            fill_color: "#eef2f7",
            color: "#516981",
        },
        GraphViewNodeKind::Document => NodeStyle {
            shape: "note",
            fill_color: "#ffffff",
            color: "#647486",
        },
        GraphViewNodeKind::Other => NodeStyle {
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
    // Graphviz accepts RGBA hex colors and renders the alpha channel as opacity.
    // This lets nested clusters retain a hint of their parent background and borders.
    match kind {
        GraphViewNodeKind::Environment => ClusterStyle {
            color: "#8fb36f99",
            background_color: "#8fb36f2c",
        },
        GraphViewNodeKind::Document => ClusterStyle {
            color: "#b8c2cc99",
            background_color: "#b8c2cc2c",
        },
        _ => ClusterStyle {
            color: "#c3ccd699",
            background_color: "#c3ccd62c",
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
        Article, Directory, File, Function, Graph, GraphEdge, GraphEdgeKind, GraphNode, Node,
        SoftwareApplication, SoftwareSourceCode,
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
    fn renders_all_containers_as_clusters_and_hides_edge_less_container_nodes() {
        let graph = Graph::new(
            "test:function-cluster".to_string(),
            vec![
                graph_node(
                    "dir:scripts",
                    Node::Directory(Directory::new("scripts".to_string(), "scripts".to_string())),
                ),
                graph_node("code:scripts/analysis.py", code_node("analysis.py")),
                graph_node(
                    "function:scripts/analysis.py:python:summarize",
                    Node::Function(Function::new("summarize".to_string(), Vec::new())),
                ),
                graph_node(
                    "symbol:scripts/analysis.py:python:summarize:table",
                    Node::Variable(stencila_schema::Variable::new("table".to_string())),
                ),
                graph_node(
                    "file:data.csv",
                    Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
                ),
                graph_node(
                    "file:summary.csv",
                    Node::File(File::new(
                        "summary.csv".to_string(),
                        "summary.csv".to_string(),
                    )),
                ),
            ],
            vec![
                GraphEdge::new(
                    "file:data.csv".to_string(),
                    "symbol:scripts/analysis.py:python:summarize:table".to_string(),
                    GraphEdgeKind::ReadBy,
                ),
                GraphEdge::new(
                    "symbol:scripts/analysis.py:python:summarize:table".to_string(),
                    "file:summary.csv".to_string(),
                    GraphEdgeKind::WrittenTo,
                ),
                GraphEdge::new(
                    "symbol:scripts/analysis.py:python:summarize:table".to_string(),
                    "function:scripts/analysis.py:python:summarize".to_string(),
                    GraphEdgeKind::PartOf,
                ),
                GraphEdge::new(
                    "function:scripts/analysis.py:python:summarize".to_string(),
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
        assert!(dot.contains("subgraph \"cluster_function:scripts/analysis.py:python:summarize\""));
        assert!(dot.contains("label=\"fn summarize\""));
        assert!(!dot.contains("\"code:scripts/analysis.py\" ["));
        assert!(!dot.contains("\"function:scripts/analysis.py:python:summarize\" ["));
        assert!(dot.contains("\"symbol:scripts/analysis.py:python:summarize:table\" ["));
        assert!(dot.contains(
            "\"symbol:scripts/analysis.py:python:summarize:table\" -> \"file:summary.csv\""
        ));
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

    #[test]
    fn renders_decoded_documents_as_note_shapes() {
        let graph = Graph::new(
            "test:document-shape".to_string(),
            vec![graph_node(
                "node:report.html#art_",
                Node::Article(Article::new(Vec::new())),
            )],
            Vec::new(),
        );
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::All,
                containment: Some(GraphContainmentMode::None),
                ..Default::default()
            },
        );

        let dot = to_dot(&view);

        assert!(dot.contains(
            "\"node:report.html#art_\" [label=\"report.html\", kind=\"document\", shape=\"note\""
        ));
    }

    #[test]
    fn renders_software_applications_with_distinct_shape() {
        let graph = Graph::new(
            "test:software-shape".to_string(),
            vec![graph_node(
                "software:gpt-image",
                Node::SoftwareApplication(SoftwareApplication::new("gpt-image".to_string())),
            )],
            Vec::new(),
        );
        let view = project_graph(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::All,
                containment: Some(GraphContainmentMode::None),
                ..Default::default()
            },
        );

        let dot = to_dot(&view);

        assert!(dot.contains(
            "\"software:gpt-image\" [label=\"gpt-image\", kind=\"software\", shape=\"box3d\""
        ));
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
