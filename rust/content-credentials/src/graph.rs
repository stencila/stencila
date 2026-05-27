//! Projection of collected credential provenance into Stencila Schema `Graph`.

use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::{Value, json};
use stencila_schema::{
    ActionAgent, Array, CreateAction, CreativeWork, DateTime, ExecuteAction, File, Graph,
    GraphAction, GraphEdge, GraphEdgeKind, GraphEvidence, GraphEvidenceConfidence,
    GraphEvidenceKind, GraphNode, ImageObject, Node, Null, Object, Primitive, PropertyValue,
    PropertyValueOrString, SoftwareApplication, StringOrNumber,
};

use crate::{
    error::{Error, Result},
    snapshot::{
        AgentSnapshot, AssetSnapshot, DocumentSnapshot, IngredientRelationship, IngredientSnapshot,
        ProducerSnapshot, ProvenanceSnapshot, SourceRangeSnapshot,
    },
};

/// C2PA assertion label for Stencila provenance.
pub const PROVENANCE_LABEL: &str = "org.stencila.provenance";

/// Payload schema URL for the Stencila Schema graph used as the assertion body.
pub const PROVENANCE_SCHEMA: &str = concat!(
    "https://stencila.org/v",
    env!("CARGO_PKG_VERSION"),
    "/Graph.schema.json"
);

/// JSON-LD context URL for the Stencila Schema graph assertion body.
pub const PROVENANCE_CONTEXT: &str = concat!(
    "https://stencila.org/v",
    env!("CARGO_PKG_VERSION"),
    "/context.jsonld"
);

const ASSET_ID: &str = "asset:signed";
const PRODUCER_ID: &str = "software:producer";
const ROOT_ID: &str = "node:root";

const PROP_ASSET_TYPE: &str = "org.stencila.assetType";
const PROP_ASSET_ID: &str = "org.stencila.assetId";
const PROP_ASSET_ROLE: &str = "org.stencila.assetRole";
const PROP_ASSET_TITLE: &str = "org.stencila.assetTitle";
const PROP_ASSET_LABEL: &str = "org.stencila.assetLabel";
const PROP_ASSET_DESCRIPTION: &str = "org.stencila.assetDescription";
const PROP_MEDIA_TYPE: &str = "org.stencila.mediaType";
const PROP_CONTENT_DIGEST: &str = "org.stencila.contentDigest";
const PROP_PRODUCER_NAME: &str = "org.stencila.producer.name";
const PROP_PRODUCER_VERSION: &str = "org.stencila.producer.version";
const PROP_PRODUCER_CODEC: &str = "org.stencila.producer.codec";
const PROP_PRODUCER_RENDERER: &str = "org.stencila.producer.renderer";
const PROP_ACTIVITY_NAME: &str = "org.stencila.activity.name";
const PROP_ACTIVITY_STARTED_AT: &str = "org.stencila.activity.startedAt";
const PROP_ACTIVITY_ENDED_AT: &str = "org.stencila.activity.endedAt";
const PROP_SOURCE_REPOSITORY: &str = "org.stencila.source.repository";
const PROP_SOURCE_COMMIT: &str = "org.stencila.source.commit";
const PROP_SOURCE_PATH: &str = "org.stencila.source.path";
const PROP_SOURCE_DIRTY: &str = "org.stencila.source.dirty";
const PROP_SOURCE_RANGE: &str = "org.stencila.sourceRange";
const PROP_REDACTION_COUNT: &str = "org.stencila.redactionCount";
const PROP_NODE_TYPE: &str = "org.stencila.nodeType";
const PROP_NODE_ID: &str = "org.stencila.nodeId";
const PROP_PERSISTENT_ID: &str = "org.stencila.persistentId";
const PROP_PROGRAMMING_LANGUAGE: &str = "org.stencila.programmingLanguage";
const PROP_EXECUTION: &str = "org.stencila.execution";
const PROP_AI_STANDARD_ASSERTION: &str = "org.stencila.aiDisclosure.standardAssertion";

/// Compact metadata extracted from the credential graph for C2PA projections.
#[derive(Debug, Clone, Default)]
pub(crate) struct GraphMetadata {
    pub(crate) asset_type: Option<String>,
    pub(crate) asset_role: Option<String>,
    pub(crate) asset_title: Option<String>,
    pub(crate) asset_label: Option<String>,
    pub(crate) media_type: Option<String>,
    pub(crate) producer_name: Option<String>,
    pub(crate) producer_version: Option<String>,
    pub(crate) producer_codec: Option<String>,
    pub(crate) producer_renderer: Option<String>,
    pub(crate) activity_name: Option<String>,
    pub(crate) activity_started_at: Option<String>,
    pub(crate) activity_ended_at: Option<String>,
    pub(crate) source_repository: Option<String>,
    pub(crate) source_commit: Option<String>,
    pub(crate) source_path: Option<String>,
    pub(crate) source_dirty: Option<bool>,
    pub(crate) source_range: Option<String>,
    pub(crate) redaction_count: Option<u32>,
    pub(crate) executed_node_type: Option<String>,
    pub(crate) executed_node_id: Option<String>,
    pub(crate) executed_persistent_id: Option<String>,
    pub(crate) programming_language: Option<String>,
    pub(crate) execution: Option<Value>,
    pub(crate) ai_standard_assertion: Option<String>,
}

