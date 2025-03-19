use std::{
    collections::HashMap,
    fs::remove_dir_all,
    io::Write,
    path::{Path, PathBuf},
};

use fts_indices::FTS_INDICES;
use kuzu::{
    Connection, Database, LogicalType, PreparedStatement, QueryResult, SystemConfig, Value,
};

use common::{
    eyre::{Context, Report, Result, eyre},
    itertools::Itertools,
    tempfile::NamedTempFile,
    tracing,
};
use schema::{Node, NodeId, NodePath, NodeProperty, NodeSlot, NodeType, Visitor};

#[rustfmt::skip]
mod node_types;
#[rustfmt::skip]
mod fts_indices;

mod from_kuzu;
mod to_kuzu;
mod walker;

use to_kuzu::ToKuzu;
use walker::DatabaseWalker;

/// A trait for representing Stencila Schema nodes in a [`NodeDatabase`]
///
/// The implementation of this trait is generated, by the `schema-gen` crate
/// for each node type. To minimize compiled code size, we use enums such as
/// [`NodeType`] and [`NodeProperty`] for the return types of the trait's methods.
#[allow(clippy::type_complexity)]
pub trait DatabaseNode {
    /// Get the [`NodeType`] for the node
    ///
    /// Used for the name of the node table and as they key for storing prepared
    /// statements for the node type.
    fn node_type(&self) -> NodeType;

    /// Get the [`NodeId`] for the node
    ///
    /// Used for the `nodeId` and `parentId` columns of the node table.
    fn node_id(&self) -> NodeId;

    /// Get the names, types, and values of properties of the node table for the node type
    ///
    /// Used to create an entry in the node table for the node type.
    /// In the future may also be used to generate the Cypher to `CREATE NODE TABLE`.
    fn node_table(&self) -> Vec<(NodeProperty, LogicalType, Value)>;

    /// Get the names of relation tables and lists of node id and position for each
    ///
    /// Used to create an entry in relation tables.
    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId)>)>;
}

/// A database of Stencila Schema nodes
///
/// Nodes are stored in a Kuzu database having a node table for each node type
/// e.g. `Article`, `Section`, `Paragraph` and a relationship table for each
/// property which involves a relationship (e.g. `content`, `authors`)
pub struct NodeDatabase {
    /// The instance of the Kuzu database
    database: Database,

    /// The document storage directory associated with the database
    store: Option<PathBuf>,

    /// Prepared statements for deleting a document and all its root node
    delete_doc_statement: Option<PreparedStatement>,

    /// Prepared statements for creating a node
    create_node_statements: HashMap<NodeType, PreparedStatement>,

    /// Prepared statements for creating a relation
    create_rel_statements: HashMap<(NodeType, NodeProperty, NodeType), PreparedStatement>,
}

// The number of entries, above which `COPY FROM` CSV file should be used
const USE_CSV_COUNT: usize = 5;

const EXPECT_JUST_INSERTED: &str = "should exist, just inserted";

impl NodeDatabase {
    /// Create a new node database
    ///
    /// Note that `path` should be a directory (not a file) and will be created if it
    /// does not yet exist.
    pub fn new(path: &Path) -> Result<Self> {
        let exists = path.exists();

        let database = Database::new(path, SystemConfig::default())?;

        if !exists {
            let create = || {
                let connection = Connection::new(&database)?;
                let schema = include_str!("schema.kuzu");
                for statement in schema.split(";") {
                    let statement = statement.trim();
                    if statement.starts_with("//") || statement.is_empty() {
                        continue;
                    }
                    connection
                        .query(statement)
                        .wrap_err_with(|| eyre!("Failed to execute: {statement}"))?;
                }
                Ok::<(), Report>(())
            };
            if let Err(error) = create() {
                // If there is any error in creating the database then remove it so that
                // it is not in a corrupted/partial state
                drop(database);
                remove_dir_all(path)?;

                return Err(error);
            }
        }

        let store = if path.to_string_lossy() == ":memory:" {
            None
        } else {
            path.parent().map(|parent| parent.join("store"))
        };

        Ok(Self {
            database,
            store,
            delete_doc_statement: None,
            create_node_statements: HashMap::new(),
            create_rel_statements: HashMap::new(),
        })
    }

    /// Create a new in-memory node database
    pub fn in_memory() -> Result<Self> {
        NodeDatabase::new(&PathBuf::from(":memory:"))
    }

    /// Insert a document into the database
    #[tracing::instrument(skip(self, node))]
    pub fn insert(&mut self, doc_id: &NodeId, node: &Node) -> Result<()> {
        self.create_node(doc_id, node)?;
        self.update()?;

        Ok(())
    }

