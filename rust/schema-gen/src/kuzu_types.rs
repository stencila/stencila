use std::fmt::Display;

use common::{
    itertools::Itertools,
    serde::{Deserialize, Serialize},
};

/// Represents a complete Kuzu database schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DatabaseSchema {
    pub node_tables: Vec<NodeTable>,
    pub relationship_tables: Vec<RelationshipTable>,
    pub indices: Vec<Index>,
}

/// Represents a Kuzu node table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct NodeTable {
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Option<String>,
    pub derived_properties: Vec<DerivedProperty>,
    pub has_embeddings: bool,
    pub has_standard_node_fields: bool,
}

/// Represents a Kuzu relationship table  
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct RelationshipTable {
    pub name: String,
    pub pairs: Vec<FromToPair>,
    pub cardinality: Cardinality,
}

/// Represents a relationship FROM/TO pair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct FromToPair {
    pub from: String,
    pub to: String,
}

/// Relationship cardinality
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToMany,
}

/// Represents a table column
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub on_options: bool,
}

/// Represents a derived property (computed fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DerivedProperty {
    pub name: String,
    pub derivation: String,
}

/// Kuzu data types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub enum DataType {
    Null,
    Boolean,
    Int64,
    UInt64,
    Double,
    String,
    Date,
    Timestamp,
    Interval,
    BooleanArray,
    Int64Array,
    DoubleArray,
    StringArray,
    FloatArray(usize), // For embeddings with dimension
}

/// Database indices
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub enum Index {
    FullTextSearch {
        table: String,
        name: String,
        properties: Vec<String>,
    },
    Vector {
        table: String,
        name: String,
        property: String,
    },
}

/// Information about a relation field for Rust code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct RelationInfo {
    pub name: String,
    pub on_options: bool,
    pub is_option: bool,
    pub is_box: bool,
    pub is_array: bool,
    pub ref_type: String,
}

impl DatabaseSchema {
    pub fn new() -> Self {
        Self {
            node_tables: Vec::new(),
            relationship_tables: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_node_table(&mut self, table: NodeTable) {
        self.node_tables.push(table);
    }

    pub fn add_relationship_table(&mut self, table: RelationshipTable) {
        self.relationship_tables.push(table);
    }

    pub fn add_index(&mut self, index: Index) {
        self.indices.push(index);
    }
}

impl NodeTable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: Vec::new(),
            primary_key: None,
            derived_properties: Vec::new(),
            has_embeddings: false,
            has_standard_node_fields: true,
        }
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    pub fn set_primary_key(&mut self, key: String) {
        self.primary_key = Some(key);
    }

    pub fn add_derived_property(&mut self, property: DerivedProperty) {
        self.derived_properties.push(property);
    }

    pub fn with_embeddings(mut self) -> Self {
        self.has_embeddings = true;
        self
    }

    pub fn without_standard_fields(mut self) -> Self {
        self.has_standard_node_fields = false;
        self
    }
}

impl RelationshipTable {
    pub fn new(name: String, cardinality: Cardinality) -> Self {
        Self {
            name,
            pairs: Vec::new(),
            cardinality,
        }
    }

    pub fn add_pair(&mut self, from: String, to: String) {
        self.pairs.push(FromToPair { from, to });
    }
}

impl Column {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
            on_options: false,
        }
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn on_options(mut self) -> Self {
        self.on_options = true;
        self
    }
}

impl DerivedProperty {
    pub fn new(name: String, derivation: String) -> Self {
        Self { name, derivation }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Null => write!(f, "NULL"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Int64 => write!(f, "INT64"),
            DataType::UInt64 => write!(f, "UINT64"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::String => write!(f, "STRING"),
            DataType::Date => write!(f, "DATE"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Interval => write!(f, "INTERVAL"),
            DataType::BooleanArray => write!(f, "BOOLEAN[]"),
            DataType::Int64Array => write!(f, "INT64[]"),
            DataType::DoubleArray => write!(f, "DOUBLE[]"),
            DataType::StringArray => write!(f, "STRING[]"),
            DataType::FloatArray(dim) => write!(f, "FLOAT[{}]", dim),
        }
    }
}

impl Display for Cardinality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cardinality::OneToOne => write!(f, "ONE_ONE"),
            Cardinality::OneToMany => write!(f, "ONE_MANY"),
            Cardinality::ManyToMany => write!(f, "MANY_MANY"),
        }
    }
}

impl Display for FromToPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FROM `{}` TO `{}`", self.from, self.to)
    }
}

// Cypher DDL generation
impl Display for NodeTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CREATE NODE TABLE IF NOT EXISTS `{}` (", self.name)?;

        // Regular columns
        for column in &self.columns {
            write!(f, "\n  `{}` {},", column.name, column.data_type)?;
        }

        // Derived properties
        for derived in &self.derived_properties {
            write!(f, "\n  `{}` STRING,", derived.name)?;
        }

        // Embeddings
        if self.has_embeddings {
            write!(f, "\n  `embeddings` FLOAT[384],")?;
        }

        // Primary key or standard node fields
        if let Some(ref primary_key) = self.primary_key {
            write!(f, "\n  PRIMARY KEY (`{}`)", primary_key)?;
        } else if self.has_standard_node_fields {
            write!(
                f,
                "\n  `docId` STRING,\n  `nodeId` STRING PRIMARY KEY,\n  `nodePath` STRING,\n  `nodeAncestors` STRING,\n  `position` UINT32"
            )?;
        }

        write!(f, "\n);")
    }
}

impl Display for RelationshipTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CREATE REL TABLE IF NOT EXISTS `{}` (\n  ", self.name)?;
        write!(f, "{}", self.pairs.iter().join(",\n  "))?;
        write!(f, ",\n  {}\n);", self.cardinality)
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Index::FullTextSearch {
                table,
                name,
                properties,
            } => {
                let props = properties.iter().map(|p| format!("'{}'", p)).join(",");
                write!(
                    f,
                    "CALL CREATE_FTS_INDEX('{}', '{}', [{}]);",
                    table, name, props
                )
            }
            Index::Vector {
                table,
                name,
                property,
            } => {
                write!(
                    f,
                    "CALL CREATE_VECTOR_INDEX('{}', '{}', '{}');",
                    table, name, property
                )
            }
        }
    }
}

impl Default for DatabaseSchema {
    fn default() -> Self {
        Self::new()
    }
}