/// Build the graph embedded as Stencila's custom C2PA assertion.
#[must_use]
#[allow(clippy::too_many_lines)]
pub(crate) fn graph_from_snapshot(
    snapshot: &ProvenanceSnapshot,
    ingredients: &[IngredientSnapshot],
) -> Graph {
    let mut nodes = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut edges = Vec::new();
    let evidence = recorded_evidence(snapshot);
    let metadata = metadata_identifiers(snapshot);

    add_asset_node(&mut nodes, &mut node_ids, &snapshot.asset);
    add_document_node(
        &mut nodes,
        &mut node_ids,
        ROOT_ID,
        "Root Stencila document node",
        &snapshot.root_node,
    );
    add_producer_node(&mut nodes, &mut node_ids, snapshot.producer.as_ref());

    let root_to_asset = GraphEdgeKind::ConvertedInto;
    edges.push(edge_with_evidence_and_action(
        ROOT_ID,
        ASSET_ID,
        root_to_asset,
        evidence.clone(),
        Some(create_action(snapshot)),
    ));
    edges.push(edge_with_evidence_and_action(
        PRODUCER_ID,
        ASSET_ID,
        GraphEdgeKind::Generated,
        evidence.clone(),
        Some(create_action(snapshot)),
    ));

    let executed_id = snapshot
        .executed_node
        .as_ref()
        .map(|node| document_node_id("executed", node));
    if let (Some(id), Some(node)) = (executed_id.as_deref(), snapshot.executed_node.as_ref()) {
        add_document_node(
            &mut nodes,
            &mut node_ids,
            id,
            "Executed Stencila node",
            node,
        );
        edges.push(edge_with_evidence(
            id,
            ROOT_ID,
            GraphEdgeKind::PartOf,
            evidence.clone(),
        ));
    }

    let output_id = snapshot
        .output_node
        .as_ref()
        .map(|node| document_node_id("output", node));
    if let (Some(id), Some(node)) = (output_id.as_deref(), snapshot.output_node.as_ref()) {
        add_document_node(&mut nodes, &mut node_ids, id, "Output Stencila node", node);
        if let Some(executed_id) = executed_id.as_deref() {
            edges.push(edge_with_evidence_and_action(
                executed_id,
                id,
                GraphEdgeKind::Generated,
                evidence.clone(),
                execute_action(snapshot),
            ));
        }
        edges.push(edge_with_evidence(
            id,
            ASSET_ID,
            GraphEdgeKind::ConvertedInto,
            evidence.clone(),
        ));
    } else if let Some(executed_id) = executed_id.as_deref() {
        edges.push(edge_with_evidence_and_action(
            executed_id,
            ASSET_ID,
            GraphEdgeKind::Generated,
            evidence.clone(),
            execute_action(snapshot),
        ));
    }

    if let Some(source) = snapshot.source.as_ref()
        && let Some(path) = source.path.as_deref()
    {
        let source_id = "source:file";
        let mut source_file = File::new(path.to_string(), path.to_string());
        source_file.id = Some(source_id.to_string());
        source_file.options.identifiers = Some(
            [
                property_opt(PROP_SOURCE_REPOSITORY, source.repository.as_deref()),
                property_opt(PROP_SOURCE_COMMIT, source.commit.as_deref()),
                property_opt(PROP_SOURCE_PATH, source.path.as_deref()),
                source
                    .dirty
                    .map(|dirty| property(PROP_SOURCE_DIRTY, Primitive::Boolean(dirty))),
            ]
            .into_iter()
            .flatten()
            .collect(),
        );
        add_node(
            &mut nodes,
            &mut node_ids,
            source_id,
            Node::File(source_file),
        );
        edges.push(edge_with_evidence(
            source_id,
            ROOT_ID,
            GraphEdgeKind::ReadBy,
            evidence.clone(),
        ));
    }

    for (index, ingredient) in ingredients.iter().enumerate() {
        let id = ingredient_node_id(ingredient, index);
        add_ingredient_node(&mut nodes, &mut node_ids, &id, ingredient);
        edges.push(edge_with_evidence(
            &id,
            ASSET_ID,
            ingredient_edge_kind(ingredient.relationship),
            evidence.clone(),
        ));
    }

    for (index, attribution) in snapshot.attributions.iter().enumerate() {
        add_agent_node(
            &mut nodes,
            &mut node_ids,
            &format!("agent:attribution:{index}"),
            &attribution.agent,
        );
        edges.push(edge_with_evidence(
            &format!("agent:attribution:{index}"),
            ASSET_ID,
            GraphEdgeKind::Generated,
            evidence.clone(),
        ));
    }

    if let Some(disclosure) = snapshot.ai_disclosure.as_ref() {
        let name = disclosure
            .model_name
            .as_deref()
            .or(disclosure.model_identifier.as_deref())
            .unwrap_or("AI model");
        let mut model = SoftwareApplication::new(name.to_string());
        let id = "software:ai-model";
        model.id = Some(id.to_string());
        model.options.identifiers = Some(
            [
                property_opt(
                    "org.stencila.aiDisclosure.modelType",
                    Some(disclosure.model_type.as_str()),
                ),
                property_opt(
                    "org.stencila.aiDisclosure.modelIdentifier",
                    disclosure.model_identifier.as_deref(),
                ),
                property_opt(
                    PROP_AI_STANDARD_ASSERTION,
                    disclosure.standard_assertion.as_deref(),
                ),
            ]
            .into_iter()
            .flatten()
            .collect(),
        );
        add_node(
            &mut nodes,
            &mut node_ids,
            id,
            Node::SoftwareApplication(model),
        );
        edges.push(edge_with_evidence(
            id,
            ASSET_ID,
            GraphEdgeKind::Generated,
            evidence.clone(),
        ));
    }

    let mut graph = Graph::new(
        format!(
            "urn:stencila:content-credential:{}",
            snapshot.asset.content_digest
        ),
        nodes,
        edges,
    );
    graph.id = Some("graph:content-credential".to_string());
    graph.options.name = Some("Stencila content credential provenance".to_string());
    graph.options.description =
        Some("Asset-centered provenance graph embedded in a C2PA manifest".to_string());
    graph.options.identifiers = Some(metadata);
    graph
}

