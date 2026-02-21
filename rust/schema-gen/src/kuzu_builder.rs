use std::collections::{BTreeMap, HashMap};

use eyre::{OptionExt, Result, bail};
use itertools::Itertools;

use crate::{
    kuzu_types::{
        Cardinality, Column, DataType, DatabaseSchema, DerivedProperty, Index, NodeTable,
        RelationshipInfo, RelationshipTable,
    },
    schema::{Items, ItemsRef, ItemsType, Type},
    schemas::Schemas,
};

/// Builds KuzuSchema from Stencila schemas
pub struct KuzuSchemaBuilder<'a> {
    schemas: &'a Schemas,
    skip_props: Vec<&'static str>,
    no_skip_props: Vec<&'static str>,
    skip_types: Vec<&'static str>,
    expand_types: Vec<&'static str>,
    derived_properties: HashMap<&'static str, Vec<(&'static str, &'static str)>>,
    primary_keys: BTreeMap<&'static str, &'static str>,
    fts_properties: BTreeMap<&'static str, Vec<&'static str>>,
    embeddings_properties: BTreeMap<&'static str, Vec<&'static str>>,
    node_relationships: BTreeMap<String, Vec<RelationshipInfo>>,
}

impl<'a> KuzuSchemaBuilder<'a> {
    pub fn new(schemas: &'a Schemas) -> Self {
        Self {
            schemas,
            skip_props: vec![
                // General
                "type",
                "id",
                // Blanket exclusions, regardless of type
                "abstract",
                "alternateNames",
                "archive",
                "bitrate",
                "caption", // Derived text field used instead
                "config",
                "embedUrl",
                "executionInstance",
                "extra",
                "frontmatter",
                "headings",
                "images",
                "labelAutomatically",
                "labelOnly",
                "mathml",
                "pageEnd",
                "pageStart",
                "pagination",
                "text",
                "title",
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
                // Table exclusions
                "Table.notes",
                "TableCell.content",
                "TableCell.name",
                "TableCell.columnSpan",
                "TableCell.rowSpan",
                "TableCell.horizontalAlignment",
                "TableCell.horizontalAlignmentCharacter",
                "TableCell.verticalAlignment",
                // Datatable exclusions
                "Datatable.notes",
                // List exclusions
                "ListItem.position",
                // ForBlock exclusions
                "ForBlock.iterations",
                "ForBlock.otherwise",
                // Citation and reference exclusions
                "Citation.citationPrefix",
                "Citation.citationSuffix",
                "Citation.content",
                "CitationGroup.content",
                "Reference.title",
                "Reference.content",
                // Periodical exclusions
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
                // Person exclusions
                "Person.description",
                "Person.jobTitle",
                "Person.telephoneNumbers",
                "Person.emails",
                "Person.name",
                "Person.memberOf",
                // Organization exclusions
                "Organization.logo",
                "Organization.departments",
            ],
            no_skip_props: vec!["CreativeWork.workType", "Reference.workType"],
            skip_types: vec![
                // Object types for which tables are not created
                "Agent",
                "AppendixBreak",
                "Bibliography",
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
                "Page",
                "PostalAddress",
                "Product",
                "Prompt",
                "PromptBlock",
                "PropertyValue",
                "ProvenanceCount",
                "Skill",
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
                "Workflow",
                // Types for which tables are not currently created
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
                // Union types not expanded
                "Node",
                "ThingVariant",
                "CreativeWorkVariant",
                "Primitive",
                // Hints
                "Hint",
                "ArrayHint",
                "DatatableHint",
                "DatatableColumnHint",
                "ObjectHint",
                "StringHint",
                // Validators
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
            ],
            expand_types: vec!["Block", "Inline", "Author", "AuthorRoleAuthor"],
            derived_properties: HashMap::from([
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
            ]),
            primary_keys: BTreeMap::from([
                ("Reference", "doi"),
                ("Person", "orcid"),
                ("Organization", "ror"),
            ]),
            fts_properties: BTreeMap::from([
                ("Article", vec!["title", "abstract", "description"]),
                ("Table", vec!["caption"]),
                ("Figure", vec!["caption"]),
                ("CodeChunk", vec!["caption", "code"]),
                ("Paragraph", vec!["text"]),
                ("Sentence", vec!["text"]),
            ]),
            embeddings_properties: BTreeMap::from([
                ("Article", vec!["title", "abstract"]),
                ("Paragraph", vec!["text"]),
                ("Sentence", vec!["text"]),
            ]),
            node_relationships: BTreeMap::new(),
        }
    }

