use std::collections::HashMap;

use kuzu::{
    Connection, Database, LogicalType, PreparedStatement, QueryResult, SystemConfig, Value,
};

use common::{
    eyre::{Context, Result, eyre},
    itertools::Itertools,
};
use schema::{Block, Inline, Node, NodeId, NodeProperty, NodeType, Visitor, WalkControl};

#[rustfmt::skip]
mod node_types;
mod to_kuzu;

use to_kuzu::ToKuzu;

/// A database of Stencila Schema nodes
///
/// Nodes are stored in a Kuzu database having a node table for each node type
/// e.g. `Article`, `Section`, `Paragraph` and a relationship table for each
/// property which involves a relationship (e.g. `content`, `authors`)
pub struct NodeDatabase {
    /// The instance of the Kuzu database
    database: Database,

    /// Prepared statements for creating a node
    create_node_statements: HashMap<NodeType, PreparedStatement>,

    /// Prepared statements for creating a relation
    create_rel_statements: HashMap<(NodeType, NodeProperty, NodeType), PreparedStatement>,
}

impl NodeDatabase {
    /// Create a new node database
    ///
    /// Note that `path` should be a directory (not a file) and will be created if it
    /// does not yet exist.
    pub fn new(path: &str) -> Result<Self> {
        let database = Database::new(path, SystemConfig::default())?;

        {
            let connection = Connection::new(&database)?;
            connection.query(include_str!("schema.kuzu"))?;
        }

        Ok(Self {
            database,
            create_node_statements: HashMap::new(),
            create_rel_statements: HashMap::new(),
        })
    }

    /// Create a new in-memory node database
    pub fn in_memory() -> Result<Self> {
        NodeDatabase::new(":memory:")
    }