/// Serialize a provenance graph with standalone JSON metadata.
///
/// This mirrors the standalone JSON/YAML codecs by adding the versioned JSON
/// Schema and JSON-LD context URLs at the top of the assertion payload.
pub(crate) fn graph_assertion_payload(graph: &Graph) -> Result<Value> {
    let value = serde_json::to_value(graph)?;
    let Some(object) = value.as_object() else {
        return Err(Error::other(
            "Stencila provenance graph did not serialize to a JSON object",
        ));
    };

    let mut root = serde_json::Map::with_capacity(object.len() + 2);
    root.insert(
        "$schema".to_string(),
        Value::String(PROVENANCE_SCHEMA.to_string()),
    );
    root.insert(
        "@context".to_string(),
        Value::String(PROVENANCE_CONTEXT.to_string()),
    );
    for (key, value) in object {
        root.insert(key.clone(), value.clone());
    }

    Ok(Value::Object(root))
}

/// Extract graph metadata used for standard C2PA assertions and reports.
#[must_use]
pub(crate) fn metadata_from_graph(graph: &Graph) -> GraphMetadata {
    let mut metadata = GraphMetadata::default();
    if let Ok(value) = serde_json::to_value(graph) {
        read_identifiers(value.get("identifiers"), &mut metadata);

        for node in value
            .get("nodes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            read_identifiers(node.pointer("/node/identifiers"), &mut metadata);
        }

        for edge in value
            .get("edges")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for evidence in evidence_values(edge) {
                read_identifiers(
                    evidence
                        .get("details")
                        .and_then(|details| details.get("identifiers")),
                    &mut metadata,
                );
            }
        }
    }

    metadata
}

/// Maps a media type to the broad Stencila asset kind used in credentials.
#[must_use]
pub(crate) fn asset_kind_for_media_type(media_type: &str) -> &'static str {
    match media_type {
        value if value.starts_with("image/") => "image",
        value if value.starts_with("video/") => "video",
        value if value.starts_with("audio/") => "audio",
        "text/csv" | "text/tab-separated-values" | "application/json" => "dataset",
        "application/pdf"
        | "text/html"
        | "text/markdown"
        | "text/plain"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => "document",
        _ => "asset",
    }
}