    pub fn build(&mut self) -> Result<DatabaseSchema> {
        let mut schema = DatabaseSchema::new();

        // Track relationships
        let mut one_to_many = BTreeMap::new();
        let mut one_to_one = BTreeMap::new();
        let mut many_to_many = BTreeMap::new();

        for (title, stencila_schema) in &self.schemas.schemas {
            if !stencila_schema.is_object()
                || stencila_schema.r#abstract
                || title.starts_with("Config")
                || self.skip_types.contains(&title.as_str())
            {
                continue;
            }

            let mut node_table = NodeTable::new(title.clone());

            // Process properties
            let mut relations = Vec::new();
            for (name, property) in &stencila_schema.properties {
                if (self.skip_props.contains(&name.as_str())
                    || self
                        .skip_props
                        .contains(&format!("{title}.{name}").as_str()))
                    && !self
                        .no_skip_props
                        .contains(&format!("{title}.{name}").as_str())
                {
                    continue;
                }

                let on_options = !(property.is_required || property.is_core);
                let is_option = !property.is_required;
                let is_box = name == "isPartOf";
                let is_array = property.is_array();

                if let Some(property_type) = &property.r#type {
                    if let Some(column) = self.process_type_property(
                        property_type,
                        property,
                        title,
                        name,
                        on_options,
                        is_option,
                        is_box,
                        is_array,
                        &mut one_to_many,
                        &mut one_to_one,
                        &mut many_to_many,
                        &mut relations,
                    )? {
                        node_table.add_column(column)
                    }
                } else if let Some(ref_type) = &property.r#ref
                    && let Some(column) = self.process_ref_property(
                        ref_type,
                        name,
                        on_options,
                        is_option,
                        is_box,
                        is_array,
                        title,
                        &mut one_to_many,
                        &mut one_to_one,
                        &mut many_to_many,
                        &mut relations,
                    )?
                {
                    node_table.add_column(column)
                }
            }

            // Add derived properties
            if let Some(props) = self.derived_properties.get(title.as_str()) {
                for &(name, derivation) in props {
                    node_table.add_derived_property(DerivedProperty::new(
                        name.to_string(),
                        derivation.to_string(),
                    ));
                }
            }

            // Add embeddings if needed
            if self.embeddings_properties.contains_key(title.as_str()) {
                node_table = node_table.with_embeddings();
            }

            // Set primary key if needed
            if let Some(&primary_key) = self.primary_keys.get(title.as_str()) {
                node_table.set_primary_key(primary_key.to_string());
                node_table = node_table.without_standard_fields();
            }

            self.node_relationships.insert(title.clone(), relations);
            schema.add_node_table(node_table);
        }

        // Add relationship tables
        self.add_relationship_tables(&mut schema, one_to_one, Cardinality::OneToOne);
        self.add_relationship_tables(&mut schema, one_to_many, Cardinality::OneToMany);
        self.add_relationship_tables(&mut schema, many_to_many, Cardinality::ManyToMany);

        // Add indices
        self.add_indices(&mut schema);

