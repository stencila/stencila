use std::{
    ops::Deref,
    str::FromStr,
    sync::{Arc, Mutex as SyncMutex, MutexGuard as SyncMutexGuard},
};

use codec_text_trait::to_text;
use kernel_docsdb::DocsDBKernelInstance;
use kernel_jinja::{
    kernel::{
        common::{
            eyre::Result,
            inflector::Inflector,
            itertools::Itertools,
            once_cell::sync::Lazy,
            regex::{Captures, Regex},
            serde_json,
            tokio::{runtime, sync::Mutex, task},
            tracing,
        },
        schema::{
            get, CodeChunk, ExecutionMessage, MessageLevel, Node, NodePath, NodeProperty, NodeSet,
        },
        KernelInstance,
    },
    minijinja::{
        value::{from_args, DynObject, Enumerator, Kwargs, Object, ObjectRepr},
        Error, ErrorKind, State, Value,
    },
};

/// Transform property filter arguments into valid MiniJinja keyword arguments
///
/// Uses single digit codes and spacing to ensure that the code stays the same length.
pub(super) fn transform_filters(code: &str) -> String {
    static FILTERS: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"@([a-zA-Z][\w_]*)\s*(==|\!=|<=|<|>=|>|=\~|\~=|\!\~|\^=|\$=|in|=)\s*")
            .expect("invalid regex")
    });

    FILTERS
        .replace_all(code, |captures: &Captures| {
            let var = &captures[1];
            let op = match &captures[2] {
                "=" | "==" => "",
                "!=" => "0",
                "<" => "1",
                "<=" => "2",
                ">" => "3",
                ">=" => "4",
                "~=" | "=~" => "5",
                "!~" => "6",
                "^=" => "7",
                "$=" => "8",
                "in" => "9",
                echo => echo,
            };

            let spaces = captures[0].len().saturating_sub(var.len() + op.len() + 1);
            let spaces = " ".repeat(spaces);

            [var, op, &spaces, "="].concat()
        })
        .into()
}