    /// Upsert a document into the database
    ///
    /// If the document is already in the database it is replaced with
    /// the new `node`.
    #[tracing::instrument(skip(self, node))]
    pub fn upsert(&mut self, doc_id: &NodeId, node: &Node) -> Result<()> {
        self.delete(doc_id)?;
        self.insert(doc_id, node)?;

        Ok(())
    }

    /// Delete a document from the database
    #[tracing::instrument(skip(self))]
    pub fn delete(&mut self, doc_id: &NodeId) -> Result<()> {
        let connection = Connection::new(&self.database)?;

        let delete_doc = match self.delete_doc_statement.as_mut() {
            Some(statement) => statement,
            None => {
                let statement = connection
                    .prepare("MATCH (node) WHERE node.docId = $doc_id DETACH DELETE node")?;
                self.delete_doc_statement = Some(statement);
                self.delete_doc_statement
                    .as_mut()
                    .expect(EXPECT_JUST_INSERTED)
            }
        };

        connection.execute(delete_doc, vec![("doc_id", doc_id.to_kuzu_value())])?;

        Ok(())
    }

    /// Update database indices
    #[tracing::instrument(skip(self))]
    pub fn update(&self) -> Result<()> {
        self.create_fts_indices()
    }

    /// Create a node in the database
    ///
    /// Instantiates a [`DatabaseWalker`] which walks over the node and creates entries for
    /// it, and all its child nodes, in relation tables.
    #[tracing::instrument(skip(self, node))]
    fn create_node(&mut self, doc_id: &NodeId, node: &Node) -> Result<()> {
        // Walk over the node and collect nodes and relations
        let mut walker = DatabaseWalker::default();
        walker.visit(node);

        // Create entries for each of the node types collected
        for (node_type, (properties, rows)) in walker.node_tables {
            if !matches!(node_type, NodeType::Unknown) {
                self.create_node_entries(doc_id, node_type, properties, rows)?;
            }
        }

        // Create entries for each of the relations collected
        for ((from_node_type, node_property, to_node_type), rows) in walker.rel_tables {
            self.create_rel_entries(from_node_type, node_property, to_node_type, rows)?;
        }

        Ok(())
    }

    /// Create entries in a node table
    ///
    /// Creates or retrieves the prepared statement for the node table and inserts each entry.
    #[tracing::instrument(skip(self, properties, entries))]
    fn create_node_entries(
        &mut self,
        doc_id: &NodeId,
        node_type: NodeType,
        properties: Vec<(NodeProperty, LogicalType)>,
        entries: Vec<(NodePath, NodeId, Vec<Value>)>,
    ) -> Result<()> {
        let connection = Connection::new(&self.database)?;

        // Get node table properties and add additional properties
        let mut properties = properties
            .into_iter()
            .map(|(prop, ..)| prop.to_camel_case())
            .collect_vec();
        properties.append(&mut vec![
            "docId".to_string(),
            "nodeId".to_string(),
            "nodePath".to_string(),
            "position".to_string(),
        ]);

        if entries.len() < USE_CSV_COUNT {
            // Get, or prepare, `CREATE` statement
            let statement = match self.create_node_statements.get_mut(&node_type) {
                Some(statement) => statement,
                None => {
                    let properties = properties
                        .iter()
                        .map(|name| ["`", name, "`: $", name, "_"].concat())
                        .join(", ");
                    let statement = format!("CREATE (:`{node_type}` {{{}}})", properties);

                    let statement = connection.prepare(&statement)?;

                    self.create_node_statements.insert(node_type, statement);
                    self.create_node_statements
                        .get_mut(&node_type)
                        .expect(EXPECT_JUST_INSERTED)
                }
            };

            // Execute prepared statement for each entry
            for (node_path, node_id, mut values) in entries {
                // The trailing underscore on parameter names is necessary for parameters like 'order'
                // to prevent clashes with keywords
                let names = properties
                    .iter()
                    .map(|name| [name, "_"].concat())
                    .collect_vec();
                let names = names.iter().map(|name| name.as_str());

                let position = match node_path.back() {
                    Some(NodeSlot::Index(index)) => Value::UInt32((index + 1) as u32),
                    _ => Value::Null(LogicalType::UInt32),
                };
                values.append(&mut vec![
                    doc_id.to_kuzu_value(),
                    node_id.to_kuzu_value(),
                    node_path.to_kuzu_value(),
                    position,
                ]);

                let params = names.zip(values.into_iter()).collect_vec();

                connection
                    .execute(statement, params)
                    .wrap_err_with(|| eyre!("Unable to create node entry for `{node_type}`"))?;
            }
        } else {
            let mut csv = NamedTempFile::new()?;
            writeln!(&mut csv, "{}", properties.join(","))?;
            for (node_path, node_id, values) in entries {
                for value in values {
                    let field = escape_csv_field(value.to_string());
                    write!(&mut csv, "{field},")?;
                }

                let position = match node_path.back() {
                    Some(NodeSlot::Index(index)) => (index + 1).to_string(),
                    _ => String::new(),
                };
                writeln!(&mut csv, "{doc_id},{node_id},{node_path},{position}")?;
            }

            let filename = csv.path().to_string_lossy();
            connection.query(&format!(
                "COPY `{node_type}` FROM '{filename}' (HEADER=true, file_format='csv', auto_detect=false);"
            ))?;
        }

        Ok(())
    }