fn add_asset_node(
    nodes: &mut Vec<GraphNode>,
    node_ids: &mut BTreeSet<String>,
    asset: &AssetSnapshot,
) {
    let name = asset
        .title
        .as_deref()
        .or(asset.label.as_deref())
        .unwrap_or("asset")
        .to_string();

    let node = if asset.media_type.starts_with("image/") {
        let mut image = ImageObject::new(asset.label.clone().unwrap_or_else(|| name.clone()));
        image.id = Some(ASSET_ID.to_string());
        image.media_type = Some(asset.media_type.clone());
        image.options.name = Some(name);
        image.options.description.clone_from(&asset.description);
        image.options.identifiers = Some(asset_identifiers(asset));
        Node::ImageObject(image)
    } else {
        let mut file = File::new(
            name.clone(),
            asset.label.clone().unwrap_or_else(|| name.clone()),
        );
        file.id = Some(ASSET_ID.to_string());
        file.media_type = Some(asset.media_type.clone());
        file.size = asset.size;
        file.options.description.clone_from(&asset.description);
        file.options.identifiers = Some(asset_identifiers(asset));
        Node::File(file)
    };

    add_node(nodes, node_ids, ASSET_ID, node);
}

fn asset_identifiers(asset: &AssetSnapshot) -> Vec<PropertyValueOrString> {
    let mut identifiers = vec![
        property(PROP_ASSET_TYPE, Primitive::String(asset.kind.clone())),
        property(PROP_MEDIA_TYPE, Primitive::String(asset.media_type.clone())),
        property(
            PROP_CONTENT_DIGEST,
            Primitive::String(asset.content_digest.clone()),
        ),
    ];
    if let Some(role) = asset.role.as_deref() {
        identifiers.push(property(
            PROP_ASSET_ROLE,
            Primitive::String(role.to_string()),
        ));
    }
    if let Some(id) = asset.id.as_deref() {
        identifiers.push(property(PROP_ASSET_ID, Primitive::String(id.to_string())));
    }
    if let Some(title) = asset.title.as_deref() {
        identifiers.push(property(
            PROP_ASSET_TITLE,
            Primitive::String(title.to_string()),
        ));
    }
    if let Some(label) = asset.label.as_deref() {
        identifiers.push(property(
            PROP_ASSET_LABEL,
            Primitive::String(label.to_string()),
        ));
    }
    if let Some(description) = asset.description.as_deref() {
        identifiers.push(property(
            PROP_ASSET_DESCRIPTION,
            Primitive::String(description.to_string()),
        ));
    }
    identifiers
}

fn add_document_node(
    nodes: &mut Vec<GraphNode>,
    node_ids: &mut BTreeSet<String>,
    id: &str,
    role: &str,
    document: &DocumentSnapshot,
) {
    let mut node = CreativeWork::new();
    node.id = Some(id.to_string());
    node.options.name = document
        .title
        .clone()
        .or_else(|| document.label.clone())
        .or_else(|| Some(document.node_type.clone()));
    node.options.description = Some(format!("{role}: {}", document.node_type));
    node.options.url.clone_from(&document.content_url);
    node.options.identifiers = Some(document_identifiers(document));
    add_node(nodes, node_ids, id, Node::CreativeWork(node));
}

fn document_identifiers(document: &DocumentSnapshot) -> Vec<PropertyValueOrString> {
    let mut identifiers = vec![property(
        PROP_NODE_TYPE,
        Primitive::String(document.node_type.clone()),
    )];
    for (key, value) in [
        (PROP_NODE_ID, document.node_id.as_deref()),
        (PROP_PERSISTENT_ID, document.persistent_id.as_deref()),
        ("org.stencila.nodePath", document.node_path.as_deref()),
        ("org.stencila.labelType", document.label_type.as_deref()),
        ("org.stencila.label", document.label.as_deref()),
        (
            PROP_PROGRAMMING_LANGUAGE,
            document.programming_language.as_deref(),
        ),
        ("org.stencila.contentUrl", document.content_url.as_deref()),
        (PROP_MEDIA_TYPE, document.media_type.as_deref()),
    ] {
        if let Some(value) = value {
            identifiers.push(property(key, Primitive::String(value.to_string())));
        }
    }
    if let Some(range) = document.source_range.as_ref() {
        identifiers.push(property(
            PROP_SOURCE_RANGE,
            Primitive::String(format_source_range(range)),
        ));
    }
    identifiers
}

fn add_producer_node(
    nodes: &mut Vec<GraphNode>,
    node_ids: &mut BTreeSet<String>,
    producer: Option<&ProducerSnapshot>,
) {
    let name = producer
        .and_then(|producer| producer.name.as_deref())
        .unwrap_or("Stencila");
    let version = producer
        .and_then(|producer| producer.version.clone())
        .unwrap_or_else(|| stencila_version::STENCILA_VERSION.to_string());
    let mut node = SoftwareApplication::new(name.to_string());
    node.id = Some(PRODUCER_ID.to_string());
    node.version = Some(StringOrNumber::String(version.clone()));
    node.options.software_version = Some(version);
    node.options.identifiers = Some(metadata_identifiers_for_producer(producer));
    add_node(
        nodes,
        node_ids,
        PRODUCER_ID,
        Node::SoftwareApplication(node),
    );
}

