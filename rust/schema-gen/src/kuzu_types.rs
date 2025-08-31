use serde::{Deserialize, Serialize};

/// Represents a complete Kuzu database schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSchema {
    /// Node tables in the database
    ///
    /// Used to generate `CREATE NODE TABLE` statements in Cypher and
    /// `DatabaseNode` implementations in Rust data marshalling.
    pub node_tables: Vec<NodeTable>,

    /// Relationship tables defining connections between node types
    ///
    /// Used to generate `CREATE REL TABLE` statements in Cypher and
    /// relationship handling in Rust data marshalling.
    pub relationship_tables: Vec<RelationshipTable>,

    /// Database indices for performance optimization
    ///
    /// Used to generate `CREATE_FTS_INDEX` and `CREATE_VECTOR_INDEX` calls in
    /// Cypher schema.
    pub indices: Vec<Index>,
}

/// Represents a Kuzu node table
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeTable {
    /// Table name used in CREATE NODE TABLE statement
    ///
    /// Used directly in Cypher DDL generation and as the Rust struct name for `DatabaseNode` implementations
    pub name: String,

    /// Column definitions for the table
    ///
    /// Used to generate column specifications in Cypher CREATE statements and property accessors in Rust
    pub columns: Vec<Column>,

    /// Custom primary key column name if different from standard nodeId
    ///
    /// Used to generate PRIMARY KEY clause in Cypher and primary_key() method implementation in Rust
    pub primary_key: Option<String>,

    /// Computed properties derived from other fields
    ///
    /// Added as STRING columns in Cypher schema and handled specially in Rust marshalling code
    pub derived_properties: Vec<DerivedProperty>,

    /// Whether this table includes vector embeddings column
    ///
    /// Adds `embeddings FLOAT[384]` column in Cypher and embeddings handling in Rust
    pub has_embeddings: bool,

    /// Whether to include standard Stencila node fields (docId, nodeId, nodePath, etc.)
    ///
    /// Controls generation of standard node columns in Cypher and standard field access in Rust
    pub has_standard_node_fields: bool,
}

/// Represents a table column
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    /// Column name used in CREATE TABLE statements
    ///
    /// Used directly in Cypher DDL and converted to snake_case for Rust field access
    pub name: String,

    /// Kuzu data type for the column
    ///
    /// Mapped to Kuzu types in Cypher generation and ToKuzu implementations in Rust
    pub data_type: DataType,

    /// Whether the column can contain NULL values
    ///
    /// Used to determine DEFAULT NULL clauses in Cypher ALTER TABLE statements for migrations
    pub nullable: bool,

    /// Whether this column is accessed via the `options` field in Rust structs
    ///
    /// Controls field access pattern in Rust code generation: `self.field` vs `options.field`
    pub on_options: bool,
}

/// Represents a derived property (computed fields)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedProperty {
    /// Property name used as column name in database
    ///
    /// Added as STRING column in Cypher schema and converted to PascalCase for Rust property enums
    pub name: String,

    /// Rust expression that computes the property value Used directly in Rust
    ///
    /// code generation for the property value in `DatabaseNode` implementations
    pub derivation: String,
}

/// Kuzu data types mapped to Cypher types and Rust ToKuzu implementations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Fixed-dimension float array for embeddings (generates "FLOAT[n]" in Cypher)
    /// The usize parameter specifies the vector dimension (e.g., 384 for embeddings)
    FloatArray(usize),
}

/// Represents a Kuzu relationship table  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipTable {
    /// Relationship table name used in CREATE REL TABLE statement
    ///
    /// Used directly in Cypher DDL generation and for relationship handling in Rust
    pub name: String,

    /// FROM/TO node type pairs that this relationship connects
    ///
    /// Used to generate FROM/TO clauses in Cypher CREATE REL TABLE statements
    pub pairs: Vec<FromToPair>,

    /// Relationship cardinality constraint (ONE_ONE, ONE_MANY, MANY_MANY)
    ///
    /// Used to generate cardinality specification in Cypher CREATE REL TABLE statements
    pub cardinality: Cardinality,
}

/// Represents a relationship FROM/TO pair
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FromToPair {
    /// Source node table name for the relationship
    ///
    /// Used in Cypher FROM clause generation for CREATE REL TABLE statements
    pub from: String,

    /// Target node table name for the relationship
    ///
    /// Used in Cypher TO clause generation for CREATE REL TABLE statements
    pub to: String,
}

/// Relationship cardinality constraints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cardinality {
    /// One-to-one relationship constraint
    ///
    /// Generates `ONE_ONE` cardinality in Cypher CREATE REL TABLE statements
    OneToOne,

    /// One-to-many relationship constraint
    ///
    /// Generates `ONE_MANY` cardinality in Cypher CREATE REL TABLE statements
    OneToMany,

    /// Many-to-many relationship constraint
    ///
    /// Generates `MANY_MANY` cardinality in Cypher CREATE REL TABLE statements
    ManyToMany,
}

/// Information about a relation field for Rust code generation
///
/// This struct is used solely for generating Rust `DatabaseNode`
/// implementations and is not used in Cypher generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipInfo {
    /// Field name in the Rust struct
    ///
    /// Converted to snake_case for field access and PascalCase for NodeProperty enum
    pub name: String,

    /// Whether this field is accessed via the `options` field in Rust structs
    ///
    /// Controls field access pattern: `self.field` vs `options.field`
    pub on_options: bool,

    /// Whether the field is wrapped in Option<T> in Rust
    ///
    /// Affects iteration logic in Rust relation collection code
    pub is_option: bool,

    /// Whether the field is wrapped in Box<T> in Rust
    ///
    /// Requires additional .as_ref() calls in Rust relation collection code
    pub is_box: bool,

    /// Whether the field is an array/Vec<T> in Rust
    ///
    /// Affects iteration logic in Rust relation collection code
    pub is_array: bool,

    /// The referenced type name
    ///
    /// Used to determine special handling for AuthorRoleAuthor vs other types in Rust code
    pub ref_type: String,
}

/// Database indices for performance optimization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Index {
    /// Full-text search index for text search capabilities
    ///
    /// Used in Cypher CALL CREATE_FTS_INDEX statements
    FullTextSearch {
        /// Target table name for the index
        table: String,

        /// Index name identifier
        name: String,

        /// Column names to include in the full-text index
        properties: Vec<String>,
    },

    /// Vector similarity index for embeddings search
    ///
    /// Used in Cypher CALL CREATE_VECTOR_INDEX statements
    Vector {
        /// Target table name for the index
        table: String,

        /// Index name identifier
        name: String,

        /// Column name containing vector data (typically "embeddings")
        property: String,
    },
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

impl Default for DatabaseSchema {
    fn default() -> Self {
        Self::new()
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

    #[allow(dead_code)]
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
