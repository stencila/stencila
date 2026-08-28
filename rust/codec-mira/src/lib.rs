//! Encode and decode Stencila discourse graphs as MIRA JSON-LD.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use percent_encoding::percent_decode_str;
use serde_json::{Map, Value, json};
use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, Losses, async_trait,
    eyre::{Result, bail, ensure},
    stencila_format::Format,
    stencila_schema::{
        Article, Block, Claim, CreativeWork, Evidence, Graph, GraphEdge, GraphEdgeKind, GraphNode,
        Inline, Node, NodeType, Paragraph, Protocol, Question, RawBlock, Request,
        ResearchObjectRelationKind, Text,
    },
};
use stencila_codec_text_trait::to_text;

/// The immutable MIRA context targeted by this codec.
pub const MIRA_CONTEXT: &str = "https://purl.org/mira-science/mira.jsonld";

const MIRA_NAMESPACE: &str = "http://purl.org/mira-science/mira#";
const HAS_CONTAINER: &str = "has_container";

/// A codec for MIRA JSON-LD.
pub struct MiraCodec;

#[async_trait]
impl Codec for MiraCodec {
    fn name(&self) -> &str {
        "mira"
    }

    fn supports_from_format(&self, format: &Format) -> bool {
        matches!(format, Format::MiraJsonLd)
    }

    fn supports_to_format(&self, format: &Format) -> bool {
        matches!(format, Format::MiraJsonLd)
    }

    async fn from_str(
        &self,
        content: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let (graph, losses) = mira_jsonld_to_graph_with_losses(content)?;
        Ok((
            Node::Graph(graph),
            DecodeInfo {
                losses,
                ..DecodeInfo::none()
            },
        ))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let Node::Graph(graph) = node else {
            bail!("MIRA JSON-LD export requires a Graph node")
        };

        let compact = options.as_ref().and_then(|options| options.compact);
        let (value, losses) = graph_to_mira_jsonld_with_losses(graph)?;
        let json = match compact {
            Some(true) => serde_json::to_string(&value)?,
            Some(false) | None => serde_json::to_string_pretty(&value)?,
        };

        Ok((
            json,
            EncodeInfo {
                losses,
                ..EncodeInfo::none()
            },
        ))
    }
}

/// Encode a Stencila graph into a MIRA-oriented JSON-LD object.
pub fn graph_to_mira_jsonld(graph: &Graph) -> Result<Value> {
    graph_to_mira_jsonld_with_losses(graph).map(|(value, _)| value)
}

/// Decode a MIRA JSON-LD document into a Stencila graph.
pub fn mira_jsonld_to_graph(content: &str) -> Result<Graph> {
    mira_jsonld_to_graph_with_losses(content).map(|(graph, _)| graph)
}

fn mira_jsonld_to_graph_with_losses(content: &str) -> Result<(Graph, Losses)> {
    let root: Value = serde_json::from_str(content)?;
    let root = root
        .as_object()
        .ok_or_else(|| stencila_codec::eyre::eyre!("MIRA JSON-LD root should be an object"))?;
    ensure_mira_context(root.get("@context"))?;
    let items = root
        .get("@graph")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            stencila_codec::eyre::eyre!("MIRA JSON-LD should contain an @graph array")
        })?;
    let subject = root
        .get("@id")
        .and_then(Value::as_str)
        .unwrap_or("mira:graph")
        .to_string();

    let mut losses = Losses::none();
    let mut nodes = BTreeMap::<String, GraphNode>::new();
    for item in items {
        let Some(item) = item.as_object() else {
            losses.add("MiraJsonLd.graphItem");
            continue;
        };
        let Some(id) = item.get("@id").and_then(Value::as_str) else {
            losses.add("MiraJsonLd.graphItem.@id");
            continue;
        };
        if relation_kind_from_item(item).is_some() {
            continue;
        }

        let node = mira_item_to_node(item, id, &mut losses)?;
        if nodes
            .insert(
                id.to_string(),
                GraphNode::new(id.to_string(), Box::new(node)),
            )
            .is_some()
        {
            losses.add("MiraJsonLd.duplicateId");
        }
    }

    let mut edges = Vec::new();
    let mut edge_keys = BTreeSet::new();
    for item in items.iter().filter_map(Value::as_object) {
        let Some(id) = item.get("@id").and_then(Value::as_str) else {
            continue;
        };

        if let Some(kind) = relation_kind_from_item(item) {
            let Some(source) = item.get("source").and_then(iri_value) else {
                losses.add("MiraJsonLd.relation.source");
                continue;
            };
            let Some(target) = item.get("destination").and_then(iri_value) else {
                losses.add("MiraJsonLd.relation.destination");
                continue;
            };
            ensure_mira_graph_node(&mut nodes, source);
            ensure_mira_graph_node(&mut nodes, target);
            let key = (
                source.to_string(),
                target.to_string(),
                GraphEdgeKind::from(kind),
            );
            if edge_keys.insert(key.clone()) {
                let mut edge = GraphEdge::new(key.0, key.1, key.2);
                edge.id = Some(id.to_string());
                edges.push(edge);
            } else {
                losses.add("MiraJsonLd.duplicateRelation");
            }
            continue;
        }

        if let Some(container) = item.get(HAS_CONTAINER).and_then(iri_value) {
            ensure_mira_graph_node(&mut nodes, id);
            ensure_mira_graph_node(&mut nodes, container);
            let key = (id.to_string(), container.to_string(), GraphEdgeKind::PartOf);
            if edge_keys.insert(key.clone()) {
                edges.push(GraphEdge::new(key.0, key.1, key.2));
            }
        }
    }

    Ok((
        Graph::new(subject, nodes.into_values().collect(), edges),
        losses,
    ))
}