/// Translate a filter into a Cypher `WHERE` clause
fn apply_filter(alias: &str, property: &str, value: Value) -> String {
    let mut chars = property.chars().collect_vec();

    let last = *chars.last().expect("always has at least one char");
    if last.is_numeric() {
        chars.pop();
    }

    let col = || [&alias, ".", &chars.iter().join("").to_camel_case()].concat();

    let val_str = || ["'", &value.to_string(), "'"].concat();

    let val_lit = || {
        if value.as_str().is_some() {
            val_str()
        } else {
            value.to_string()
        }
    };

    match last {
        '5' => ["regexp_matches(", &col(), ", ", &val_str(), ")"].concat(),
        '6' => ["NOT regexp_matches(", &col(), ", ", &val_str(), ")"].concat(),
        '7' => ["starts_with(", &col(), ", ", &val_str(), ")"].concat(),
        '8' => ["ends_with(", &col(), ", ", &val_str(), ")"].concat(),
        '9' => {
            if value.as_str().is_some() {
                ["contains(", &val_str(), ", ", &col(), ")"].concat()
            } else {
                [&col(), "IN ", &val_lit()].concat()
            }
        }
        _ => {
            let op = match last {
                '0' => "!=",
                '1' => "<",
                '2' => "<=",
                '3' => ">",
                '4' => ">=",
                _ => "=",
            };
            [&col(), " ", op, " ", &val_lit()].concat()
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Query {
    /// The name of the database
    db_name: String,

    /// The database to query
    db: Arc<Mutex<DocsDBKernelInstance>>,

    /// Execution messages to be added to when executing the query
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,

    /// The Cypher for the query
    cypher: Option<String>,

    /// Any `CALL` for the query
    call: Option<String>,

    /// Any `MATCH` pattern for the query
    pattern: Option<String>,

    /// Condition that should be `AND`ed in the `WHERE` clause
    ands: Vec<String>,

    /// Condition that should be `OR`ed in the `WHERE` clause
    ors: Vec<String>,

    /// Any `RETURN` clause
    r#return: Option<String>,

    /// Whether any `RETURN` clause has an `DISTINCT` modifier
    return_distinct: Option<bool>,

    /// Any `ORDER BY` clause
    order_by: Option<String>,

    /// Any `ORDER BY` order
    order_by_order: Option<String>,

    /// Any `SKIP` clause
    skip: Option<usize>,

    /// Any `LIMIT` clause
    limit: Option<usize>,

    /// Any `UNION` clause
    union: Option<Box<Query>>,

    /// Whether any `UNION` clause has an `ALL` modifier
    union_all: bool,

    /// Whether the `return` method has been used
    return_used: bool,

    /// Whether the `match` method has been used
    match_used: bool,

    /// Whether one of the node table methods has been used
    node_table_used: Option<String>,
}

impl Query {
    /// Create a new query on
    pub fn new(
        db_name: String,
        db: Arc<Mutex<DocsDBKernelInstance>>,
        messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,
    ) -> Self {
        Self {
            db_name,
            db,
            messages,
            cypher: None,
            call: None,
            pattern: None,
            ands: Vec::new(),
            ors: Vec::new(),
            r#return: None,
            return_distinct: None,
            order_by: None,
            order_by_order: None,
            skip: None,
            limit: None,
            union: None,
            union_all: false,
            return_used: false,
            match_used: false,
            node_table_used: None,
        }
    }

    /// Specify the entire Cypher query manually
    fn cypher(&self, cypher: String) -> Result<Self, Error> {
        let mut query = self.clone();
        query.cypher = Some(cypher);
        Ok(query)
    }

    /// Apply a `MATCH` pattern
    fn r#match(&self, pattern: String) -> Result<Self, Error> {
        if let Some(table) = &self.node_table_used {
            let method = table.to_lowercase().to_plural();
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("`match` method can not be used with node type method `{method}`"),
            ));
        }

        let mut query = self.clone();
        query.pattern = Some(pattern);
        query.match_used = true;
        Ok(query)
    }

    /// Add to the `MATCH` pattern for a node table
    ///
    /// Adds the appropriate `-[relation]->(node)` to the pattern and
    /// makes the corresponding alias the `RETURN`.
    fn table(&self, method: &str, kwargs: Kwargs) -> Result<Self, Error> {
        if self.match_used {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("node type method `{method}` can not be used with `match` method"),
            ));
        }
        if self.return_used {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("`return` method should come after any node type method `{method}`"),
            ));
        }

        let mut query = self.clone();

        let mut alias = method.to_singular();
        let mut table = match method {
            "rows" => "TableRow".to_string(),
            "cells" => "TableCell".to_string(),
            _ => alias.to_pascal_case(),
        };

        // Escape reserved words
        if alias == "table" {
            alias = ["`", &alias, "`"].concat();
        }
        if table == "Table" {
            table = ["`", &table, "`"].concat();
        }

        let node = ["(", &alias, ":", &table, ")"].concat();

        query.pattern = Some(match query.pattern {
            Some(pattern) => {
                let prev_table = self.node_table_used.as_deref().unwrap_or_default();
                let relation = match (prev_table, table.as_str()) {
                    ("Table", "TableRow") => "[:rows]",
                    ("TableRow", "TableCell") => "[:cells]",
                    ("Table", "TableCell") => "[:rows]-[:cells]",
                    _ => "[:content* acyclic]",
                };
                [&pattern, "-", relation, "->", &node].concat()
            }
            None => node,
        });

        for arg in kwargs.args() {
            // Ensure all arguments are defined
            let value: Value = kwargs.get(arg)?;
            if value.is_undefined() {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("value for argument `{arg}` is undefined"),
                ));
            }

            match arg {
                "search" | "searchAll" | "and" | "or" => {
                    // Non-filter arguments: should be string
                    let value = value
                        .as_str()
                        .ok_or_else(|| {
                            Error::new(
                                ErrorKind::InvalidOperation,
                                format!("argument `{arg}` is should be a string"),
                            )
                        })?
                        .to_string();

                    match arg {
                        "search" | "searchAll" => {
                            let table = table.replace("`", "");
                            let option = if arg == "searchAll" {
                                ", conjunctive := true"
                            } else {
                                ""
                            };
                            query.call = Some(format!(
                                "CALL QUERY_FTS_INDEX('{table}', 'fts', '{value}'{option})",
                            ));
                        }
                        "and" => query.ands.push(value),
                        "or" => query.ors.push(value),
                        _ => unreachable!(),
                    }
                }
                _ => {
                    let filter = apply_filter(&alias, &arg, value);
                    query.ands.push(filter)
                }
            }
        }

        query.r#return = Some(alias);

        query.node_table_used = Some(table);

        Ok(query)
    }

    /// Add a `AND`ed condition to the `WHERE` to query
    fn and(&self, condition: String) -> Result<Self, Error> {
        let mut query = self.clone();
        query.ands.push(condition);
        Ok(query)
    }

    /// Add an `OR`ed condition to the `WHERE` to query
    fn or(&self, condition: String) -> Result<Self, Error> {
        let mut query = self.clone();
        query.ors.push(condition);
        Ok(query)
    }

    /// Apply a `RETURN` clause to query
    ///
    /// The default is `RETURN <table>` where <table> was the last table used in the method chain.
    /// This makes sense for most queries but this method allows the user to override that if desired.
    fn r#return(&self, what: String, distinct: Option<bool>) -> Result<Self, Error> {
        let mut query = self.clone();

        query.r#return = Some(what);
        query.return_distinct = distinct;
        query.return_used = true;

        Ok(query)
    }

    /// Apply an `ORDER BY` clause to query
    fn order_by(&self, order_by: String, order: Option<String>) -> Result<Self, Error> {
        let mut query = self.clone();

        static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\w+\.\w+").expect("invalid regex"));
        if !REGEX.is_match(&order_by) {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "first argument should have form 'name.property' e.g 'article.datePublished'",
            ));
        }
        query.order_by = Some(order_by);

        if let Some(order) = order {
            if !["ASC", "DESC"].contains(&order.to_uppercase().as_str()) {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "second argument should be one of 'ASC' or 'DESC'",
                ));
            }
            query.order_by_order = Some(order);
        }

        Ok(query)
    }

    /// Apply a `SKIP` clause to query
    fn skip(&self, count: usize) -> Self {
        let mut query = self.clone();
        query.skip = Some(count);
        query
    }

    /// Apply a `LIMIT` clause to query
    fn limit(&self, count: usize) -> Self {
        let mut query = self.clone();
        query.limit = Some(count);
        query
    }

    /// Apply a `UNION` clause to query
    fn union(&self, other: DynObject, all: Option<bool>) -> Result<Self, Error> {
        let Some(other) = other.downcast_ref::<Query>() else {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("first argument should be another query"),
            ));
        };

        let mut query = self.clone();
        query.union = Some(Box::new(other.clone()));
        query.union_all = all.unwrap_or_default();
        Ok(query)
    }

    /// Generate a Cypher query for the query
    pub fn generate(&self) -> String {
        if let Some(cypher) = &self.cypher {
            return cypher.clone();
        }

        let mut cypher = if let Some(call) = &self.call {
            call.to_string()
        } else {
            let pattern = self.pattern.as_deref().unwrap_or("(node)");
            format!("MATCH {pattern}")
        };

        if !(self.ands.is_empty() && self.ors.is_empty()) {
            cypher.push_str("\nWHERE ");

            cypher.push_str(&self.ands.join("\n  AND "));

            if !self.ands.is_empty() && !self.ors.is_empty() {
                cypher.push_str("\n  OR ");
            }

            cypher.push_str(
                &self
                    .ors
                    .iter()
                    .map(|clause| ["(", clause, ")"].concat())
                    .join("\n  OR "),
            );
        }

        if self.call.is_some() {
            if let Some(table) = &self.node_table_used {
                let mut alias = table.to_camel_case();
                if alias == "table" {
                    alias = "table".to_string();
                } else if alias == "tableCell" {
                    alias = "cell".to_string();
                } else if alias == "tableRow" {
                    alias = "row".to_string();
                }
                cypher = cypher.replace(&[&alias, "."].concat(), "node.");
            }
            cypher.push_str("\nRETURN node");
        } else {
            cypher.push_str("\nRETURN ");
            if self.return_distinct.unwrap_or_default() {
                cypher.push_str("DISTINCT ");
            }
            let r#return = self.r#return.as_deref().unwrap_or("*");
            cypher.push_str(r#return);
        }

        if let Some(order_by) = &self.order_by {
            cypher.push_str("\nORDER BY ");
            cypher.push_str(order_by);
            if let Some(order_by_order) = &self.order_by_order {
                cypher.push(' ');
                cypher.push_str(order_by_order);
            }
        }

        if let Some(skip) = &self.skip {
            cypher.push_str("\nSKIP ");
            cypher.push_str(&skip.to_string());
        }

        if let Some(limit) = &self.limit {
            cypher.push_str("\nLIMIT ");
            cypher.push_str(&limit.to_string());
        } else {
            cypher.push_str("\nLIMIT 10");
        }

        if let Some(other) = &self.union {
            cypher.push_str("\nUNION");
            if self.union_all {
                cypher.push_str(" ALL");
            }
            cypher.push('\n');
            cypher.push_str(&other.generate());
        }

        cypher
    }

    /// Return the generated Cypher as an executable `CodeChunk`
    ///
    /// Mainly intended for debugging.
    fn explain(&self) -> Value {
        let cypher = self.generate();

        let code = ["// @", &self.db_name, "\n", &cypher].concat();

        let node = Node::CodeChunk(CodeChunk {
            code: code.into(),
            programming_language: Some("docsdb".into()),
            is_echoed: Some(true), // To make visible
            ..Default::default()
        });

        Value::from_object(NodeProxy::new(node, self.messages.clone()))
    }

    /// Execute the query in the kernel
    pub fn nodes(&self) -> Vec<Node> {
        let cypher = self.generate();

        let db = self.db.clone();
        let cypher_clone = cypher.clone();
        let (outputs, mut messages) = match task::block_in_place(move || {
            runtime::Handle::current()
                .block_on(async move { db.lock().await.execute(&cypher_clone).await })
        }) {
            Ok(result) => result,
            Err(error) => (
                Vec::new(),
                vec![ExecutionMessage {
                    level: MessageLevel::Exception,
                    message: error.to_string(),
                    ..Default::default()
                }],
            ),
        };

        if !messages.is_empty() {
            if let Some(first) = messages.first_mut() {
                let trace = first.stack_trace.get_or_insert_default();
                trace.push_str("While executing Cypher:\n\n");
                trace.push_str(&cypher);
            }
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.append(&mut messages)
            };
        }

        outputs
    }

    /// Execute and return a [`NodeProxies`] for all nodes
    fn all(&self) -> Value {
        Value::from_object(NodeProxies::new(self.nodes(), self.messages.clone()))
    }

    /// Execute and return a [`NodeProxies`] for all nodes
    fn slice(&self, first: i32, last: Option<i32>) -> Result<Value, Error> {
        if let Some(last) = last {
            if last < first {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "Second argument should be greater than or equal to first",
                ));
            }
        }

        let nodes = if first >= 0 && (last.is_none() || last.unwrap_or_default() >= 0) {
            let mut query = if first > 0 {
                self.skip(first as usize)
            } else {
                self.clone()
            };

            if let Some(last) = last {
                query.limit = Some((last - first + 1) as usize);
            }

            query.nodes()
        } else {
            let nodes = self.nodes();

            let first = if first < 0 {
                let first = nodes.len() as i32 + first;
                if first < 0 {
                    0usize
                } else {
                    first as usize
                }
            } else {
                first as usize
            };

            let last = last.unwrap_or_else(|| nodes.len() as i32);
            let last = if last < 0 {
                let last = nodes.len() as i32 + last;
                if last < 0 {
                    0usize
                } else {
                    last as usize
                }
            } else {
                last as usize
            };

            nodes
                .into_iter()
                .skip(first)
                .take(last - first + 1)
                .collect()
        };

        Ok(Value::from_object(NodeProxies::new(
            nodes,
            self.messages.clone(),
        )))
    }

    /// Execute with `LIMIT 1` and return a [`NodeProxy`] for first node in result
    fn first(&self) -> Result<Value, Error> {
        let query = self.limit(1);
        match query.nodes().into_iter().next() {
            Some(node) => Ok(Value::from_object(NodeProxy::new(
                node,
                self.messages.clone(),
            ))),
            None => Err(Error::new(
                ErrorKind::InvalidOperation,
                "Empty result set so cannot get first node",
            )),
        }
    }

    /// Execute and return a [`NodeProxy`] for last node in result
    fn last(&self) -> Result<Value, Error> {
        match self.nodes().into_iter().last() {
            Some(node) => Ok(Value::from_object(NodeProxy::new(
                node,
                self.messages.clone(),
            ))),
            None => Err(Error::new(
                ErrorKind::InvalidOperation,
                "Empty result set so cannot get last node",
            )),
        }
    }
}

