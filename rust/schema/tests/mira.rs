use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use eyre::{Result, WrapErr, bail, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const MIRA_CONTEXT: &str = "https://purl.org/mira-science/mira.jsonld";
const MIRA_CONTEXT_SHA256: &str =
    "132e5a72aa14c017193ca93d16d905fb0827ab1c0e72ecaddb5369c9a200f2a5";

#[derive(Debug, Deserialize)]
struct EnumerationSchema {
    #[serde(rename = "anyOf")]
    variants: Vec<EnumerationVariant>,
}

#[derive(Debug, Deserialize)]
struct EnumerationVariant {
    #[serde(rename = "const")]
    name: String,
    #[serde(rename = "@id")]
    iri: String,
}

#[derive(Debug, Deserialize)]
struct NodeSchema {
    #[serde(default)]
    analogues: Vec<Analogue>,
}

#[derive(Debug, Deserialize)]
struct Analogue {
    id: String,
}

#[derive(Debug, Deserialize)]
struct VocabularyFixture {
    source: VocabularySource,
    classes: BTreeMap<String, String>,
    properties: BTreeMap<String, String>,
    #[serde(rename = "relationRepresentation")]
    relation_representation: String,
    relations: Vec<RelationFixture>,
}

#[derive(Debug, Deserialize)]
struct VocabularySource {
    context: String,
    #[serde(rename = "contextSha256")]
    context_sha256: String,
    namespace: String,
    revision: String,
}

#[derive(Debug, Deserialize)]
struct RelationFixture {
    stencila: String,
    term: String,
    curie: String,
    iri: String,
}

#[test]
fn relation_mappings_match_pinned_mira_vocabulary() -> Result<()> {
    let fixture: VocabularyFixture = read_json_fixture("vocabulary.json")?;
    assert_eq!(fixture.source.context, MIRA_CONTEXT);
    assert_eq!(fixture.source.context_sha256, MIRA_CONTEXT_SHA256);
    ensure!(
        fixture.source.revision.len() == 40
            && fixture
                .source
                .revision
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
        "the MIRA source revision should be a full Git commit hash"
    );
    let namespace = fixture.source.namespace.as_str();

    let context_contents = read_fixture("context.jsonld")?;
    let context_sha256 = Sha256::digest(context_contents.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(context_sha256, MIRA_CONTEXT_SHA256);
    let context: Value = serde_json::from_str(&context_contents)?;
    let context = context
        .get("@context")
        .and_then(Value::as_object)
        .ok_or_else(|| eyre::eyre!("pinned MIRA context should contain an object"))?;
    ensure!(
        !context.contains_key("isContainedBy"),
        "the pinned context deliberately does not define isContainedBy"
    );

    for (class, analogue) in &fixture.classes {
        assert_eq!(analogue, &format!("mira:{class}"));
        assert_eq!(context_iri(context, class)?, format!("{namespace}{class}"));
        let schema = read_node_schema(&format!("{class}.yaml"))?;
        ensure!(
            schema
                .analogues
                .iter()
                .any(|candidate| &candidate.id == analogue),
            "{class} should declare {analogue} as an analogue"
        );
    }

    assert_eq!(
        fixture.properties,
        BTreeMap::from([
            (
                "content".to_string(),
                "http://rdfs.org/sioc/ns#content".to_string(),
            ),
            (
                "description".to_string(),
                "http://purl.org/dc/terms/description".to_string(),
            ),
            (
                "format".to_string(),
                "http://purl.org/dc/terms/format".to_string(),
            ),
            (
                "has_container".to_string(),
                "http://rdfs.org/sioc/ns#has_container".to_string(),
            ),
            (
                "title".to_string(),
                "http://purl.org/dc/terms/title".to_string(),
            ),
        ])
    );
    for (term, iri) in &fixture.properties {
        assert_eq!(context_iri(context, term)?, *iri);
    }

    let expected = fixture
        .relations
        .iter()
        .map(|relation| (relation.stencila.clone(), relation.curie.clone()))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        expected.len() == 11,
        "the pinned MIRA fixture should contain all eleven relations"
    );

    for relation in &fixture.relations {
        assert_eq!(relation.curie, format!("mira:{}", relation.term));
        assert_eq!(relation.iri, format!("{namespace}{}", relation.term));
        assert_eq!(context_iri(context, &relation.term)?, relation.iri);
    }

    let research_relations = read_enumeration_schema("ResearchObjectRelationKind.yaml")?;
    assert_eq!(variant_map(research_relations), expected);

    let graph_relations = read_enumeration_schema("GraphEdgeKind.yaml")?
        .into_iter()
        .filter(|variant| expected.contains_key(&variant.name))
        .collect();
    assert_eq!(variant_map(graph_relations), expected);

    Ok(())
}

#[test]
fn representative_jsonld_fixtures_cover_the_interchange_contract() -> Result<()> {
    let vocabulary: VocabularyFixture = read_json_fixture("vocabulary.json")?;
    assert_eq!(vocabulary.relation_representation, "standalone-objects");

    let standalone = read_json_value("standalone-document.jsonld")?;
    let workspace = read_json_value("repository-workspace.jsonld")?;
    ensure_context(&standalone)?;
    ensure_context(&workspace)?;

    let standalone_items = graph_items(&standalone)?;
    let workspace_items = graph_items(&workspace)?;

    let supported_classes = standalone_items
        .iter()
        .flat_map(item_types)
        .filter(|node_type| vocabulary.classes.values().any(|class| class == *node_type))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        supported_classes,
        vocabulary.classes.values().map(String::as_str).collect()
    );

    let relation_terms = vocabulary
        .relations
        .iter()
        .map(|relation| relation.curie.as_str())
        .collect::<BTreeSet<_>>();
    let fixture_relation_terms = standalone_items
        .iter()
        .flat_map(item_types)
        .filter(|node_type| relation_terms.contains(node_type))
        .collect::<BTreeSet<_>>();
    assert_eq!(fixture_relation_terms, relation_terms);

    for relation in standalone_items
        .iter()
        .chain(workspace_items.iter())
        .filter(|item| {
            item_types(item)
                .iter()
                .any(|node_type| relation_terms.contains(node_type))
        })
    {
        let relation_id = relation
            .get("@id")
            .and_then(Value::as_str)
            .ok_or_else(|| eyre::eyre!("every MIRA relation fixture should have a string @id"))?;
        ensure!(
            relation.get("source").and_then(Value::as_str).is_some(),
            "MIRA relation `{relation_id}` should have a string source"
        );
        ensure!(
            relation
                .get("destination")
                .and_then(Value::as_str)
                .is_some(),
            "MIRA relation `{relation_id}` should have a string destination"
        );
    }

    ensure!(
        standalone_items.iter().any(|item| {
            item.get("@id")
                .and_then(Value::as_str)
                .is_some_and(|id| id.starts_with('#'))
        }),
        "the standalone fixture should contain a local fragment @id"
    );
    ensure!(
        workspace_items.iter().any(|item| {
            item.get("@id")
                .and_then(Value::as_str)
                .is_some_and(|id| id.starts_with("https://"))
        }),
        "the workspace fixture should contain an absolute HTTPS @id"
    );
    ensure!(
        standalone_items
            .iter()
            .chain(workspace_items.iter())
            .any(|item| { item.get("has_container").and_then(Value::as_str).is_some() }),
        "the MIRA fixtures should contain a string has_container reference"
    );

    let rich_description = standalone_items
        .iter()
        .find_map(|item| item.get("description").and_then(Value::as_object))
        .ok_or_else(|| eyre::eyre!("standalone fixture should contain a rich description"))?;
    assert_eq!(
        rich_description.get("format").and_then(Value::as_str),
        Some("application/vnd.oxa+json")
    );
    let oxa = rich_description
        .get("content")
        .and_then(Value::as_str)
        .ok_or_else(|| eyre::eyre!("rich description should contain serialized OXA JSON"))?;
    let oxa: Value = serde_json::from_str(oxa)?;
    assert_eq!(oxa.get("type").and_then(Value::as_str), Some("Document"));

    Ok(())
}