fn add_ingredient_node(
    nodes: &mut Vec<GraphNode>,
    node_ids: &mut BTreeSet<String>,
    id: &str,
    ingredient: &IngredientSnapshot,
) {
    let title = ingredient
        .title
        .clone()
        .unwrap_or_else(|| "ingredient".to_string());
    let mut node = File::new(title.clone(), title);
    node.id = Some(id.to_string());
    node.media_type.clone_from(&ingredient.media_type);
    node.options.description.clone_from(&ingredient.description);
    node.options.url.clone_from(&ingredient.informational_uri);
    node.options.identifiers = Some(
        [
            ingredient
                .content_digest
                .as_deref()
                .map(|value| property(PROP_CONTENT_DIGEST, Primitive::String(value.to_string()))),
            ingredient.label.as_deref().map(|value| {
                property(
                    "org.stencila.ingredientLabel",
                    Primitive::String(value.to_string()),
                )
            }),
            Some(property(
                "org.stencila.ingredientRelationship",
                Primitive::String(
                    ingredient_relationship_name(ingredient.relationship).to_string(),
                ),
            )),
        ]
        .into_iter()
        .flatten()
        .collect(),
    );
    add_node(nodes, node_ids, id, Node::File(node));
}

fn add_agent_node(
    nodes: &mut Vec<GraphNode>,
    node_ids: &mut BTreeSet<String>,
    id: &str,
    agent: &AgentSnapshot,
) {
    let name = agent.name.as_deref().unwrap_or("agent");
    let mut node = SoftwareApplication::new(name.to_string());
    node.id = Some(id.to_string());
    node.options.software_version.clone_from(&agent.version);
    node.options.description.clone_from(&agent.provider);
    node.options.url.clone_from(&agent.url);
    add_node(nodes, node_ids, id, Node::SoftwareApplication(node));
}

fn add_node(nodes: &mut Vec<GraphNode>, node_ids: &mut BTreeSet<String>, id: &str, node: Node) {
    if node_ids.insert(id.to_string()) {
        nodes.push(GraphNode::new(id.to_string(), Box::new(node)));
    }
}

fn edge_with_evidence(
    source: &str,
    target: &str,
    kind: GraphEdgeKind,
    evidence: GraphEvidence,
) -> GraphEdge {
    let mut edge = GraphEdge::new(source.to_string(), target.to_string(), kind);
    edge.options.evidence = Some(vec![evidence]);
    edge
}

fn edge_with_evidence_and_action(
    source: &str,
    target: &str,
    kind: GraphEdgeKind,
    evidence: GraphEvidence,
    action: Option<GraphAction>,
) -> GraphEdge {
    let mut edge = edge_with_evidence(source, target, kind, evidence);
    edge.options.actions = action.map(|action| vec![action]);
    edge
}

fn recorded_evidence(snapshot: &ProvenanceSnapshot) -> GraphEvidence {
    let mut evidence = GraphEvidence::new(GraphEvidenceKind::Recorded);
    evidence.confidence = Some(GraphEvidenceConfidence::Certain);
    evidence.options.description = Some("Projected from Stencila export provenance".to_string());
    evidence.options.details = Some(object_from_json(json!({
        "identifiers": metadata_identifiers_as_json(snapshot),
        "activity": snapshot.activity,
        "source": snapshot.source,
        "execution": execution_details(snapshot),
        "workflow": snapshot.workflow,
        "environment": snapshot.environment,
        "aiDisclosure": snapshot.ai_disclosure,
        "provenanceSummary": snapshot.provenance_summary,
        "reproducibility": snapshot.reproducibility,
        "privacy": snapshot.privacy,
    })));
    evidence
}

fn create_action(snapshot: &ProvenanceSnapshot) -> GraphAction {
    let mut action = CreateAction::new();
    action.id = Some("action:created".to_string());
    action.options.name = snapshot
        .activity
        .as_ref()
        .and_then(|activity| activity.name.clone())
        .or_else(|| Some("Create asset".to_string()));
    action.agent = Some(producer_agent(snapshot.producer.as_ref()));
    if let Some(started) = snapshot
        .activity
        .as_ref()
        .and_then(|activity| activity.started_at.clone())
    {
        action.start_time = Some(DateTime::new(started));
    }
    if let Some(ended) = snapshot
        .activity
        .as_ref()
        .and_then(|activity| activity.ended_at.clone())
    {
        action.end_time = Some(DateTime::new(ended));
    }
    GraphAction::CreateAction(action)
}