    /// Create entries in a relation table
    ///
    /// Creates or retrieves the prepared statement for the relation table and inserts each entry.
    /// Each relation includes the 1-based position of the relation.
    #[tracing::instrument(skip(self, entries))]
    fn create_rel_entries(
        &mut self,
        from_node_type: NodeType,
        node_property: NodeProperty,
        to_node_type: NodeType,
        entries: Vec<(NodeId, Vec<NodeId>)>,
    ) -> Result<()> {
        let connection = Connection::new(&self.database)?;
        let relation = node_property.to_camel_case();

        if entries.len() < USE_CSV_COUNT {
            // Get, or prepare, `MATCH ... CREATE` statement
            let key = (from_node_type, node_property, to_node_type);
            let statement = match self.create_rel_statements.get_mut(&key) {
                Some(statement) => statement,
                None => {
                    let statement = format!(
                        "
                    MATCH (from:`{from_node_type}`), (to:`{to_node_type}`)
                    WHERE from.nodeId = $from_node_id AND to.nodeId = $to_node_id
                    CREATE (from)-[:`{relation}`]->(to)
                    "
                    );

                    let statement = connection.prepare(&statement)?;

                    self.create_rel_statements.insert(key, statement);
                    self.create_rel_statements
                        .get_mut(&key)
                        .expect(EXPECT_JUST_INSERTED)
                }
            };

            // Execute prepared statement for each entry
            for (from_node_id, to_nodes) in entries {
                for node_id in to_nodes {
                    let params = vec![
                        ("from_node_id", from_node_id.to_kuzu_value()),
                        ("to_node_id", node_id.to_kuzu_value()),
                    ];
                    connection.execute(statement, params)?;
                }
            }
        } else {
            let mut csv = NamedTempFile::new()?;
            for (from_node_id, to_nodes) in entries {
                for node_id in to_nodes {
                    writeln!(&mut csv, "{from_node_id},{node_id}")?;
                }
            }

            let filename = csv.path().to_string_lossy();
            connection.query(&format!(
                "COPY `{relation}` FROM '{filename}' (from='{from_node_type}',to='{to_node_type}', file_format='csv', auto_detect=false);"
            ))?;
        }

        Ok(())
    }

    /// Create FTS indices
    #[tracing::instrument(skip(self))]
    pub fn create_fts_indices(&self) -> Result<()> {
        let connection = Connection::new(&self.database)?;

        // This occasionally throws error "Too many values for string_format",
        // although it seems to succeed, so is `ok()`ed.
        connection.query("LOAD EXTENSION FTS;").ok();

        for (table, properties) in FTS_INDICES {
            // This is `ok()`ed because it may may fail if the index does not exist yet.
            // This is a lot less code than explicitly listing indices and checking for each one.
            connection
                .query(&format!("CALL DROP_FTS_INDEX('{table}', 'fts');"))
                .ok();

            connection.query(&format!(
                "CALL CREATE_FTS_INDEX('{table}', 'fts', [{}]);",
                properties
                    .iter()
                    .map(|name| ["'", name, "'"].concat())
                    .join(",")
            ))?;
        }

        Ok(())
    }

    /// Query the database using Cypher Query Language
    pub fn query(&self, cypher: &str) -> Result<QueryResult> {
        let connection = Connection::new(&self.database)?;

        let result = connection.query(cypher)?;

        Ok(result)
    }
}

fn escape_csv_field(field: String) -> String {
    if field.contains(',') || field.contains('\n') || field.contains('"') {
        let escaped = field.replace("\"", "\"\"").replace("\n", "\\n");
        format!("\"{}\"", escaped)
    } else {
        field
    }
}
