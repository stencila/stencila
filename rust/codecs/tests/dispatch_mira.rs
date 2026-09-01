//! Tests for MIRA JSON-LD registry dispatch through the codecs crate.

use stencila_codec::{
    CodecDirection, DecodeOptions, EncodeOptions,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{Graph, GraphEdge, GraphEdgeKind, GraphNode, Node, NodeType, Protocol},
};

#[test]
fn dispatches_mira_for_encoding_and_decoding() -> Result<()> {
    for direction in [CodecDirection::Encode, CodecDirection::Decode] {
        let codec = stencila_codecs::get(None, Some(&Format::MiraJsonLd), Some(direction))?;
        assert_eq!(codec.name(), "mira");
    }

    Ok(())
}

#[tokio::test]
async fn decodes_mira_through_registry() -> Result<()> {
    let content = include_str!("../../schema/tests/fixtures/mira/standalone-document.jsonld");
    let node = stencila_codecs::from_str(
        content,
        Some(DecodeOptions {
            format: Some(Format::MiraJsonLd),
            ..Default::default()
        }),
    )
    .await?;
    let Node::Graph(graph) = node else {
        stencila_codec::eyre::bail!("MIRA should decode to a Graph")
    };

    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.node.node_type() == NodeType::Claim)
    );
    assert_eq!(graph.edges.len(), 16);

    Ok(())
}

#[tokio::test]
async fn reports_known_mira_relation_endpoint_mismatches() -> Result<()> {
    let graph = Graph::new(
        "https://example.org/report.smd".to_string(),
        vec![
            GraphNode::new(
                "protocol-1".to_string(),
                Box::new(Node::Protocol(Protocol::new(Vec::new()))),
            ),
            GraphNode::new(
                "protocol-2".to_string(),
                Box::new(Node::Protocol(Protocol::new(Vec::new()))),
            ),
        ],
        vec![GraphEdge::new(
            "protocol-1".to_string(),
            "protocol-2".to_string(),
            GraphEdgeKind::Follows,
        )],
    );

    let (_content, info) = stencila_codecs::to_string_with_info(
        &Node::Graph(graph),
        Some(EncodeOptions {
            format: Some(Format::MiraJsonLd),
            ..Default::default()
        }),
    )
    .await?;
    assert!(info.losses.iter().any(|(label, count)| {
        label == "MiraJsonLd.relation.follows.domain.Protocol" && count == 1
    }));
    Ok(())
}
