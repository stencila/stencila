//! Graphviz DOT rendering for projected graph views.

use crate::{GraphView, GraphViewEdge, GraphViewNode, GraphViewNodeKind};
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

    for node in &view.nodes {
        render_node(&mut dot, node);
    }

    for edge in &view.edges {
        render_edge(&mut dot, edge);
    }

    dot.push_str("}\n");
    dot
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

fn render_edge(dot: &mut String, edge: &GraphViewEdge) {
    let style = edge_style(edge);
    dot.push_str(&format!(
        "  \"{}\" -> \"{}\" [label=\"{}\", kind=\"{}\", count=\"{}\", evidence_count=\"{}\", action_count=\"{}\", low_confidence=\"{}\", style=\"{}\", color=\"{}\"];\n",
        dot_escape(&edge.source),
        dot_escape(&edge.target),
        dot_escape(&edge.label),
        edge.kind,
        edge.count,
        edge.evidence_count,
        edge.action_count,
        edge.low_confidence,
        style.line_style,
        style.color
    ));
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
    use stencila_schema::{File, Graph, GraphEdge, GraphEdgeKind, GraphNode, Node};

    use crate::{GraphProjectionOptions, GraphProjectionPreset, project_graph};

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
}
