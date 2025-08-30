use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

use common::{
    eyre::{OptionExt, Result, bail},
    inflector::Inflector,
    itertools::Itertools,
    tokio::fs::write,
};

use crate::{
    schema::{Items, ItemsRef, ItemsType, Type},
    schemas::Schemas,
};

impl Schemas {
    /// Generate a Kuzu database schema from Stencila Schema
    ///
    /// For each node type in the Stencila Schema we create a node table with
    /// table properties for primitive properties (e.g. strings, numbers) e.g.
    ///
    /// CREATE NODE TABLE Paragraph (...)
    ///
    /// For entity type properties we create relationship tables which have `TO`
    /// and `FROM` pairs for every possible parent-child combination. e.g.
    ///
    /// CREATE REL TABLE content(FROM Article TO Paragraph, FROM Article TO CodeBlock, ..)
    ///
    /// It is currently necessary to create this all in one big schema file which includes
    /// all node and relationship tables. We can can't create these tables on demand
    /// until this is implemented https://github.com/kuzudb/kuzu/issues/5051.
    #[allow(clippy::print_stderr)]
    pub async fn kuzu(&self) -> Result<()> {
        eprintln!("Generating Kuzu Schema");

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node-db");

        // Skip properties that users are never likely to want to use in query filters
        let skip_props = [
            // General
            "type",
            "id",
            // Blanket exclusions, regardless of type
            "abstract", // Provided for by the derived `abstract` text field for Article and not often used for other creative work types
            "alternateNames",
            "archive",
            "bitrate",
            "caption", // Provided for by the derived `caption` text field for most creative work types that have caption
            "config",
            "embedUrl",
            "executionInstance",
            "extra",
            "frontmatter",
            "headings",
            "images",
            "labelOnly",
            "mathml",
            "pageEnd",
            "pageStart",
            "pagination",
            "text",
            "title", // Provided for by the derived `title` text field for Article and not often used for other creative work types
            "thumbnail",
            "transcript",
            "transferEncoding",
            "value",
            "workType",
            // Articles
            "Article.executionMode",
            "Article.executionCount",
            "Article.executionRequired",
            "Article.executionStatus",
            "Article.executionEnded",
            "Article.executionDuration",
            "Table.notes", // Provided for by the derived `notes` text field
            // Avoid many paragraph nodes for each table cell with `text`
            // same as the `text` of the table cell itself (most table cells have a single paragraph)
            "TableCell.content",
            "TableCell.name",
            "TableCell.columnSpan",
            "TableCell.rowSpan",
            "TableCell.horizontalAlignment",
            "TableCell.horizontalAlignmentCharacter",
            "TableCell.verticalAlignment",
            // Exclude list item position as it is provided by the position calculated from the node path
            "ListItem.position",
            // Exclude unnecessary citation properties
            "Citation.citationPrefix",
            "Citation.citationSuffix",
            "Citation.content", // Provided for by the derived `text` field
            // Exclude ForBlock iterations and otherwise properties
            "ForBlock.iterations",
            "ForBlock.otherwise",
            // Exclude unnecessary reference properties
            "Reference.title", // Provided for by the derived `title` text field
            // Exclude unnecessary properties of periodicals, publication volumes and issues
            "Periodical.abstract",
            "Periodical.authors",
            "Periodical.contributors",
            "Periodical.dateAccepted",
            "Periodical.dateCreated",
            "Periodical.dateModified",
            "Periodical.datePublished",
            "Periodical.dateReceived",
            "Periodical.doi",
            "Periodical.editors",
            "Periodical.images",
            "Periodical.references",
            "Periodical.title",
            "PublicationIssue.abstract",
            "PublicationIssue.alternateNames",
            "PublicationIssue.authors",
            "PublicationIssue.contributors",
            "PublicationIssue.dateAccepted",
            "PublicationIssue.dateCreated",
            "PublicationIssue.dateModified",
            "PublicationIssue.datePublished",
            "PublicationIssue.dateReceived",
            "PublicationIssue.description",
            "PublicationIssue.doi",
            "PublicationIssue.editors",
            "PublicationIssue.genre",
            "PublicationIssue.images",
            "PublicationIssue.keywords",
            "PublicationIssue.name",
            "PublicationIssue.references",
            "PublicationIssue.title",
            "PublicationIssue.url",
            "PublicationVolume.abstract",
            "PublicationVolume.alternateNames",
            "PublicationVolume.authors",
            "PublicationVolume.contributors",
            "PublicationVolume.dateAccepted",
            "PublicationVolume.dateCreated",
            "PublicationVolume.dateModified",
            "PublicationVolume.datePublished",
            "PublicationVolume.dateReceived",
            "PublicationVolume.description",
            "PublicationVolume.doi",
            "PublicationVolume.editors",
            "PublicationVolume.genre",
            "PublicationVolume.images",
            "PublicationVolume.keywords",
            "PublicationVolume.name",
            "PublicationVolume.references",
            "PublicationVolume.title",
            "PublicationVolume.url",
            // Exclude unnecessary person properties
            "Person.description",
            "Person.jobTitle",
            "Person.telephoneNumbers",
            "Person.emails",
            "Person.name", // Provided by derived name field
            "Person.memberOf",
            // Exclude unnecessary organization properties
            "Organization.logo",
            "Organization.departments",
        ];

        // Props on specific types that should not be excluded
        let no_skip_props = ["CreativeWork.workType", "Reference.workType"];

        let skip_types = [
            // Object types for which tables are not created
            "AppendixBreak",
            "Brand",
            "Button",
            "CallArgument",
            "CallBlock",
            "Chat",
            "ChatMessage",
            "ChatMessageGroup",
            "CodeInline",
            "CodeLocation",
            "CompilationDigest",
            "CompilationMessage",
            "ContactPoint",
            "DefinedTerm",
            "Emphasis",
            "Enumeration",
            "Excerpt",
            "ExecutionDependant",
            "ExecutionDependency",
            "ExecutionMessage",
            "ExecutionTag",
            "Form",
            "Grant",
            "InlinesBlock",
            "InstructionBlock",
            "InstructionInline",
            "InstructionMessage",
            "Island",
            "ModelParameters",
            "MonetaryGrant",
            "PostalAddress",
            "Product",
            "Prompt",
            "PromptBlock",
            "PropertyValue",
            "ProvenanceCount",
            "SoftwareApplication",
            "Strikeout",
            "Strong",
            "Subscript",
            "SuggestionBlock",
            "SuggestionInline",
            "Superscript",
            "Thing",
            "Underline",
            "Unknown",
            "Walkthrough",
            "WalkthroughStep",
            // Types for which tables are not currently create but probably
            // will be for relations between creative works etc
            "Collection",
            "Comment",
            "CreativeWork",
            "Review",
            // Types that have equivalent Kuzu data types
            "Null",
            "Boolean",
            "Integer",
            "UnsignedInteger",
            "Number",
            "String",
            "Cord",
            "Text",
            "Array",
            "Object",
            "Date",
            "DateTime",
            "Time",
            "Timestamp",
            "Duration",
            // Union types (and variant object types) not expanded
            "Node",
            "ThingVariant",
            "CreativeWorkVariant",
            "Primitive",
            // .. hints
            "Hint",
            "ArrayHint",
            "DatatableHint",
            "DatatableColumnHint",
            "ObjectHint",
            "StringHint",
            // .. validators
            "Validator",
            "ArrayValidator",
            "BooleanValidator",
            "ConstantValidator",
            "DateTimeValidator",
            "DateValidator",
            "DurationValidator",
            "EnumValidator",
            "IntegerValidator",
            "NumberValidator",
            "StringValidator",
            "TimeValidator",
            "TimestampValidator",
            "TupleValidator",
        ];

        // Union types that need to be expanded
        let expand_types = ["Block", "Inline", "Author", "AuthorRoleAuthor"];

        // Node types for which a derive property should be added (e.g. with the plain text
        // representation of the node, or one of its properties, to be used in FTS and semantic search,
        // or in the case of `TableCell` just to use string functions like `contains` or pattern matching)
        let derived_properties = HashMap::from([
            (
                "Article",
                vec![
                    ("title", "to_text(&self.title)"),
                    ("abstract", "to_text(&self.r#abstract)"),
                ],
            ),
            (
                "Table",
                vec![
                    ("caption", "to_text(&self.caption)"),
                    ("notes", "to_text(&self.notes)"),
                ],
            ),
            ("Figure", vec![("caption", "to_text(&self.caption)")]),
            ("CodeChunk", vec![("caption", "to_text(&self.caption)")]),
            ("Paragraph", vec![("text", "to_text(self)")]),
            ("Sentence", vec![("text", "to_text(self)")]),
            ("TableCell", vec![("text", "to_text(self)")]),
            ("Citation", vec![("text", "to_text(&self.options.content)")]),
            ("Reference", vec![("title", "to_text(&self.title)")]),
            (
                "PublicationVolume",
                vec![("volumeNumber", "to_text(&self.volume_number)")],
            ),
            (
                "PublicationIssue",
                vec![("issueNumber", "to_text(&self.issue_number)")],
            ),
            ("Person", vec![("name", "self.name()")]),
        ]);

        // Node types where the primary key is not the node id. These node types are treated
        // as being "outside" of documents: we do not use node ids to create relations with them
        // rather, we use these canonical ids.
        let primary_keys = BTreeMap::from([
            ("Reference", "doi"),
            ("Person", "orcid"),
            ("Organization", "ror"),
        ]);

        // Node types for which full-text search indices should be created.
        let fts_properties = BTreeMap::from([
            ("Article", vec!["title", "abstract", "description"]),
            ("Table", vec!["caption"]),
            ("Figure", vec!["caption"]),
            ("CodeChunk", vec!["caption", "code"]),
            ("Paragraph", vec!["text"]),
            ("Sentence", vec!["text"]),
        ]);

        // Node types for which embeddings should be created.
        // Used to generate a function which returns the text string that is used to create the embeddings
        let embeddings_properties = BTreeMap::from([
            ("Article", vec!["title", "abstract"]),
            ("Paragraph", vec!["text"]),
            ("Sentence", vec!["text"]),
        ]);

        let mut node_tables = Vec::new();
        let mut one_to_many = BTreeMap::new();
        let mut one_to_one = BTreeMap::new();
        let mut many_to_many = BTreeMap::new();
        let mut implems = Vec::new();
        for (title, schema) in &self.schemas {
            if !schema.is_object()
                || schema.r#abstract
                || title.starts_with("Config")
                || skip_types.contains(&title.as_str())
            {
                continue;
            }

            let mut properties = Vec::new();
            let mut relations = Vec::new();
            for (name, property) in &schema.properties {
                if (skip_props.contains(&name.as_str())
                    || skip_props.contains(&format!("{title}.{name}").as_str()))
                    && !no_skip_props.contains(&format!("{title}.{name}").as_str())
                {
                    continue;
                }

                let name = name.as_str();
                let on_options = !(property.is_required || property.is_core);
                let is_option = !property.is_required;
                let is_box = name == "isPartOf";
                let is_array = property.is_array();

                if let Some(property_type) = &property.r#type {
                    let data_type = match property_type {
                        Type::Null => "NULL",
                        Type::Boolean => "BOOLEAN",
                        Type::Integer => "INT64",
                        Type::Number => "DOUBLE",
                        Type::String => "STRING",
                        Type::Array => {
                            match property
                                .items
                                .as_ref()
                                .ok_or_eyre("type `array` should always have `items` specified")?
                            {
                                Items::Type(items_type) => {
                                    let ItemsType { r#type } = items_type;
                                    match r#type {
                                        Type::Null => "NULL[]",
                                        Type::Boolean => "BOOLEAN[]",
                                        Type::Integer => "INT64[]",
                                        Type::Number => "DOUBLE[]",
                                        Type::String => "STRING[]",
                                        _ => bail!("Unhandled items type: {type}"),
                                    }
                                }
                                Items::Ref(ItemsRef { r#ref: ref_type }) => {
                                    if skip_types.contains(&ref_type.as_str()) {
                                        continue;
                                    }

                                    let schema = self
                                        .schemas
                                        .get(ref_type)
                                        .ok_or_eyre("schema should exist")?;

                                    if schema.is_enumeration() {
                                        properties.push((name, "STRING[]", on_options));
                                        continue;
                                    }

                                    let mut pairs = if expand_types.contains(&ref_type.as_str()) {
                                        let variants = self
                                            .schemas
                                            .get(ref_type)
                                            .ok_or_eyre("schema should exist")?
                                            .any_of
                                            .as_ref()
                                            .ok_or_eyre("any_of should be some")?;
                                        variants
                                            .iter()
                                            .filter_map(|schema| {
                                                let variant =
                                                    schema.r#ref.as_deref().expect("should exit");

                                                (!skip_types.contains(&variant)).then_some(format!(
                                                    "FROM `{title}` TO `{variant}`"
                                                ))
                                            })
                                            .collect_vec()
                                    } else {
                                        vec![format!("FROM `{title}` TO `{ref_type}`")]
                                    };

                                    relations.push((
                                        name, on_options, is_option, is_box, is_array, ref_type,
                                    ));

                                    if primary_keys.contains_key(&ref_type.as_str())
                                        || ref_type == "Author"
                                        || ref_type == "AuthorRole"
                                    {
                                        &mut many_to_many
                                    } else {
                                        &mut one_to_many
                                    }
                                    .entry(name)
                                    .and_modify(|existing: &mut Vec<String>| {
                                        existing.append(&mut pairs)
                                    })
                                    .or_insert(pairs);

                                    continue;
                                }
                                _ => {
                                    // Ignore arrays with any_of and lists
                                    continue;
                                }
                            }
                        }
                        Type::Object => "STRING",
                    };

                    properties.push((name, data_type, on_options));
                    continue;
                }

                if let Some(ref_type) = &property.r#ref {
                    let data_type = match ref_type.as_str() {
                        "UnsignedInteger" => "UINT64",
                        "Cord" | "Time" => "STRING",
                        "Date" => "DATE",
                        "DateTime" | "Timestamp" => "TIMESTAMP",
                        "Duration" => "INTERVAL",
                        _ => {
                            if skip_types.contains(&ref_type.as_str()) {
                                continue;
                            }

                            let schema = self
                                .schemas
                                .get(ref_type)
                                .ok_or_eyre(format!("schema should exist: {ref_type}"))?;

                            if schema.is_enumeration() {
                                properties.push((name, "STRING", on_options));
                                continue;
                            }

                            let mut pairs = if expand_types.contains(&ref_type.as_str()) {
                                let variants = self
                                    .schemas
                                    .get(ref_type)
                                    .ok_or_eyre("schema should exist")?
                                    .any_of
                                    .as_ref()
                                    .ok_or_eyre("any_of should be some")?;
                                variants
                                    .iter()
                                    .filter_map(|schema| {
                                        let variant = schema.r#ref.as_deref().expect("should exit");

                                        (!skip_types.contains(&variant))
                                            .then_some(format!("FROM `{title}` TO `{variant}`"))
                                    })
                                    .collect_vec()
                            } else {
                                vec![format!("FROM `{title}` TO `{ref_type}`")]
                            };

                            relations
                                .push((name, on_options, is_option, is_box, is_array, ref_type));

                            if primary_keys.contains_key(&ref_type.as_str())
                                || ref_type == "Author"
                                || ref_type == "AuthorRole"
                            {
                                &mut many_to_many
                            } else {
                                &mut one_to_one
                            }
                            .entry(name)
                            .and_modify(|existing: &mut Vec<String>| existing.append(&mut pairs))
                            .or_insert(pairs);

                            continue;
                        }
                    };
                    properties.push((name, data_type, on_options));
                }
            }

            let mut node_table_props = properties
                .iter()
                .map(|(name, data_type, ..)| format!("\n  `{name}` {data_type},"))
                .join("");
            if let Some(props) = derived_properties.get(&title.as_str()) {
                for (name, ..) in props {
                    node_table_props.push_str(&format!("\n  `{name}` STRING,"));
                }
            }
            if embeddings_properties.contains_key(&title.as_str()) {
                node_table_props.push_str("\n  `embeddings` FLOAT[384],");
            }

            let extra = if let Some(primary_key) = primary_keys.get(&title.as_str()) {
                format!(
                    "
  PRIMARY KEY (`{primary_key}`)
"
                )
            } else {
                "
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
"
                .to_string()
            };

            node_tables.push(format!(
                "CREATE NODE TABLE IF NOT EXISTS `{title}` ({node_table_props}{extra});",
            ));

            let primary_key = primary_keys
                .get(&title.as_str())
                .map(|primary_key| format!("self.{primary_key}.to_kuzu_value()"))
                .unwrap_or("self.node_id().to_kuzu_value()".to_string());

            let mut implem_node_table = properties
                .iter()
                .map(|&(name, .., on_options)| {
                    let mut property = name.to_pascal_case();
                    if property.ends_with("ID") {
                        property.pop();
                        property.push('d');
                    }

                    let mut field = name.to_snake_case();
                    if on_options {
                        field = ["options.", &field].concat()
                    };

                    format!("(NodeProperty::{property}, self.{field}.to_kuzu_type(), self.{field}.to_kuzu_value())")
                })
                .collect_vec();
            if let Some(props) = derived_properties.get(&title.as_str()) {
                for (name, derivation) in props {
                    let property = name.to_pascal_case();
                    implem_node_table.push(format!(
                        "(NodeProperty::{property}, LogicalType::String, {derivation}.to_kuzu_value())"
                    ));
                }
            }
            if embeddings_properties.contains_key(&title.as_str()) {
                implem_node_table.push(
                    "(embeddings_property(), embeddings_type(), Null.to_kuzu_value())".to_string(),
                );
            }
            let implem_node_table = implem_node_table.join(",\n            ");

            let implem_rel_tables = relations
                .into_iter()
                .map(
                    |(name, on_options, is_option, is_box, is_array, ref_type)| {
                        let property = name.to_pascal_case();

                        let mut field = name.to_snake_case();
                        if field == "abstract" {
                            field = "r#abstract".to_string();
                        }

                        if on_options {
                            field = ["options.", &field].concat()
                        };

                        let collect = if ref_type == "AuthorRoleAuthor" {
                            format!("vec![(self.{field}.node_type(), self.{field}.primary_key())]")
                        } else {
                            let mut collect = format!("self.{field}");

                            if is_option || is_array {
                                collect += ".iter()"
                            }

                            if is_box {
                                collect += ".map(|boxed| boxed.as_ref())";
                            }

                            if is_option && is_array {
                                collect += ".flatten()"
                            }

                            format!("relations({collect})")
                        };

                        format!("(NodeProperty::{property}, {collect})")
                    },
                )
                .join(",\n            ");

            implems.push(format!(
                r#"impl DatabaseNode for {title} {{
    fn node_type(&self) -> NodeType {{
        NodeType::{title}
    }}

    fn node_id(&self) -> NodeId {{
        {title}::node_id(self)
    }}
    
    fn primary_key(&self) -> Value {{
        {primary_key}
    }}
    
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        vec![
            {implem_node_table}
        ]
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {{
        vec![
            {implem_rel_tables}
        ]
    }}
}}
"#
            ))
        }

        let node_tables = node_tables.join("\n\n");

        let one_to_one = one_to_one
            .into_iter()
            .map(|(name, pairs)| {
                format!(
                    "CREATE REL TABLE IF NOT EXISTS `{name}` (\n  {},\n  ONE_ONE\n);",
                    pairs.join(",\n  ")
                )
            })
            .join("\n\n");

        let one_to_many = one_to_many
            .into_iter()
            .map(|(name, pairs)| {
                format!(
                    "CREATE REL TABLE IF NOT EXISTS `{name}` (\n  {},\n  ONE_MANY\n);",
                    pairs.join(",\n  ")
                )
            })
            .join("\n\n");

        let many_to_many = many_to_many
            .into_iter()
            .map(|(name, pairs)| {
                format!(
                    "CREATE REL TABLE IF NOT EXISTS `{name}` (\n  {},\n  MANY_MANY\n);",
                    pairs.join(",\n  ")
                )
            })
            .join("\n\n");

        let fts_indices = fts_properties
            .into_iter()
            .map(|(table, properties)| {
                format!(
                    "CALL CREATE_FTS_INDEX('{table}', 'fts', [{}]);",
                    properties
                        .iter()
                        .map(|name| ["'", name, "'"].concat())
                        .join(",")
                )
            })
            .join("\n");

        let vector_indices = embeddings_properties
            .into_iter()
            .map(|(table, ..)| {
                format!("CALL CREATE_VECTOR_INDEX('{table}', 'vector', 'embeddings');")
            })
            .join("\n");

        write(
            dir.join("schemas").join("current.cypher"),
            format!(
                "// Generated file, do not edit. See the Rust `schema-gen` crate;

{node_tables}

{one_to_one}

{one_to_many}

{many_to_many}

INSTALL FTS;
LOAD EXTENSION FTS;
{fts_indices}

INSTALL VECTOR;
LOAD EXTENSION VECTOR;
{vector_indices}
"
            ),
        )
        .await?;

        for title in ["Node", "Block", "Inline", "Author", "AuthorRoleAuthor"] {
            let variants = self
                .schemas
                .get(title)
                .ok_or_eyre("schema should exist")?
                .any_of
                .as_ref()
                .ok_or_eyre("any_of should be some")?
                .iter()
                .filter_map(|schema| {
                    let variant = schema.r#ref.as_deref().expect("should exit");
                    (!skip_types.contains(&variant)).then_some(variant)
                })
                .collect_vec();

            if variants.is_empty() {
                continue;
            }

            let node_type = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.node_type()"))
                .join(",\n            ");

            let node_id = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.node_id()"))
                .join(",\n            ");

            let primary_key = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.primary_key()"))
                .join(",\n            ");

            let node_table = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.node_table()"))
                .join(",\n            ");

            let rel_tables = variants
                .iter()
                .map(|variant| format!("{title}::{variant}(node) => node.rel_tables()"))
                .join(",\n            ");

            implems.push(format!(
                r#"#[allow(unreachable_patterns)]
impl DatabaseNode for {title} {{
    fn node_type(&self) -> NodeType {{
        match self {{
            {node_type},
            _ => NodeType::Unknown
        }}
    }}

    fn node_id(&self) -> NodeId {{
        match self {{
            {node_id},
            _ => NodeId::null()
        }}
    }}

    fn primary_key(&self) -> Value {{
        match self {{
            {primary_key},
            _ => Value::Null(LogicalType::Any)
        }}
    }}

    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)> {{
        match self {{
            {node_table},
            _ => Vec::new()
        }}
    }}

    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, Value)>)> {{
        match self {{
            {rel_tables},
            _ => Vec::new()
        }}
    }}
}}
"#
            ));
        }

        let primary_keys = primary_keys
            .iter()
            .map(|(node_type, value)| format!("NodeType::{node_type} => \"{value}\","))
            .join("\n        ");

        let implems = implems.join("\n");

        write(
            dir.join("src").join("node_types.rs"),
            format!(
                "// Generated file, do not edit. See the Rust `schema-gen` crate.

use kernel_kuzu::{{kuzu::{{LogicalType, Value}}, ToKuzu}};
use codec_text_trait::to_text;
use schema::*;

use super::{{embeddings_property, embeddings_type, DatabaseNode}};

pub(super) fn primary_key(node_type: &NodeType) -> &'static str {{
    match node_type {{
        {primary_keys}
        _ => \"nodeId\"
    }}
}}

fn relations<'lt, I, D>(iter: I) -> Vec<(NodeType, Value)>
where
    I: Iterator<Item = &'lt D>,
    D: DatabaseNode + 'lt,
{{
    iter.flat_map(|item| (!matches!(item.node_type(), NodeType::Unknown)).then_some((item.node_type(), item.primary_key())))
        .collect()
}}

{implems}
"
            ),
        )
        .await?;

        Ok(())
    }
}