fn ensure_mira_context(context: Option<&Value>) -> Result<()> {
    let contains_mira = match context {
        Some(Value::String(context)) => context == MIRA_CONTEXT,
        Some(Value::Array(contexts)) => contexts.iter().any(|context| context == MIRA_CONTEXT),
        _ => false,
    };
    ensure!(
        contains_mira,
        "MIRA JSON-LD should use the pinned MIRA context"
    );
    Ok(())
}

fn mira_item_to_node(item: &Map<String, Value>, id: &str, losses: &mut Losses) -> Result<Node> {
    let content = mira_item_content(item, losses)?;
    let authored_id = if id.starts_with('#') {
        id.trim_start_matches('#').to_string()
    } else {
        id.to_string()
    };
    let label = item
        .get("label")
        .and_then(Value::as_str)
        .map(str::to_string);
    let title = item
        .get("title")
        .and_then(Value::as_str)
        .map(|title| vec![Inline::Text(Text::new(title.into()))]);

    macro_rules! research_node {
        ($type:ident, $variant:ident) => {{
            let mut node = $type::new(content);
            node.id = Some(authored_id);
            node.label = label;
            node.options.title = title;
            Node::$variant(node)
        }};
    }

    Ok(match research_type_from_item(item) {
        Some(NodeType::Claim) => research_node!(Claim, Claim),
        Some(NodeType::Evidence) => research_node!(Evidence, Evidence),
        Some(NodeType::Protocol) => research_node!(Protocol, Protocol),
        Some(NodeType::Question) => research_node!(Question, Question),
        Some(NodeType::Request) => research_node!(Request, Request),
        _ => {
            if !content.is_empty() {
                losses.add("MiraJsonLd.Item.description");
            }
            let mut work = CreativeWork::new();
            work.id = Some(authored_id);
            if is_absolute_iri(id) {
                work.options.url = Some(id.to_string());
            }
            work.options.name = item
                .get("name")
                .or_else(|| item.get("title"))
                .and_then(Value::as_str)
                .map(str::to_string);
            Node::CreativeWork(work)
        }
    })
}

fn mira_item_content(item: &Map<String, Value>, losses: &mut Losses) -> Result<Vec<Block>> {
    let Some(description) = item.get("description") else {
        return Ok(Vec::new());
    };
    if let Some(description) = description.as_str() {
        return Ok(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::new(description.into()),
        )]))]);
    }

    let Some(description) = description.as_object() else {
        losses.add("MiraJsonLd.description");
        return Ok(Vec::new());
    };
    let Some(content) = description.get("content").and_then(Value::as_str) else {
        losses.add("MiraJsonLd.description.content");
        return Ok(Vec::new());
    };
    let format = description
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or("text/plain");

    if format == Format::OxaJson.media_type() {
        let (node, info) = stencila_codec_oxa::decode(
            content,
            Some(DecodeOptions {
                format: Some(Format::OxaJson),
                ..Default::default()
            }),
        )?;
        losses.merge(info.losses);
        let Node::Article(article) = node else {
            bail!("OXA description should decode to an Article")
        };
        return Ok(article.content);
    }

    let format = Format::from_media_type(format)
        .map(|format| format.to_string())
        .unwrap_or_else(|_| format.to_string());
    Ok(vec![Block::RawBlock(RawBlock::new(format, content.into()))])
}

fn ensure_mira_graph_node(nodes: &mut BTreeMap<String, GraphNode>, id: &str) {
    nodes.entry(id.to_string()).or_insert_with(|| {
        let mut work = CreativeWork::new();
        work.id = Some(id.trim_start_matches('#').to_string());
        if is_absolute_iri(id) {
            work.options.url = Some(id.to_string());
        }
        GraphNode::new(id.to_string(), Box::new(Node::CreativeWork(work)))
    });
}

fn iri_value(value: &Value) -> Option<&str> {
    value
        .as_str()
        .or_else(|| value.get("@id").and_then(Value::as_str))
}