    /// Create a node in the database
    ///
    /// Instantiates a [`DatabaseWalker`] which walks over the node and creates entries for
    /// it, and all its child nodes, in relation tables.
    pub fn create(&mut self, node: &Node) -> Result<()> {
        // Walk over the node and collect nodes and relations
        let mut walker = DatabaseWalker::default();
        walker.visit(node);

        // Create entries for each of the node types collected
        for (node_type, (properties, rows)) in walker.node_tables {
            self.create_node_entries(node_type, properties, rows)?;
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
    fn create_node_entries(
        &mut self,
        node_type: NodeType,
        properties: Vec<(NodeProperty, LogicalType)>,
        entries: Vec<(NodeId, Vec<Value>)>,
    ) -> Result<()> {
        // Get node table properties and add additional properties
        let mut properties = properties
            .into_iter()
            .map(|(prop, ..)| prop.to_camel_case())
            .collect_vec();
        properties.push("nodeId".to_string());

        let connection = Connection::new(&self.database)?;

        // Get, or prepare, `CREATE` statement
        let statement = match self.create_node_statements.get_mut(&node_type) {
            Some(statement) => statement,
            None => {
                let properties = properties
                    .iter()
                    .map(|name| [name, ": $", name].concat())
                    .join(", ");
                let statement = format!("CREATE (:{node_type} {{{}}})", properties);

                let statement = connection.prepare(&statement)?;

                self.create_node_statements.insert(node_type, statement);
                self.create_node_statements
                    .get_mut(&node_type)
                    .expect("should exist because just inserted")
            }
        };

        // Execute prepared statement for each entry
        for (node_id, mut values) in entries {
            let names = properties.iter().map(|name| name.as_str());
            values.push(node_id.to_kuzu_value());
            let params = names.zip(values.into_iter()).collect_vec();

            connection
                .execute(statement, params)
                .wrap_err_with(|| eyre!("Unable to create node entry for `{node_type}`"))?;
        }

        Ok(())
    }

    /// Create entries in a relation table
    ///
    /// Creates or retrieves the prepared statement for the relation table and inserts each entry.
    /// Each relation includes the 1-based position of the relation.
    fn create_rel_entries(
        &mut self,
        from_node_type: NodeType,
        node_property: NodeProperty,
        to_node_type: NodeType,
        entries: Vec<(NodeId, Vec<(NodeId, usize)>)>,
    ) -> Result<()> {
        let connection = Connection::new(&self.database)?;

        // Get, or prepare, `MATCH ... CREATE` statement
        let key = (from_node_type, node_property, to_node_type);
        let statement = match self.create_rel_statements.get_mut(&key) {
            Some(statement) => statement,
            None => {
                let statement = format!(
                    "
                    MATCH (from:{from_node_type}), (to:{to_node_type})
                    WHERE from.nodeId = $from_node_id AND to.nodeId = $to_node_id
                    CREATE (from)-[:{node_property} {{position: $position}}]->(to)
                    "
                );

                let statement = connection.prepare(&statement)?;

                self.create_rel_statements.insert(key, statement);
                self.create_rel_statements
                    .get_mut(&key)
                    .expect("just inserted")
            }
        };

        // Execute prepared statement for each entry
        for (from_node_id, to_nodes) in entries {
            for (node_id, position) in to_nodes {
                let params = vec![
                    ("from_node_id", from_node_id.to_kuzu_value()),
                    ("to_node_id", node_id.to_kuzu_value()),
                    ("position", position.to_kuzu_value()),
                ];
                connection.execute(statement, params)?;
            }
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

/// A trait for representing Stencila Schema nodes in a Kuzu database
///
/// The implementation of this trait is generated, by the `schema-gen` crate
/// for each node type. To minimize compiled code size, we use enums such as
/// [`NodeType`] and [`NodeProperty`] for the return types of the trait's methods.
#[allow(clippy::type_complexity)]
trait DatabaseNode {
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
    fn rel_tables(&self) -> Vec<(NodeProperty, Vec<(NodeType, NodeId, usize)>)>;
}

/// A visitor which collects entries for node and relation tables from a node
///
/// Walks over a [`Node`] and collects the results of [`DatabaseNode::node_table`] and
/// [`DatabaseNode::rel_tables`]. These results are normalized into the `node_tables` and
/// `rel_tables` hash maps which are optimized for having one prepared statement for
/// each entry.
#[derive(Default)]
#[allow(clippy::type_complexity)]
struct DatabaseWalker {
    node_tables: HashMap<NodeType, (Vec<(NodeProperty, LogicalType)>, Vec<(NodeId, Vec<Value>)>)>,

    rel_tables: HashMap<(NodeType, NodeProperty, NodeType), Vec<(NodeId, Vec<(NodeId, usize)>)>>,
}

impl DatabaseWalker {
    /// Visit a [`DatabaseNode`] and insert the results of [`DatabaseNode::node_table`] and
    /// [`DatabaseNode::rel_tables`] into `node_tables` and `rel_tables`.
    fn visit_database_node<T>(&mut self, node: &T)
    where
        T: DatabaseNode,
    {
        let node_type = node.node_type();
        let node_id = node.node_id();
        let node_table = node.node_table();
        let rel_tables = node.rel_tables();

        self.node_tables
            .entry(node_type)
            .or_insert_with(|| {
                (
                    node_table
                        .iter()
                        .map(|(node_property, logical_type, ..)| {
                            (*node_property, logical_type.clone())
                        })
                        .collect(),
                    Vec::new(),
                )
            })
            .1
            .push((
                node_id.clone(),
                node_table.into_iter().map(|(.., value)| value).collect(),
            ));

        for (node_property, to_nodes) in rel_tables {
            let mut to_node_ids: HashMap<NodeType, Vec<(NodeId, usize)>> = HashMap::new();
            for (to_node_type, to_node_id, position) in to_nodes {
                to_node_ids
                    .entry(to_node_type)
                    .or_default()
                    .push((to_node_id, position));
            }

            for (to_node_type, to_node_ids) in to_node_ids {
                self.rel_tables
                    .entry((node_type, node_property, to_node_type))
                    .or_default()
                    .push((node_id.clone(), to_node_ids));
            }
        }
    }
}

impl Visitor for DatabaseWalker {
    fn visit_node(&mut self, node: &Node) -> WalkControl {
        self.visit_database_node(node);
        WalkControl::Continue
    }

    fn visit_block(&mut self, node: &Block) -> WalkControl {
        self.visit_database_node(node);
        WalkControl::Continue
    }

    fn visit_inline(&mut self, node: &Inline) -> WalkControl {
        self.visit_database_node(node);
        WalkControl::Continue
    }
}

#[cfg(test)]
mod tests {
    use schema::shortcuts::{art, p, stg, t};

    use super::*;

    #[test]
    fn create_nodes() -> Result<()> {
        let mut db = NodeDatabase::in_memory()?;

        let art = art([
            p([t("Para "), stg([t("one")]), t(".")]),
            p([t("Para two.")]),
        ]);
        db.create(&art)?;

        Ok(())
    }
}