impl Object for Query {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let query = match name {
            // Core Cypher query building methods
            "cypher" => {
                let (cypher,) = from_args(args)?;
                self.cypher(cypher)?
            }
            "match" => {
                let (pattern,) = from_args(args)?;
                self.r#match(pattern)?
            }
            "and" | "where" => {
                let (condition,) = from_args(args)?;
                self.and(condition)?
            }
            "or" => {
                let (condition,) = from_args(args)?;
                self.or(condition)?
            }
            "return" => {
                let (r#return, distinct) = from_args(args)?;
                self.r#return(r#return, distinct)?
            }
            "order_by" | "orderBy" => {
                let (order_by, order) = from_args(args)?;
                self.order_by(order_by, order)?
            }
            "skip" => {
                let (skip,) = from_args(args)?;
                self.skip(skip)
            }
            "limit" => {
                let (limit,) = from_args(args)?;
                self.limit(limit)
            }
            "union" => {
                let (union, all) = from_args(args)?;
                self.union(union, all)?
            }
            // Return the generated Cypher
            "explain" => return Ok(self.explain()),
            // Methods that execute the query and return node proxies
            "all" | "one" | "first" | "last" => {
                if !args.is_empty() {
                    return Err(Error::new(
                        ErrorKind::TooManyArguments,
                        format!("Method `{name}` takes no arguments."),
                    ));
                }
                let result = match name {
                    "all" => self.all(),
                    "one" | "first" => self.first()?,
                    "last" => self.last()?,
                    _ => unreachable!(),
                };
                return Ok(result);
            }
            "slice" => {
                let (first, last) = from_args(args)?;
                return self.slice(first, last);
            }
            // Fallback to node adding a MATCH pattern for a node table
            _ => {
                let (kwargs,) = from_args(args)?;
                self.table(name, kwargs)?
            }
        };
        Ok(Value::from_object(query))
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(property) = key.as_str() {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    format!("A query does not have property `{property}`"),
                ))
            };
        }

        let Some(mut key) = key.as_i64() else {
            return None;
        };

        let mut query = if key > 0 {
            self.limit(1)
        } else {
            self.deref().clone()
        };

        if key > 0 && query.skip.is_none() {
            query.skip = Some(key as usize);
            key = 0;
        }

        let mut nodes = query.nodes();

        let index = if key < 0 {
            let first = nodes.len() as i64 + key;
            if first < 0 {
                0usize
            } else {
                first as usize
            }
        } else {
            key as usize
        };

        if index >= nodes.len() {
            return None;
        }

        let node = nodes.swap_remove(index as usize);
        Some(Value::from_object(NodeProxy::new(
            node,
            self.messages.clone(),
        )))
    }
}