        Ok(schema)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_type_property(
        &self,
        property_type: &Type,
        property: &crate::schema::Schema,
        title: &str,
        name: &str,
        on_options: bool,
        is_option: bool,
        is_box: bool,
        is_array: bool,
        one_to_many: &mut BTreeMap<String, Vec<String>>,
        one_to_one: &mut BTreeMap<String, Vec<String>>,
        many_to_many: &mut BTreeMap<String, Vec<String>>,
        relationships: &mut Vec<RelationshipInfo>,
    ) -> Result<Option<Column>> {
        let data_type = match property_type {
            Type::Null => DataType::Null,
            Type::Boolean => DataType::Boolean,
            Type::Integer => DataType::Int64,
            Type::Number => DataType::Double,
            Type::String => DataType::String,
            Type::Array => {
                let items = property
                    .items
                    .as_ref()
                    .ok_or_eyre("type `array` should always have `items` specified")?;

                match items {
                    Items::Type(items_type) => {
                        let ItemsType { r#type } = items_type;
                        match r#type {
                            Type::Null => DataType::Null, // Will be converted to NULL[]
                            Type::Boolean => DataType::BooleanArray,
                            Type::Integer => DataType::Int64Array,
                            Type::Number => DataType::DoubleArray,
                            Type::String => DataType::StringArray,
                            _ => bail!("Unhandled items type: {type}"),
                        }
                    }
                    Items::Ref(ItemsRef { r#ref: ref_type }) => {
                        if self.skip_types.contains(&ref_type.as_str()) {
                            return Ok(None);
                        }

                        let schema = self
                            .schemas
                            .schemas
                            .get(ref_type.as_str())
                            .ok_or_eyre("schema should exist")?;

                        if schema.is_enumeration() {
                            return Ok(Some(
                                Column::new(name.to_string(), DataType::StringArray).on_options(),
                            ));
                        }

                        // This is a relationship, not a column
                        self.add_relationship_pairs(
                            title,
                            ref_type,
                            name,
                            on_options,
                            is_option,
                            is_box,
                            is_array,
                            one_to_many,
                            one_to_one,
                            many_to_many,
                            relationships,
                        )?;
                        return Ok(None);
                    }
                    _ => return Ok(None),
                }
            }
            Type::Object => DataType::String,
        };

        let mut column = Column::new(name.to_string(), data_type);
        if on_options {
            column = column.on_options();
        }
        Ok(Some(column))
    }

    #[allow(clippy::too_many_arguments)]
    fn process_ref_property(
        &self,
        ref_type: &str,
        name: &str,
        on_options: bool,
        is_option: bool,
        is_box: bool,
        is_array: bool,
        title: &str,
        one_to_many: &mut BTreeMap<String, Vec<String>>,
        one_to_one: &mut BTreeMap<String, Vec<String>>,
        many_to_many: &mut BTreeMap<String, Vec<String>>,
        relationships: &mut Vec<RelationshipInfo>,
    ) -> Result<Option<Column>> {
        let data_type = match ref_type {
            "UnsignedInteger" => DataType::UInt64,
            "Cord" | "Time" => DataType::String,
            "Date" => DataType::Date,
            "DateTime" | "Timestamp" => DataType::Timestamp,
            "Duration" => DataType::Interval,
            _ => {
                if self.skip_types.contains(&ref_type) {
                    return Ok(None);
                }

                let schema = self
                    .schemas
                    .schemas
                    .get(ref_type)
                    .ok_or_eyre("schema should exist")?;

                if schema.is_enumeration() {
                    let mut column = Column::new(name.to_string(), DataType::String);
                    if on_options {
                        column = column.on_options();
                    }
                    return Ok(Some(column));
                }

                // This is a relationship, not a column
                self.add_relationship_pairs(
                    title,
                    ref_type,
                    name,
                    on_options,
                    is_option,
                    is_box,
                    is_array,
                    one_to_many,
                    one_to_one,
                    many_to_many,
                    relationships,
                )?;
                return Ok(None);
            }
        };

        let mut column = Column::new(name.to_string(), data_type);
        if on_options {
            column = column.on_options();
        }
        Ok(Some(column))
    }

    #[allow(clippy::too_many_arguments)]
    fn add_relationship_pairs(
        &self,
        title: &str,
        ref_type: &str,
        name: &str,
        on_options: bool,
        is_option: bool,
        is_box: bool,
        is_array: bool,
        one_to_many: &mut BTreeMap<String, Vec<String>>,
        one_to_one: &mut BTreeMap<String, Vec<String>>,
        many_to_many: &mut BTreeMap<String, Vec<String>>,
        relationships: &mut Vec<RelationshipInfo>,
    ) -> Result<()> {
        let mut pairs = if self.expand_types.contains(&ref_type) {
            let variants = self
                .schemas
                .schemas
                .get(ref_type)
                .ok_or_eyre("schema should exist")?
                .any_of
                .as_ref()
                .ok_or_eyre("any_of should be some")?;

            variants
                .iter()
                .filter_map(|schema| {
                    let variant = schema.r#ref.as_deref().expect("should exist");
                    (!self.skip_types.contains(&variant))
                        .then_some(format!("FROM `{title}` TO `{variant}`"))
                })
                .collect_vec()
        } else {
            vec![format!("FROM `{title}` TO `{ref_type}`")]
        };

        relationships.push(RelationshipInfo {
            name: name.to_string(),
            on_options,
            is_option,
            is_box,
            is_array,
            ref_type: ref_type.to_string(),
        });

        let target_map = if self.primary_keys.contains_key(ref_type)
            || ref_type == "Author"
            || ref_type == "AuthorRole"
        {
            many_to_many
        } else if is_array {
            one_to_many
        } else {
            one_to_one
        };

        target_map
            .entry(name.to_string())
            .and_modify(|existing: &mut Vec<String>| existing.append(&mut pairs))
            .or_insert(pairs);

        Ok(())
    }

    fn add_relationship_tables(
        &self,
        schema: &mut DatabaseSchema,
        tables: BTreeMap<String, Vec<String>>,
        cardinality: Cardinality,
    ) {
        for (name, pairs) in tables {
            let mut rel_table = RelationshipTable::new(name, cardinality.clone());
            for pair_str in pairs {
                // Parse "FROM `X` TO `Y`" format
                if let Some(from_to) = self.parse_from_to(&pair_str) {
                    rel_table.add_pair(from_to.0, from_to.1);
                }
            }
            schema.add_relationship_table(rel_table);
        }
    }

    fn parse_from_to(&self, pair_str: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = pair_str.split("FROM `").collect();
        if parts.len() != 2 {
            return None;
        }

        let rest = parts[1];
        let parts: Vec<&str> = rest.split("` TO `").collect();
        if parts.len() != 2 {
            return None;
        }

        let from = parts[0].to_string();
        let to = parts[1].trim_end_matches('`').to_string();
        Some((from, to))
    }

    fn add_indices(&self, schema: &mut DatabaseSchema) {
        // Add FTS indices
        for (table, properties) in &self.fts_properties {
            schema.add_index(Index::FullTextSearch {
                table: table.to_string(),
                name: "fts".to_string(),
                properties: properties.iter().map(|s| s.to_string()).collect(),
            });
        }

        // Add vector indices
        for table in self.embeddings_properties.keys() {
            schema.add_index(Index::Vector {
                table: table.to_string(),
                name: "vector".to_string(),
                property: "embeddings".to_string(),
            });
        }
    }

    pub fn get_primary_keys(&self) -> BTreeMap<String, String> {
        self.primary_keys
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    pub fn get_node_relationships(&self) -> &BTreeMap<String, Vec<RelationshipInfo>> {
        &self.node_relationships
    }

    pub fn get_skip_types(&self) -> &[&'static str] {
        &self.skip_types
    }
}