fn read_enumeration_schema(name: &str) -> Result<Vec<EnumerationVariant>> {
    let path = schema_dir().join(name);
    let contents =
        fs::read_to_string(&path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    Ok(serde_yaml::from_str::<EnumerationSchema>(&contents)?.variants)
}

fn read_node_schema(name: &str) -> Result<NodeSchema> {
    let path = schema_dir().join(name);
    let contents =
        fs::read_to_string(&path).wrap_err_with(|| format!("unable to read {}", path.display()))?;
    Ok(serde_yaml::from_str(&contents)?)
}

fn variant_map(variants: Vec<EnumerationVariant>) -> BTreeMap<String, String> {
    variants
        .into_iter()
        .map(|variant| (variant.name, variant.iri))
        .collect()
}

fn read_json_fixture<T>(name: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = read_fixture(name)?;
    Ok(serde_json::from_str(&contents)?)
}

fn read_fixture(name: &str) -> Result<String> {
    let path = fixture_dir().join(name);
    fs::read_to_string(&path).wrap_err_with(|| format!("unable to read {}", path.display()))
}

fn read_json_value(name: &str) -> Result<Value> {
    read_json_fixture(name)
}

fn ensure_context(value: &Value) -> Result<()> {
    let Some(context) = value.get("@context") else {
        bail!("MIRA fixture should declare an @context")
    };

    let has_mira_context = match context {
        Value::String(context) => context == MIRA_CONTEXT,
        Value::Array(contexts) => contexts.iter().any(|context| context == MIRA_CONTEXT),
        _ => false,
    };
    ensure!(
        has_mira_context,
        "MIRA fixture should use the pinned context"
    );
    Ok(())
}

fn graph_items(value: &Value) -> Result<&[Value]> {
    value
        .get("@graph")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| eyre::eyre!("MIRA fixture should contain an @graph array"))
}

fn item_types(item: &Value) -> Vec<&str> {
    match item.get("@type") {
        Some(Value::String(node_type)) => vec![node_type],
        Some(Value::Array(node_types)) => node_types.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn context_iri(context: &serde_json::Map<String, Value>, term: &str) -> Result<String> {
    let definition = context
        .get(term)
        .ok_or_else(|| eyre::eyre!("pinned MIRA context is missing `{term}`"))?;
    let compact_iri = match definition {
        Value::String(value) => value.as_str(),
        Value::Object(value) => value
            .get("@id")
            .and_then(Value::as_str)
            .ok_or_else(|| eyre::eyre!("context term `{term}` has no @id"))?,
        _ => bail!("context term `{term}` has an unsupported definition"),
    };
    if compact_iri.starts_with("http://") || compact_iri.starts_with("https://") {
        return Ok(compact_iri.to_string());
    }

    let (prefix, suffix) = compact_iri
        .split_once(':')
        .ok_or_else(|| eyre::eyre!("context term `{term}` is not an IRI or compact IRI"))?;
    let namespace = context
        .get(prefix)
        .and_then(Value::as_str)
        .ok_or_else(|| eyre::eyre!("context term `{term}` uses unknown prefix `{prefix}`"))?;
    Ok(format!("{namespace}{suffix}"))
}

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mira")
}

fn schema_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schema")
}