/// A proxy for a [`Node`] to allow it to be accessed as a minijinja [`Value`]
///
/// This has several advantage over simply converting all nodes to values
/// via `serde_json`:
///
/// 1. We can provide getters for derived properties such as `text`
///
/// 2. We can create an error message if a non-existent property is accessed
///
/// 3. We can chain proxies together and convert to a minijinja value only
///    when appropriate e.g. for primitives
#[derive(Debug, Clone)]
pub(super) struct NodeProxy {
    /// The node being proxied
    node: Node,

    /// Execution messages to be added to when accessing the node
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,
}

impl NodeProxy {
    pub fn new(node: Node, messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self { node, messages }
    }

    pub fn nodes(&self) -> Vec<Node> {
        vec![self.node.clone()]
    }
}

impl Object for NodeProxy {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Map
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if key.is_integer() {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    "Cannot index a single node".into(),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        let Some(property) = key.as_str() else {
            return None;
        };

        if property == "type" {
            return Some(Value::from(self.node.node_type().to_string()));
        }

        let Ok(property) = NodeProperty::from_str(property) else {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    format!("Invalid node property `{property}`"),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        let Ok(property) = get(&self.node, NodePath::from(property)) else {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    format!(
                        "`{property}` is not a property of node type `{}`",
                        self.node.node_type()
                    ),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        match node_set_to_value(property, &self.messages) {
            Ok(value) => Some(value),
            Err(error) => {
                tracing::error!("While converting node to minijinja value: {error}");
                None
            }
        }
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        if method == "text" {
            if !args.is_empty() {
                Err(Error::new(
                    ErrorKind::TooManyArguments,
                    "Method `text` takes no arguments.",
                ))
            } else {
                Ok(Value::from(to_text(&self.node)))
            }
        } else {
            Err(Error::new(
                ErrorKind::UnknownMethod,
                format!("Method `{method}` takes no arguments."),
            ))
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct NodeProxies {
    /// The nodes being proxied
    nodes: Vec<Node>,

    /// Execution messages to be added to when accessing the nodes
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,
}

impl NodeProxies {
    pub fn new(nodes: Vec<Node>, messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self { nodes, messages }
    }

    pub fn nodes(&self) -> Vec<Node> {
        self.nodes.clone()
    }
}

impl Object for NodeProxies {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Seq
    }

    fn enumerate(self: &Arc<Self>) -> Enumerator {
        Enumerator::Seq(self.nodes.len())
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(property) = key.as_str() {
            if let Some(mut msgs) = lock_messages(&self.messages) {
                msgs.push(ExecutionMessage::new(
                    MessageLevel::Error,
                    format!("`{property}` is not a property of a node list"),
                ))
            };
            return Some(Value::UNDEFINED);
        };

        let Some(key) = key.as_i64() else {
            return None;
        };

        let index = if key < 0 {
            self.nodes.len() as i64 - key - 1
        } else {
            key
        };

        if index < 0 || index >= self.nodes.len() as i64 {
            return None;
        }

        let node = self.nodes[index as usize].clone();
        Some(Value::from_object(NodeProxy::new(
            node,
            self.messages.clone(),
        )))
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        method: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        if method == "text" {
            if !args.is_empty() {
                Err(Error::new(
                    ErrorKind::TooManyArguments,
                    "Method `text` takes no arguments.",
                ))
            } else {
                Ok(Value::from(to_text(&self.nodes)))
            }
        } else {
            Err(Error::new(
                ErrorKind::UnknownMethod,
                format!("Method `{method}` takes no arguments."),
            ))
        }
    }
}

fn node_set_to_value(
    node_set: NodeSet,
    messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>,
) -> Result<Value> {
    match node_set {
        NodeSet::One(node) => node_to_value(node, messages),
        NodeSet::Many(nodes) => Ok(Value::from_object(NodeProxies::new(
            nodes,
            messages.clone(),
        ))),
    }
}

fn node_to_value(node: Node, messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Result<Value> {
    match node {
        Node::Null(..) => Ok(Value::from(())),
        Node::Boolean(node) => Ok(Value::from(node)),
        Node::Integer(node) => Ok(Value::from(node)),
        Node::UnsignedInteger(node) => Ok(Value::from(node)),
        Node::Number(node) => Ok(Value::from(node)),
        Node::String(node) => Ok(Value::from(node)),
        Node::Array(..) | Node::Object(..) => node_to_value_via_serde(node),
        _ => Ok(Value::from_object(NodeProxy::new(node, messages.clone()))),
    }
}

fn node_to_value_via_serde(node: Node) -> Result<Value> {
    let value = serde_json::to_value(node)?;
    Ok(serde_json::from_value(value)?)
}

fn lock_messages<'m>(
    messages: &'m SyncMutex<Vec<ExecutionMessage>>,
) -> Option<SyncMutexGuard<'m, Vec<ExecutionMessage>>> {
    match messages.lock() {
        Ok(messages) => Some(messages),
        Err(..) => {
            tracing::error!("Unable to lock messages");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn transform() {
        use super::transform_filters as t;

        assert_eq!(t(""), "");
        assert_eq!(t("@a"), "@a");

        assert_eq!(t("@a = 1"), "a   =1");
        assert_eq!(t("@a= 1"), "a  =1");
        assert_eq!(t("@a =1"), "a  =1");
        assert_eq!(t("@a=1"), "a =1");

        assert_eq!(t("@a == 1"), "a    =1");
        assert_eq!(t("@a== 1"), "a   =1");
        assert_eq!(t("@a ==1"), "a   =1");
        assert_eq!(t("@a==1"), "a  =1");

        assert_eq!(t("@a < 1"), "a1  =1");
        assert_eq!(t("@a< 1"), "a1 =1");
        assert_eq!(t("@a <1"), "a1 =1");
        assert_eq!(t("@a<1"), "a1=1");

        assert_eq!(t("@abc !~ 'regex'"), "abc6   ='regex'");
        assert_eq!(t("@abc!~ 'regex'"), "abc6  ='regex'");
        assert_eq!(t("@abc !~'regex'"), "abc6  ='regex'");
        assert_eq!(t("@abc!~'regex'"), "abc6 ='regex'");

        assert_eq!(t("@a < 1"), "a1  =1");
        assert_eq!(t("@a <= 1"), "a2   =1");
        assert_eq!(t("@a > 1"), "a3  =1");
        assert_eq!(t("@a >= 1"), "a4   =1");
        assert_eq!(t("@a =~ 1"), "a5   =1");
        assert_eq!(t("@a !~ 1"), "a6   =1");
        assert_eq!(t("@a ^= 1"), "a7   =1");
        assert_eq!(t("@a $= 1"), "a8   =1");
        assert_eq!(t("@a in 1"), "a9   =1");
    }
}