fn item_type_names(item: &Map<String, Value>) -> Vec<&str> {
    match item.get("@type") {
        Some(Value::String(value)) => vec![value],
        Some(Value::Array(values)) => values.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn mira_local_name(value: &str) -> Option<&str> {
    value
        .strip_prefix("mira:")
        .or_else(|| value.strip_prefix(MIRA_NAMESPACE))
        .or_else(|| (!value.contains(':')).then_some(value))
}

fn research_type_from_item(item: &Map<String, Value>) -> Option<NodeType> {
    item_type_names(item)
        .into_iter()
        .find_map(|name| match mira_local_name(name)? {
            "Claim" => Some(NodeType::Claim),
            "Evidence" => Some(NodeType::Evidence),
            "Protocol" => Some(NodeType::Protocol),
            "Question" => Some(NodeType::Question),
            "Request" => Some(NodeType::Request),
            _ => None,
        })
}

fn relation_kind_from_item(item: &Map<String, Value>) -> Option<ResearchObjectRelationKind> {
    item_type_names(item).into_iter().find_map(|name| {
        let name = mira_local_name(name)?;
        ResearchObjectRelationKind::all()
            .iter()
            .copied()
            .find(|kind| kind.mira_name() == name)
    })
}

fn graph_to_mira_jsonld_with_losses(graph: &Graph) -> Result<(Value, Losses)> {
    let root_id = root_item_id(graph);
    let id_map = graph
        .nodes
        .iter()
        .map(|node| {
            let document_id = document_item_id_for_node(graph, node);
            (node.id.as_str(), mira_node_id(node, document_id.as_deref()))
        })
        .collect::<BTreeMap<_, _>>();

    let mut losses = Losses::none();
    let mut objects = BTreeMap::<String, Map<String, Value>>::new();
    for node in &graph.nodes {
        if let Some(object) = graph_node_to_object(node, &id_map, &mut losses)? {
            objects.insert(node.id.clone(), object);
        }
    }

    let mut relation_edges = Vec::new();
    for edge in &graph.edges {
        match edge.kind {
            GraphEdgeKind::PartOf => {
                if !objects.contains_key(edge.target.as_str()) {
                    losses.add("GraphEdge.PartOf");
                    continue;
                }
                let target = edge_target_id(&edge.target, &id_map);
                if let Some(source) = objects.get_mut(&edge.source) {
                    set_has_container(source, target);
                } else {
                    losses.add("GraphEdge.PartOf");
                }
            }
            kind => {
                let Some(relation) = mira_relation_name(kind) else {
                    losses.add(format!("GraphEdge.{kind}"));
                    continue;
                };
                if let Some(object) =
                    relation_edge_object(edge, relation, &id_map, &objects, root_id.as_deref())
                {
                    relation_edges.push(object);
                } else {
                    losses.add(format!("GraphEdge.{kind}"));
                }
            }
        }
    }

    let default_container = repository_item_id(graph).or_else(|| root_id.clone());
    if let Some(container_id) = default_container.as_deref() {
        for node in &graph.nodes {
            if node.node.as_research_object().is_some()
                && let Some(object) = objects.get_mut(&node.id)
            {
                set_default_has_container(object, container_id.to_string());
            }
        }
    }

    let graph_items = source_items(graph)
        .into_iter()
        .map(Value::Object)
        .chain(objects.into_values().map(Value::Object))
        .chain(relation_edges.into_iter().map(Value::Object))
        .collect::<Vec<_>>();

    let mut root = Map::new();
    root.insert(
        "@context".to_string(),
        mira_context(graph, root_id.as_deref()),
    );
    if let Some(root_id) = root_id {
        root.insert("@id".to_string(), Value::String(root_id));
    }
    root.insert("@graph".to_string(), Value::Array(graph_items));

    Ok((Value::Object(root), losses))
}

fn graph_node_to_object(
    graph_node: &GraphNode,
    id_map: &BTreeMap<&str, String>,
    losses: &mut Losses,
) -> Result<Option<Map<String, Value>>> {
    let id = id_map
        .get(graph_node.id.as_str())
        .cloned()
        .unwrap_or_else(|| graph_node.id.clone());

    if let Some(research) = graph_node.node.as_research_object() {
        let Some(node_type) = mira_research_type(research.kind()) else {
            losses.add(format!("GraphNode.{}", research.kind()));
            return Ok(None);
        };
        let mut object =
            research_object(id, node_type, research.content(), research.title(), losses)?;
        if let Some(label) = research.label() {
            object.insert("label".to_string(), Value::String(label.to_string()));
        }
        if let Node::Claim(node) = graph_node.node.as_ref()
            && node.claim_type.is_some()
        {
            losses.add("Claim.claimType");
        }
        record_extra_losses(research.extra(), &research.kind().to_string(), losses);
        return Ok(Some(object));
    }

    match graph_node.node.as_ref() {
        Node::CreativeWork(node) => {
            let mut object = base_object(id.clone(), "Item");
            if node.options.url.as_ref().is_some_and(|url| url != &id) {
                losses.add("CreativeWork.url");
            }
            if let Some(name) = &node.options.name {
                object.insert("name".to_string(), Value::String(name.clone()));
            }
            Ok(Some(object))
        }
        _ => {
            losses.add(format!("GraphNode.{}", graph_node.node.node_type()));
            Ok(None)
        }
    }
}

fn mira_research_type(kind: NodeType) -> Option<&'static str> {
    Some(match kind {
        NodeType::Claim => "mira:Claim",
        NodeType::Evidence => "mira:Evidence",
        NodeType::Protocol => "mira:Protocol",
        NodeType::Question => "mira:Question",
        NodeType::Request => "mira:Request",
        _ => return None,
    })
}

fn record_extra_losses(
    extra: Option<&stencila_codec::stencila_schema::Object>,
    type_name: &str,
    losses: &mut Losses,
) {
    if let Some(extra) = extra {
        for key in extra.keys() {
            if ResearchObjectRelationKind::from_authored_key(key).is_none() {
                losses.add(format!("{type_name}.extra.{key}"));
            }
        }
    }
}

fn research_object(
    id: String,
    node_type: &str,
    content: &[Block],
    title: Option<&[Inline]>,
    losses: &mut Losses,
) -> Result<Map<String, Value>> {
    let mut object = base_object(id, node_type);

    if let Some(title) = title_text(title, content) {
        object.insert("title".to_string(), Value::String(title));
    }

    if !content.is_empty() {
        let (content, content_losses) = oxa_content(content)?;
        losses.merge(content_losses);
        object.insert(
            "description".to_string(),
            json!({
                "@type": "Item",
                "content": content,
                "format": Format::OxaJson.media_type(),
            }),
        );
    }

    Ok(object)
}

fn title_text(title: Option<&[Inline]>, content: &[Block]) -> Option<String> {
    title
        .and_then(|title| {
            let text = title.iter().fold(String::new(), |mut text, inline| {
                text.push_str(&to_text(inline));
                text
            });
            non_empty_title(text)
        })
        .or_else(|| title_from_content(content))
}

fn title_from_content(content: &[Block]) -> Option<String> {
    match content.first() {
        Some(Block::Heading(heading)) => non_empty_title(to_text(&heading.content)),
        Some(..) => non_empty_title(first_sentence(&to_text(&content.to_vec()))),
        None => None,
    }
}

fn first_sentence(text: &str) -> String {
    let mut chars = text.char_indices().peekable();
    while let Some((index, character)) = chars.next() {
        if matches!(character, '.' | '!' | '?') {
            match chars.peek() {
                Some((_, next)) if next.is_whitespace() => return text[..=index].to_string(),
                None => return text[..=index].to_string(),
                Some(..) => {}
            }
        }
    }
    text.to_string()
}

fn non_empty_title(text: String) -> Option<String> {
    let title = text.trim().to_string();
    (!title.is_empty()).then_some(title)
}

fn oxa_content(content: &[Block]) -> Result<(String, Losses)> {
    let article = Node::Article(Article::new(content.to_vec()));
    let (json, info) = stencila_codec_oxa::encode(
        &article,
        Some(EncodeOptions {
            compact: Some(true),
            format: Some(Format::OxaJson),
            ..Default::default()
        }),
    )?;
    Ok((json, info.losses))
}

fn base_object(id: String, node_type: &str) -> Map<String, Value> {
    Map::from_iter([
        ("@id".to_string(), Value::String(id)),
        ("@type".to_string(), Value::String(node_type.to_string())),
    ])
}

fn repository_item(graph: &Graph) -> Option<Map<String, Value>> {
    let repository = repository_item_id(graph)?;
    Some(Map::from_iter([
        ("@id".to_string(), Value::String(repository)),
        ("@type".to_string(), Value::String("Container".to_string())),
    ]))
}

fn source_item(id: String, format: String, container: String) -> Map<String, Value> {
    let mut item = Map::from_iter([
        ("@id".to_string(), Value::String(id)),
        ("@type".to_string(), Value::String("Item".to_string())),
        ("format".to_string(), Value::String(format)),
    ]);
    set_has_container(&mut item, container);
    item
}

fn source_items(graph: &Graph) -> Vec<Map<String, Value>> {
    let mut items = BTreeMap::new();

    if let Some(repository) = repository_item(graph) {
        insert_source_item(&mut items, repository);

        if let Some(directory) = directory_item(graph) {
            insert_source_item(&mut items, directory);
        }

        for document in document_items(graph) {
            insert_source_item(&mut items, document);
        }
    } else if let Some(root_id) = root_item_id(graph) {
        insert_source_item(
            &mut items,
            Map::from_iter([
                ("@id".to_string(), Value::String(root_id)),
                ("@type".to_string(), Value::String("Container".to_string())),
            ]),
        );
    }

    items.into_values().collect()
}

fn insert_source_item(items: &mut BTreeMap<String, Map<String, Value>>, item: Map<String, Value>) {
    if let Some(id) = item.get("@id").and_then(Value::as_str) {
        items.insert(id.to_string(), item);
    }
}

fn directory_item(graph: &Graph) -> Option<Map<String, Value>> {
    if !is_directory_graph(graph) {
        return None;
    }
    let repository = repository_item_id(graph)?;
    let path = graph.options.path.as_deref().and_then(source_item_path)?;
    let id = repository_path_id(graph, &path, RepositoryEntryKind::Directory)?;
    Some(source_item(id, "inode/directory".to_string(), repository))
}

fn document_items(graph: &Graph) -> Vec<Map<String, Value>> {
    let Some(repository) = repository_item_id(graph) else {
        return Vec::new();
    };
    let mut paths = BTreeSet::new();

    if let Some(path) = source_document_item_path(graph) {
        paths.insert(path);
    }
    for node in &graph.nodes {
        if node.node.as_research_object().is_some()
            && let Some(path) = document_item_path_for_node(graph, node)
        {
            paths.insert(path);
        }
    }

    paths
        .into_iter()
        .filter_map(|path| {
            let id = repository_path_id(graph, &path, RepositoryEntryKind::Document)?;
            let media_type = Format::from_path(Path::new(&path)).media_type();
            Some(source_item(id, media_type, repository.clone()))
        })
        .collect()
}

fn repository_item_id(graph: &Graph) -> Option<String> {
    let repository = graph.options.repository.as_deref()?.trim_end_matches('/');
    (!repository.is_empty()).then(|| repository.to_string())
}

fn root_item_id(graph: &Graph) -> Option<String> {
    document_item_id(graph)
        .or_else(|| directory_item_id(graph))
        .or_else(|| (!graph.subject.trim().is_empty()).then(|| graph.subject.clone()))
}

fn directory_item_id(graph: &Graph) -> Option<String> {
    let repository = repository_item_id(graph)?;
    let Some(path) = graph.options.path.as_deref().and_then(source_item_path) else {
        return Some(repository);
    };
    repository_path_id(graph, &path, RepositoryEntryKind::Directory)
}

fn document_item_id(graph: &Graph) -> Option<String> {
    let path = source_document_item_path(graph)?;
    repository_path_id(graph, &path, RepositoryEntryKind::Document)
}

fn document_item_id_for_node(graph: &Graph, node: &GraphNode) -> Option<String> {
    let path = document_item_path_for_node(graph, node)?;
    repository_path_id(graph, &path, RepositoryEntryKind::Document)
}

fn document_item_path_for_node(graph: &Graph, node: &GraphNode) -> Option<String> {
    if !is_directory_graph(graph)
        && let Some(path) = source_document_item_path(graph)
    {
        return Some(path);
    }
    graph_node_scope(&node.id).and_then(|scope| source_item_path(&scope))
}

fn graph_node_scope(id: &str) -> Option<String> {
    let scope = id.strip_prefix("node:")?.split_once('#')?.0;
    percent_decode_str(scope)
        .decode_utf8()
        .ok()
        .map(|scope| scope.into_owned())
}

#[derive(Debug, Clone, Copy)]
enum RepositoryEntryKind {
    Directory,
    Document,
}

fn repository_path_id(graph: &Graph, path: &str, kind: RepositoryEntryKind) -> Option<String> {
    let repository = repository_item_id(graph)?;
    let path = source_item_path(path)?;
    let commit = graph
        .options
        .commit
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if repository.starts_with("https://github.com/")
        && let Some(commit) = commit
    {
        let segment = match kind {
            RepositoryEntryKind::Directory => "tree",
            RepositoryEntryKind::Document => "blob",
        };
        Some(format!("{repository}/{segment}/{commit}/{path}"))
    } else {
        Some(format!("{repository}/{path}"))
    }
}

fn source_document_item_path(graph: &Graph) -> Option<String> {
    if is_directory_graph(graph) {
        return None;
    }
    source_item_path(graph.options.path.as_deref()?)
}

fn source_item_path(path: &str) -> Option<String> {
    let path = path
        .trim_start_matches("./")
        .trim_start_matches('/')
        .trim_end_matches('/');
    if path.is_empty() || path == "." {
        return None;
    }
    let file_name = path
        .rsplit_once('/')
        .map_or(path, |(.., file_name)| file_name);
    (!file_name.is_empty() && file_name != "." && file_name != "..").then(|| path.to_string())
}

fn is_directory_graph(graph: &Graph) -> bool {
    graph.options.path.as_deref().is_some_and(source_item_path_is_directory)
        || graph.nodes.iter().any(|node| {
            matches!(node.node.as_ref(), Node::Directory(directory) if directory.path == ".")
        })
}

fn source_item_path_is_directory(path: &str) -> bool {
    let path = path.trim_start_matches("./").trim_start_matches('/');
    path.is_empty() || path == "." || path.ends_with('/')
}

fn mira_node_id(node: &GraphNode, root_id: Option<&str>) -> String {
    let declared_id = match node.node.as_ref() {
        Node::Claim(node) => node.id.as_ref(),
        Node::Evidence(node) => node.id.as_ref(),
        Node::Protocol(node) => node.id.as_ref(),
        Node::Question(node) => node.id.as_ref(),
        Node::Request(node) => node.id.as_ref(),
        Node::CreativeWork(node) => node.options.url.as_ref(),
        _ => None,
    };

    if let Some(id) = declared_id.map(String::as_str)
        && is_absolute_iri(id)
    {
        return id.to_string();
    }

    let fragment = declared_id
        .map(String::as_str)
        .or_else(|| node.id.rsplit_once('#').map(|(.., fragment)| fragment))
        .map(|fragment| fragment.trim_start_matches('#'))
        .filter(|fragment| !fragment.is_empty());

    if node.node.as_research_object().is_some()
        && let Some(fragment) = fragment
    {
        return root_id.map_or_else(
            || format!("#{fragment}"),
            |root_id| format!("{root_id}#{fragment}"),
        );
    }

    declared_id.cloned().unwrap_or_else(|| node.id.clone())
}

fn edge_target_id(target: &str, id_map: &BTreeMap<&str, String>) -> String {
    id_map
        .get(target)
        .cloned()
        .unwrap_or_else(|| target.to_string())
}

fn mira_relation_name(kind: GraphEdgeKind) -> Option<&'static str> {
    ResearchObjectRelationKind::try_from(kind)
        .ok()
        .map(ResearchObjectRelationKind::mira_name)
}