fn execute_action(snapshot: &ProvenanceSnapshot) -> Option<GraphAction> {
    let executed = snapshot.executed_node.as_ref()?;
    let mut action = ExecuteAction::new();
    action.id = Some("action:executed".to_string());
    action.options.name = Some(format!("Execute {}", executed.node_type));
    action.options.description = snapshot
        .execution
        .as_ref()
        .and_then(|execution| execution.status.clone())
        .map(|status| format!("Stencila execution status: {status}"));
    action.agent = Some(producer_agent(snapshot.producer.as_ref()));
    if let Some(ended) = snapshot
        .execution
        .as_ref()
        .and_then(|execution| execution.ended_at.clone())
        .or_else(|| {
            snapshot
                .activity
                .as_ref()
                .and_then(|activity| activity.ended_at.clone())
        })
    {
        action.end_time = Some(DateTime::new(ended));
    }
    Some(GraphAction::ExecuteAction(action))
}

fn producer_agent(producer: Option<&ProducerSnapshot>) -> ActionAgent {
    let name = producer
        .and_then(|producer| producer.name.as_deref())
        .unwrap_or("Stencila");
    let version = producer
        .and_then(|producer| producer.version.clone())
        .unwrap_or_else(|| stencila_version::STENCILA_VERSION.to_string());
    let mut software = SoftwareApplication::new(name.to_string());
    software.version = Some(StringOrNumber::String(version.clone()));
    software.options.software_version = Some(version);
    ActionAgent::SoftwareApplication(software)
}

fn metadata_identifiers(snapshot: &ProvenanceSnapshot) -> Vec<PropertyValueOrString> {
    metadata_identifier_entries(snapshot)
        .into_iter()
        .map(|(key, value)| property(key, value))
        .collect()
}

fn metadata_identifiers_as_json(snapshot: &ProvenanceSnapshot) -> Value {
    Value::Array(
        metadata_identifier_entries(snapshot)
            .into_iter()
            .map(|(key, value)| {
                json!({
                    "type": "PropertyValue",
                    "propertyId": key,
                    "value": primitive_to_json(&value),
                })
            })
            .collect(),
    )
}

#[allow(clippy::too_many_lines)]
fn metadata_identifier_entries(snapshot: &ProvenanceSnapshot) -> Vec<(&'static str, Primitive)> {
    let mut entries = vec![
        (
            PROP_ASSET_TYPE,
            Primitive::String(snapshot.asset.kind.clone()),
        ),
        (
            PROP_MEDIA_TYPE,
            Primitive::String(snapshot.asset.media_type.clone()),
        ),
        (
            PROP_CONTENT_DIGEST,
            Primitive::String(snapshot.asset.content_digest.clone()),
        ),
    ];

    for (key, value) in [
        (PROP_ASSET_ROLE, snapshot.asset.role.as_deref()),
        (PROP_ASSET_ID, snapshot.asset.id.as_deref()),
        (PROP_ASSET_TITLE, snapshot.asset.title.as_deref()),
        (PROP_ASSET_LABEL, snapshot.asset.label.as_deref()),
        (
            PROP_ASSET_DESCRIPTION,
            snapshot.asset.description.as_deref(),
        ),
        (
            PROP_PRODUCER_NAME,
            snapshot
                .producer
                .as_ref()
                .and_then(|producer| producer.name.as_deref())
                .or(Some("Stencila")),
        ),
        (
            PROP_PRODUCER_VERSION,
            snapshot
                .producer
                .as_ref()
                .and_then(|producer| producer.version.as_deref())
                .or(Some(stencila_version::STENCILA_VERSION)),
        ),
        (
            PROP_PRODUCER_CODEC,
            snapshot
                .producer
                .as_ref()
                .and_then(|producer| producer.codec.as_deref()),
        ),
        (
            PROP_PRODUCER_RENDERER,
            snapshot
                .producer
                .as_ref()
                .and_then(|producer| producer.renderer.as_deref()),
        ),
        (
            PROP_ACTIVITY_NAME,
            snapshot
                .activity
                .as_ref()
                .and_then(|activity| activity.name.as_deref()),
        ),
        (
            PROP_ACTIVITY_STARTED_AT,
            snapshot
                .activity
                .as_ref()
                .and_then(|activity| activity.started_at.as_deref()),
        ),
        (
            PROP_ACTIVITY_ENDED_AT,
            snapshot
                .activity
                .as_ref()
                .and_then(|activity| activity.ended_at.as_deref()),
        ),
        (
            PROP_SOURCE_REPOSITORY,
            snapshot
                .source
                .as_ref()
                .and_then(|source| source.repository.as_deref()),
        ),
        (
            PROP_SOURCE_COMMIT,
            snapshot
                .source
                .as_ref()
                .and_then(|source| source.commit.as_deref()),
        ),
        (
            PROP_SOURCE_PATH,
            snapshot
                .source
                .as_ref()
                .and_then(|source| source.path.as_deref()),
        ),
        (
            PROP_NODE_TYPE,
            snapshot
                .executed_node
                .as_ref()
                .map(|node| node.node_type.as_str()),
        ),
        (
            PROP_NODE_ID,
            snapshot
                .executed_node
                .as_ref()
                .and_then(|node| node.node_id.as_deref()),
        ),
        (
            PROP_PERSISTENT_ID,
            snapshot
                .executed_node
                .as_ref()
                .and_then(|node| node.persistent_id.as_deref()),
        ),
        (
            PROP_PROGRAMMING_LANGUAGE,
            snapshot
                .executed_node
                .as_ref()
                .and_then(|node| node.programming_language.as_deref()),
        ),
        (
            PROP_AI_STANDARD_ASSERTION,
            snapshot
                .ai_disclosure
                .as_ref()
                .and_then(|disclosure| disclosure.standard_assertion.as_deref()),
        ),
    ] {
        if let Some(value) = value {
            entries.push((key, Primitive::String(value.to_string())));
        }
    }

    if let Some(dirty) = snapshot.source.as_ref().and_then(|source| source.dirty) {
        entries.push((PROP_SOURCE_DIRTY, Primitive::Boolean(dirty)));
    }

    if let Some(range) = snapshot
        .executed_node
        .as_ref()
        .and_then(|node| node.source_range.as_ref())
        .or_else(|| {
            snapshot
                .output_node
                .as_ref()
                .and_then(|node| node.source_range.as_ref())
        })
        .or(snapshot.root_node.source_range.as_ref())
    {
        entries.push((
            PROP_SOURCE_RANGE,
            Primitive::String(format_source_range(range)),
        ));
    }

    if let Some(privacy) = snapshot.privacy.as_ref()
        && let Ok(count) = u64::try_from(privacy.redactions.len())
    {
        entries.push((PROP_REDACTION_COUNT, Primitive::UnsignedInteger(count)));
    }

    if let Some(execution) = execution_details(snapshot) {
        entries.push((PROP_EXECUTION, primitive_from_json(execution)));
    }

    entries
}