fn relation_edge_object(
    edge: &GraphEdge,
    relation: &str,
    id_map: &BTreeMap<&str, String>,
    objects: &BTreeMap<String, Map<String, Value>>,
    root_id: Option<&str>,
) -> Option<Map<String, Value>> {
    let source_object = objects.get(&edge.source)?;
    let source = edge_target_id(&edge.source, id_map);
    let destination = edge_target_id(&edge.target, id_map);
    let source_title = object_title(source_object, &source);
    let destination_title = objects
        .get(&edge.target)
        .map(|object| object_title(object, &destination))
        .unwrap_or_else(|| destination.clone());
    Some(Map::from_iter([
        (
            "@id".to_string(),
            Value::String(relation_edge_id(
                edge,
                relation,
                &source,
                &destination,
                root_id,
            )),
        ),
        (
            "@type".to_string(),
            Value::String(format!("mira:{relation}")),
        ),
        ("source".to_string(), Value::String(source)),
        ("destination".to_string(), Value::String(destination)),
        (
            "title".to_string(),
            Value::String(format!("{source_title} -{relation}-> {destination_title}")),
        ),
    ]))
}

fn relation_edge_id(
    edge: &GraphEdge,
    relation: &str,
    source: &str,
    destination: &str,
    root_id: Option<&str>,
) -> String {
    if let Some(id) = edge
        .id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        if is_absolute_iri(id) || id.starts_with('#') {
            return id.to_string();
        }
        return relation_base_id(source, root_id)
            .map_or_else(|| format!("#{id}"), |base| format!("{base}#{id}"));
    }
    let fragment = format!(
        "rel_{:016x}",
        stable_relation_hash(source, relation, destination)
    );
    relation_base_id(source, root_id)
        .map_or_else(|| format!("#{fragment}"), |id| format!("{id}#{fragment}"))
}