fn metadata_identifiers_for_producer(
    producer: Option<&ProducerSnapshot>,
) -> Vec<PropertyValueOrString> {
    [
        property_opt(
            PROP_PRODUCER_NAME,
            producer
                .and_then(|producer| producer.name.as_deref())
                .or(Some("Stencila")),
        ),
        property_opt(
            PROP_PRODUCER_VERSION,
            producer
                .and_then(|producer| producer.version.as_deref())
                .or(Some(stencila_version::STENCILA_VERSION)),
        ),
        property_opt(
            PROP_PRODUCER_CODEC,
            producer.and_then(|producer| producer.codec.as_deref()),
        ),
        property_opt(
            PROP_PRODUCER_RENDERER,
            producer.and_then(|producer| producer.renderer.as_deref()),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn property_opt(key: &'static str, value: Option<&str>) -> Option<PropertyValueOrString> {
    value.map(|value| property(key, Primitive::String(value.to_string())))
}

fn property(key: &'static str, value: Primitive) -> PropertyValueOrString {
    let mut property = PropertyValue::new(value);
    property.property_id = Some(key.to_string());
    PropertyValueOrString::PropertyValue(property)
}

fn execution_details(snapshot: &ProvenanceSnapshot) -> Option<Value> {
    let execution = snapshot.execution.as_ref()?;
    let mut value = json_or_null(execution);
    if let Value::Object(object) = &mut value {
        object.remove("dependencies");
        if let Some(status) = object
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_ascii_lowercase)
        {
            object.insert("status".to_string(), Value::String(status));
        }
    }
    Some(value)
}

fn object_from_json(value: Value) -> Object {
    match value {
        Value::Object(object) => Object(
            object
                .into_iter()
                .map(|(key, value)| (key, primitive_from_json(value)))
                .collect(),
        ),
        other => Object(
            [("value".to_string(), primitive_from_json(other))]
                .into_iter()
                .collect(),
        ),
    }
}

fn primitive_from_json(value: Value) -> Primitive {
    match value {
        Value::Null => Primitive::Null(Null),
        Value::Bool(value) => Primitive::Boolean(value),
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                Primitive::Integer(value)
            } else if let Some(value) = number.as_u64() {
                Primitive::UnsignedInteger(value)
            } else {
                Primitive::Number(number.as_f64().unwrap_or_default())
            }
        }
        Value::String(value) => Primitive::String(value),
        Value::Array(values) => {
            Primitive::Array(Array(values.into_iter().map(primitive_from_json).collect()))
        }
        Value::Object(object) => Primitive::Object(Object(
            object
                .into_iter()
                .map(|(key, value)| (key, primitive_from_json(value)))
                .collect(),
        )),
    }
}

fn primitive_to_json(value: &Primitive) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn json_or_null(value: impl Serialize) -> Value {
    serde_json::to_value(value).unwrap_or(Value::Null)
}

fn document_node_id(prefix: &str, node: &DocumentSnapshot) -> String {
    let identity = node
        .node_id
        .as_deref()
        .or(node.persistent_id.as_deref())
        .or(node.node_path.as_deref())
        .unwrap_or(&node.node_type);
    format!("node:{prefix}:{}", sanitize_id(identity))
}

fn ingredient_node_id(ingredient: &IngredientSnapshot, index: usize) -> String {
    let identity = ingredient
        .content_digest
        .as_deref()
        .or(ingredient.label.as_deref())
        .or(ingredient.title.as_deref())
        .unwrap_or("ingredient");
    format!("ingredient:{index}:{}", sanitize_id(identity))
}

fn sanitize_id(value: &str) -> String {
    let cleaned = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, ':' | '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if cleaned.is_empty() {
        "unknown".to_string()
    } else {
        cleaned
    }
}

fn ingredient_edge_kind(relationship: IngredientRelationship) -> GraphEdgeKind {
    match relationship {
        IngredientRelationship::ParentOf => GraphEdgeKind::ConvertedInto,
        IngredientRelationship::InputTo => GraphEdgeKind::UsedBy,
        IngredientRelationship::ComponentOf => GraphEdgeKind::PartOf,
    }
}

fn ingredient_relationship_name(relationship: IngredientRelationship) -> &'static str {
    match relationship {
        IngredientRelationship::ParentOf => "parentOf",
        IngredientRelationship::InputTo => "inputTo",
        IngredientRelationship::ComponentOf => "componentOf",
    }
}

fn format_source_range(range: &SourceRangeSnapshot) -> String {
    format!(
        "{}:{}-{}:{}",
        range.start_line, range.start_column, range.end_line, range.end_column
    )
}

fn read_identifiers(value: Option<&Value>, metadata: &mut GraphMetadata) {
    for identifier in value.and_then(Value::as_array).into_iter().flatten() {
        let Some(key) = identifier
            .get("propertyId")
            .or_else(|| identifier.get("propertyID"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let value = identifier.get("value").unwrap_or(identifier);
        match key {
            PROP_ASSET_TYPE => metadata.asset_type = value_string(value),
            PROP_ASSET_ROLE => metadata.asset_role = value_string(value),
            PROP_ASSET_TITLE => metadata.asset_title = value_string(value),
            PROP_ASSET_LABEL => metadata.asset_label = value_string(value),
            PROP_MEDIA_TYPE => metadata.media_type = value_string(value),
            PROP_PRODUCER_NAME => metadata.producer_name = value_string(value),
            PROP_PRODUCER_VERSION => metadata.producer_version = value_string(value),
            PROP_PRODUCER_CODEC => metadata.producer_codec = value_string(value),
            PROP_PRODUCER_RENDERER => metadata.producer_renderer = value_string(value),
            PROP_ACTIVITY_NAME => metadata.activity_name = value_string(value),
            PROP_ACTIVITY_STARTED_AT => metadata.activity_started_at = value_string(value),
            PROP_ACTIVITY_ENDED_AT => metadata.activity_ended_at = value_string(value),
            PROP_SOURCE_REPOSITORY => metadata.source_repository = value_string(value),
            PROP_SOURCE_COMMIT => metadata.source_commit = value_string(value),
            PROP_SOURCE_PATH => metadata.source_path = value_string(value),
            PROP_SOURCE_DIRTY => metadata.source_dirty = value_bool(value),
            PROP_SOURCE_RANGE => metadata.source_range = value_string(value),
            PROP_REDACTION_COUNT => metadata.redaction_count = value_u32(value),
            PROP_NODE_TYPE => metadata.executed_node_type = value_string(value),
            PROP_NODE_ID => metadata.executed_node_id = value_string(value),
            PROP_PERSISTENT_ID => metadata.executed_persistent_id = value_string(value),
            PROP_PROGRAMMING_LANGUAGE => metadata.programming_language = value_string(value),
            PROP_EXECUTION => metadata.execution = Some(value.clone()),
            PROP_AI_STANDARD_ASSERTION => metadata.ai_standard_assertion = value_string(value),
            _ => {}
        }
    }
}

fn evidence_values(edge: &Value) -> Vec<&Value> {
    match edge.get("evidence") {
        Some(Value::Array(values)) => values.iter().collect(),
        Some(value) => vec![value],
        None => Vec::new(),
    }
}

fn value_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
}

fn value_bool(value: &Value) -> Option<bool> {
    value.as_bool()
}

fn value_u32(value: &Value) -> Option<u32> {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| value.as_i64().and_then(|value| u32::try_from(value).ok()))
}