fn relation_base_id<'a>(source: &'a str, fallback: Option<&'a str>) -> Option<&'a str> {
    if source.starts_with('#') {
        None
    } else {
        source
            .rsplit_once('#')
            .map(|(base, ..)| base)
            .filter(|base| !base.is_empty())
            .or(fallback)
    }
}

fn stable_relation_hash(source: &str, relation: &str, destination: &str) -> u64 {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;
    source
        .bytes()
        .chain([0xff])
        .chain(relation.bytes())
        .chain([0xff])
        .chain(destination.bytes())
        .fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
        })
}

fn object_title(object: &Map<String, Value>, fallback: &str) -> String {
    ["title", "name"]
        .into_iter()
        .find_map(|property| {
            object
                .get(property)
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(String::from)
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn set_has_container(object: &mut Map<String, Value>, target: String) {
    object.insert(HAS_CONTAINER.to_string(), Value::String(target));
}

fn set_default_has_container(object: &mut Map<String, Value>, target: String) {
    if !object.contains_key(HAS_CONTAINER) {
        set_has_container(object, target);
    }
}

fn mira_context(graph: &Graph, root_id: Option<&str>) -> Value {
    if repository_item_id(graph).is_none()
        && let Some(root_id) = root_id.filter(|id| is_absolute_iri(id))
    {
        json!([MIRA_CONTEXT, { "@base": root_id }])
    } else {
        Value::String(MIRA_CONTEXT.to_string())
    }
}

fn is_absolute_iri(value: &str) -> bool {
    value.split_once(':').is_some_and(|(scheme, rest)| {
        !scheme.is_empty()
            && !rest.is_empty()
            && scheme.chars().enumerate().all(|(index, character)| {
                character.is_ascii_alphabetic()
                    || (index > 0
                        && (character.is_ascii_digit() || matches!(character, '+' | '-' | '.')))
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_codec::stencila_schema::{
        Claim, Evidence, Paragraph, Protocol, Question, Request, ResearchObjectRelationKind, Text,
    };

    #[test]
    fn exports_all_research_types_and_relation_kinds() -> Result<()> {
        let mut claim = Claim::new(vec![paragraph("A claim.")]);
        claim.id = Some("claim-1".to_string());
        claim.label = Some("Claim A".to_string());
        claim.claim_type = Some(stencila_codec::stencila_schema::ClaimType::Statement);

        let mut evidence = Evidence::new(vec![paragraph("Evidence.")]);
        evidence.id = Some("evidence-1".to_string());
        evidence.label = Some("Evidence A".to_string());
        let mut question = Question::new(vec![paragraph("Question?")]);
        question.id = Some("question-1".to_string());
        question.label = Some("Question A".to_string());
        let mut protocol = Protocol::new(vec![paragraph("Protocol.")]);
        protocol.id = Some("protocol-1".to_string());
        protocol.label = Some("Protocol A".to_string());
        let mut request = Request::new(vec![paragraph("Request.")]);
        request.id = Some("request-1".to_string());
        request.label = Some("Request A".to_string());

        let nodes = [
            ("claim", Node::Claim(claim)),
            ("evidence", Node::Evidence(evidence)),
            ("question", Node::Question(question)),
            ("protocol", Node::Protocol(protocol)),
            ("request", Node::Request(request)),
        ]
        .into_iter()
        .map(|(id, node)| GraphNode::new(id.to_string(), Box::new(node)))
        .collect::<Vec<_>>();
        let edges = ResearchObjectRelationKind::all()
            .iter()
            .copied()
            .map(|kind| {
                GraphEdge::new(
                    "claim".to_string(),
                    "evidence".to_string(),
                    GraphEdgeKind::from(kind),
                )
            })
            .collect();
        let graph = Graph::new("https://example.org/research".to_string(), nodes, edges);

        let (value, losses) = graph_to_mira_jsonld_with_losses(&graph)?;
        assert!(losses.iter().any(|(label, _)| label == "Claim.claimType"));
        assert_eq!(
            value["@context"],
            json!([MIRA_CONTEXT, { "@base": "https://example.org/research" }])
        );
        assert_eq!(value["@id"], "https://example.org/research");
        let items = value["@graph"]
            .as_array()
            .ok_or_else(|| stencila_codec::eyre::eyre!("MIRA graph should be an array"))?;

        for (id, kind, label) in [
            ("#claim-1", "mira:Claim", "Claim A"),
            ("#evidence-1", "mira:Evidence", "Evidence A"),
            ("#question-1", "mira:Question", "Question A"),
            ("#protocol-1", "mira:Protocol", "Protocol A"),
            ("#request-1", "mira:Request", "Request A"),
        ] {
            let item = items
                .iter()
                .find(|item| item["@id"] == id)
                .ok_or_else(|| stencila_codec::eyre::eyre!("missing {id}"))?;
            assert_eq!(item["@type"], kind);
            assert_eq!(item["label"], label);
            assert!(item["description"]["content"].is_string());
        }

        let relation_types = items
            .iter()
            .filter_map(|item| item["@type"].as_str())
            .filter(|kind| {
                matches!(
                    *kind,
                    "mira:supports"
                        | "mira:supportedBy"
                        | "mira:opposes"
                        | "mira:opposedBy"
                        | "mira:addresses"
                        | "mira:addressedBy"
                        | "mira:follows"
                        | "mira:grounds"
                        | "mira:is_grounded_in"
                        | "mira:request_for"
                        | "mira:request_target"
                )
            })
            .count();
        assert_eq!(relation_types, 11);
        let claim = items
            .iter()
            .find(|item| item["@id"] == "#claim-1")
            .ok_or_else(|| stencila_codec::eyre::eyre!("missing claim"))?;
        assert!(claim.get("claimType").is_none());

        Ok(())
    }

    #[test]
    fn records_losses_for_unsupported_graph_nodes() -> Result<()> {
        let graph = Graph::new(
            "https://example.org/research".to_string(),
            vec![GraphNode::new(
                "article".to_string(),
                Box::new(Node::Article(Article::new(Vec::new()))),
            )],
            Vec::new(),
        );
        let (_value, losses) = graph_to_mira_jsonld_with_losses(&graph)?;
        assert!(losses.iter().any(|(label, _)| label == "GraphNode.Article"));
        Ok(())
    }

    #[test]
    fn exports_standalone_container_and_rich_description_contract() -> Result<()> {
        let mut claim = Claim::new(vec![paragraph("A supported claim.")]);
        claim.id = Some("claim-authored".to_string());
        let graph = Graph::new(
            "file:///workspace/report.smd".to_string(),
            vec![GraphNode::new(
                "node:report.smd#clm_claim-authored".to_string(),
                Box::new(Node::Claim(claim)),
            )],
            Vec::new(),
        );

        let value = graph_to_mira_jsonld(&graph)?;
        assert_eq!(
            value["@context"],
            json!([
                MIRA_CONTEXT,
                { "@base": "file:///workspace/report.smd" }
            ])
        );
        let items = value["@graph"]
            .as_array()
            .ok_or_else(|| stencila_codec::eyre::eyre!("MIRA graph should be an array"))?;
        assert!(items.iter().any(|item| {
            item["@id"] == "file:///workspace/report.smd" && item["@type"] == "Container"
        }));
        let claim = items
            .iter()
            .find(|item| item["@id"] == "#claim-authored")
            .ok_or_else(|| stencila_codec::eyre::eyre!("missing standalone claim"))?;
        assert_eq!(claim["has_container"], "file:///workspace/report.smd");
        assert!(claim.get("isContainedBy").is_none());
        assert_eq!(claim["description"]["@type"], "Item");
        Ok(())
    }

    #[test]
    fn exports_workspace_items_and_preserves_absolute_ids() -> Result<()> {
        let mut claim = Claim::new(vec![paragraph("A claim.")]);
        claim.id = Some("claim-authored".to_string());
        let mut evidence = Evidence::new(vec![paragraph("External evidence.")]);
        evidence.id = Some("https://example.org/evidence/external-1".to_string());
        let nodes = vec![
            GraphNode::new(
                "node:examples/report.smd#clm_claim-authored".to_string(),
                Box::new(Node::Claim(claim)),
            ),
            GraphNode::new(
                "node:examples/report.smd#evd_external".to_string(),
                Box::new(Node::Evidence(evidence)),
            ),
        ];
        let edges = vec![GraphEdge::new(
            "node:examples/report.smd#clm_claim-authored".to_string(),
            "node:examples/report.smd#evd_external".to_string(),
            GraphEdgeKind::SupportedBy,
        )];
        let mut graph = Graph::new("workspace:examples".to_string(), nodes, edges);
        graph.options.repository = Some("https://github.com/stencila/stencila".to_string());
        graph.options.path = Some("examples/".to_string());
        graph.options.commit = Some("main".to_string());

        let value = graph_to_mira_jsonld(&graph)?;
        assert_eq!(
            value["@id"],
            "https://github.com/stencila/stencila/tree/main/examples"
        );
        let items = value["@graph"]
            .as_array()
            .ok_or_else(|| stencila_codec::eyre::eyre!("MIRA graph should be an array"))?;
        assert!(items.iter().any(|item| {
            item["@id"] == "https://github.com/stencila/stencila/tree/main/examples"
                && item["@type"] == "Item"
                && item["format"] == "inode/directory"
        }));
        assert!(items.iter().any(|item| {
            item["@id"] == "https://github.com/stencila/stencila/blob/main/examples/report.smd"
                && item["@type"] == "Item"
        }));
        assert!(items.iter().any(|item| {
            item["@id"]
                == "https://github.com/stencila/stencila/blob/main/examples/report.smd#claim-authored"
        }));
        assert!(
            items
                .iter()
                .any(|item| { item["@id"] == "https://example.org/evidence/external-1" })
        );
        assert!(items.iter().any(|item| {
            item["@type"] == "mira:supportedBy"
                && item["destination"] == "https://example.org/evidence/external-1"
        }));
        Ok(())
    }

    #[test]
    fn decodes_pinned_standalone_interchange_fixture() -> Result<()> {
        let graph = mira_jsonld_to_graph(include_str!(
            "../../schema/tests/fixtures/mira/standalone-document.jsonld"
        ))?;

        for node_type in [
            NodeType::Claim,
            NodeType::Evidence,
            NodeType::Protocol,
            NodeType::Question,
            NodeType::Request,
        ] {
            assert!(
                graph
                    .nodes
                    .iter()
                    .any(|node| node.node.node_type() == node_type)
            );
        }
        assert_eq!(
            graph
                .edges
                .iter()
                .filter(|edge| ResearchObjectRelationKind::try_from(edge.kind).is_ok())
                .count(),
            11
        );
        let encoded = graph_to_mira_jsonld(&graph)?;
        let encoded = serde_json::to_string(&encoded)?;
        assert!(!encoded.contains("claimType"));
        assert!(!encoded.contains("schema:CreativeWork"));
        assert!(!encoded.contains("\"url\""));
        Ok(())
    }

    #[test]
    fn decodes_pinned_workspace_interchange_fixture() -> Result<()> {
        let graph = mira_jsonld_to_graph(include_str!(
            "../../schema/tests/fixtures/mira/repository-workspace.jsonld"
        ))?;

        assert_eq!(
            graph.subject,
            "https://github.com/stencila/stencila/tree/main/examples"
        );
        assert!(graph.edges.iter().any(|edge| {
            edge.kind == GraphEdgeKind::SupportedBy
                && edge.target == "https://example.org/evidence/external-1"
        }));
        Ok(())
    }

    fn paragraph(value: &str) -> Block {
        Block::Paragraph(Paragraph::new(vec![Inline::Text(Text::new(value.into()))]))
    }
}
