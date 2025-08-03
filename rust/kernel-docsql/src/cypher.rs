use std::sync::{Arc, Mutex as SyncMutex};

use codec_text_trait::to_text;
use codecs;
use kernel_docsdb::{DocsDBKernelInstance, QueryResultTransform};
use kernel_jinja::{
    kernel::{
        KernelInstance,
        common::{
            eyre::Result,
            inflector::Inflector,
            itertools::Itertools,
            tokio::{runtime, sync::Mutex, task},
        },
        schema::{
            Array, CodeChunk, ExecutionMessage, MessageLevel, Node, NodeType, Primitive,
            PropertyValue, PropertyValueOrString, SectionType,
        },
    },
    minijinja::{
        Environment, Error, ErrorKind, State, Value,
        value::{DynObject, Kwargs, Object, from_args},
    },
};

use crate::{
    NodeProxies, NodeProxy,
    docsql::GLOBAL_CONSTS,
    extend_messages,
    github::GitHubQuery,
    lock_messages,
    nodes::{all, first, get, last},
    openalex::OpenAlexQuery,
    subquery::Subquery,
    try_messages,
};

/// A document shortcut functions to the environment
pub(super) fn add_document_functions(env: &mut Environment, document: Arc<CypherQuery>) {
    for (name, node_type) in [
        // Static code
        ("codeBlock", NodeType::CodeBlock),
        ("codeInline", NodeType::CodeInline),
        // Executable code
        ("codeChunk", NodeType::CodeChunk),
        ("chunk", NodeType::CodeChunk),
        ("codeExpression", NodeType::CodeExpression),
        ("expressions", NodeType::CodeExpression),
        // Math
        ("mathBlock", NodeType::MathBlock),
        ("mathInline", NodeType::MathInline),
        // Media
        ("image", NodeType::ImageObject),
        ("audio", NodeType::AudioObject),
        ("video", NodeType::VideoObject),
        // Containers
        ("admonition", NodeType::Admonition),
        ("claim", NodeType::Claim),
        ("heading", NodeType::Heading),
        ("list", NodeType::List),
        ("paragraph", NodeType::Paragraph),
        ("section", NodeType::Section),
        // Note: at present, mainly for performance reasons, the current document is not
        // "sentencized" so `sentence` and `sentences` function are not provided here.
        // Metadata
        ("organization", NodeType::Organization),
        ("person", NodeType::Person),
        ("reference", NodeType::Reference),
    ] {
        env.add_global(
            name,
            Value::from_object(CypherQueryNodeType::new(node_type, document.clone(), true)),
        );
        env.add_global(
            match name {
                "person" => "people".to_string(),
                _ => [name, "s"].concat(),
            },
            Value::from_object(CypherQueryNodeType::new(node_type, document.clone(), false)),
        );
    }

    for name in ["figure", "table", "equation"] {
        env.add_global(
            name,
            Value::from_object(CypherQueryLabelled::new(name, document.clone(), true)),
        );
        env.add_global(
            [name, "s"].concat(),
            Value::from_object(CypherQueryLabelled::new(name, document.clone(), false)),
        );
    }

    for (name, section_type) in [
        ("introduction", SectionType::Introduction),
        ("methods", SectionType::Methods),
        ("results", SectionType::Results),
        ("discussion", SectionType::Discussion),
    ] {
        env.add_global(
            name,
            Value::from_object(CypherQuerySectionType::new(section_type, document.clone())),
        );
    }

    env.add_global(
        "variable",
        Value::from_object(CypherQueryVariables::new(document.clone(), true)),
    );
    env.add_global(
        "variables",
        Value::from_object(CypherQueryVariables::new(document.clone(), false)),
    );
}

#[derive(Debug, Clone)]
pub(super) struct CypherQuery {
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

    /// Parameters of the query
    ///
    /// Used to pass embedding vectors through to the [`DocsDBKernelInstance`] by
    /// setting calling `set`.
    parameters: Vec<Node>,

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

    /// Any `UNION` clauses
    union: Vec<CypherQuery>,

    /// Whether any `UNION` clause has an `ALL` modifier
    union_all: bool,

    /// The output type of the query
    out: Option<QueryResultTransform>,

    /// Whether the `return` method has been used
    return_used: bool,

    /// Whether the `match` method has been used
    match_used: bool,

    /// Whether one of the node table methods has been used
    node_table_used: Option<String>,
}

impl CypherQuery {
    /// Create a new query
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
            parameters: Vec::new(),
            ands: Vec::new(),
            ors: Vec::new(),
            r#return: None,
            return_distinct: None,
            order_by: None,
            order_by_order: None,
            skip: None,
            limit: None,
            union: Vec::new(),
            union_all: false,
            out: None,
            return_used: false,
            match_used: false,
            node_table_used: None,
        }
    }

    /// Whether this is the base query for which no method has been called yet
    pub fn is_base(&self) -> bool {
        self.cypher.is_none()
            && self.call.is_none()
            && self.pattern.is_none()
            && self.ands.is_empty()
            && self.ors.is_empty()
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
    fn table(&self, method: &str, args: &[Value], kwargs: Kwargs) -> Result<Self, Error> {
        if self.match_used {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("node type method `{method}` can not be used with `match` method"),
            ));
        }

        let mut query = self.clone();

        let (table, and) = table_for_method(method);
        if let Some(and) = and {
            query.ands.push(and);
        }

        let alias = alias_for_table(&table);
        let node = ["(", &alias, ":", &table, ")"].concat();

        query.pattern = Some(match query.pattern {
            Some(pattern) => {
                let prev_table = self.node_table_used.as_deref().unwrap_or_default();
                let relation = relation_between_tables(prev_table, &table);
                [&pattern, "-", &relation, "->", &node].concat()
            }
            None => node,
        });

        for (index, arg) in args.iter().enumerate() {
            let index = index + 1;

            if arg.is_undefined() {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("argument {index} is undefined"),
                ));
            };

            let Some(arg) = arg.as_str() else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("argument {index} is not a string"),
                ));
            };

            match arg {
                "above" | "below" => {
                    let op = if arg == "above" { "<" } else { ">" };
                    query
                        .ands
                        .push([&alias, ".position", op, "$currentPosition"].concat());

                    // Ordering by position is important, particularly for `above`
                    // where the ordering needs to be descending
                    query.order_by = Some([&alias, ".position"].concat());
                    if arg == "above" {
                        query.order_by_order = Some("DESC".to_string());
                    }
                }

                "return" => {
                    if self.return_used {
                        return Err(Error::new(
                            ErrorKind::InvalidOperation,
                            "`return` already specified".to_string(),
                        ));
                    }
                    query.r#return = Some(alias.to_string());
                    query.return_used = true;
                }

                _ => {
                    return Err(Error::new(
                        ErrorKind::TooManyArguments,
                        format!("unrecognized argument `{arg}`"),
                    ));
                }
            }
        }

        for arg_name in kwargs.args() {
            // Ensure all arguments are defined
            let arg_value: Value = kwargs.get(arg_name)?;
            if arg_value.is_undefined() {
                return Err(Error::new(
                    ErrorKind::UndefinedError,
                    format!("value for argument `{arg_name}` is undefined"),
                ));
            }

            match arg_name {
                "like" => {
                    let text = if let Some(text) = arg_value.as_str() {
                        text.to_string()
                    } else if let Some(query) = arg_value.downcast_object::<CypherQuery>() {
                        query.text()?.to_string()
                    } else if let Some(nodes) = arg_value.downcast_object::<NodeProxies>() {
                        nodes.text()?.to_string()
                    } else if let Some(node) = arg_value.downcast_object::<NodeProxy>() {
                        node.text()?.to_string()
                    } else {
                        return Err(Error::new(
                            ErrorKind::InvalidOperation,
                            "unexpected type for argument `like`".to_string(),
                        ));
                    };

                    let embeddings = embed::query(&text).map_err(|error| {
                        Error::new(
                            ErrorKind::InvalidOperation,
                            format!("while generating embeddings: {error}"),
                        )
                    })?;
                    let embeddings = embeddings
                        .into_iter()
                        .map(|value| Primitive::Number(value as f64))
                        .collect_vec();

                    query.parameters.push(Node::Array(Array(embeddings)));
                    let par = query.parameters.len();

                    let table = table.replace("`", "");
                    query.call = Some(format!(
                        "CALL QUERY_VECTOR_INDEX('{table}', 'vector', $par{par}, 10)",
                    ));
                    query.order_by = Some("distance".to_string());
                }
                "search" | "searchAll" | "and" | "or" => {
                    // Argument value should be string
                    let value = arg_value
                        .as_str()
                        .ok_or_else(|| {
                            Error::new(
                                ErrorKind::InvalidOperation,
                                format!("argument `{arg_name}` should be a string"),
                            )
                        })?
                        .to_string();

                    match arg_name {
                        "search" | "searchAll" => {
                            let table = table.replace("`", "");
                            let option = if arg_name == "searchAll" {
                                ", conjunctive := true"
                            } else {
                                ""
                            };
                            query.call = Some(format!(
                                "CALL QUERY_FTS_INDEX('{table}', 'fts', '{value}'{option})",
                            ));
                            query.order_by = Some("score".to_string());
                            query.order_by_order = Some("DESC".to_string());
                        }
                        "and" => query.ands.push(value),
                        "or" => query.ors.push(value),
                        _ => unreachable!(),
                    }
                }
                _ => {
                    let filter = apply_filter(&alias, arg_name, arg_value, false)?;
                    query.ands.push(filter)
                }
            }
        }

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
        if self.return_used {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "`return` already specified".to_string(),
            ));
        }

        let mut query = self.clone();

        query.r#return = Some(what);
        query.return_distinct = distinct;
        query.return_used = true;

        Ok(query)
    }

    /// Select columns to output in a datatable
    fn select(&self, args: &[Value], kwargs: Kwargs) -> Result<Self, Error> {
        if self.return_used {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "`return` already specified".to_string(),
            ));
        }

        let mut query = self.clone();

        let alias = query
            .node_table_used
            .as_ref()
            .map(alias_for_table)
            .unwrap_or_else(|| "node".to_string());

        let mut returns = Vec::new();

        for arg in args {
            let Some(arg) = arg.as_str() else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "arguments should be strings",
                ));
            };

            let mut column = if !arg.contains(".") {
                [&alias, ".", &escape_keyword(arg)].concat()
            } else {
                escape_keyword(arg)
            };

            if !(arg.contains(['*']) || arg.to_uppercase().contains(" AS ")) {
                // Always escape the the arg because it might be an expression
                // such as `familyNames[1]`
                column.push_str(" AS `");
                column.push_str(arg);
                column.push('`');
            }

            returns.push(column);
        }

        for arg in kwargs.args() {
            let Ok(label) = kwargs.get::<&str>(arg) else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "keywords arguments should be string labels",
                ));
            };

            returns.push([&alias, ".", arg, " AS ", &escape_keyword(label)].concat());
        }

        let returns = if returns.is_empty() {
            [&alias, ".*"].concat()
        } else {
            returns.join(", ")
        };

        query.out = Some(QueryResultTransform::Datatable);
        query.r#return = Some(returns);
        query.return_used = true;

        Ok(query)
    }

    /// Set `RETURN` clause to `count(*)`
    fn count(&self) -> Result<Self, Error> {
        if self.return_used {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "`return` already specified".to_string(),
            ));
        }

        let mut query = self.clone();
        query.out = Some(QueryResultTransform::Value);
        query.r#return = Some("count(*)".into());
        query.return_used = true;

        Ok(query)
    }

    /// Apply an `ORDER BY` clause to query
    fn order_by(&self, order_by: String, order: Option<String>) -> Result<Self, Error> {
        let mut query = self.clone();

        let order_by = if !order_by.contains(".") {
            let Some(alias) = query.node_table_used.as_ref().map(alias_for_table) else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "first argument should have form 'name.property' e.g 'article.datePublished'",
                ));
            };

            [&alias, ".", &order_by].concat()
        } else {
            order_by
        };

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

    /// Apply `ORDER BY gen_random_uuid()` and `LIMIT` clauses to query
    fn sample(&self, count: Option<usize>) -> Self {
        let mut query = self.clone();
        query.order_by = Some("gen_random_uuid()".into());
        query.limit = Some(count.unwrap_or(10));
        query
    }

    /// Apply a `UNION` clause to query
    fn union(&self, other: DynObject, all: Option<bool>) -> Result<Self, Error> {
        let Some(other) = other.downcast_ref::<CypherQuery>() else {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "first argument should be another query".to_string(),
            ));
        };

        let mut query = self.clone();
        query.union.push(other.clone());
        query.union_all = all.unwrap_or_default();
        Ok(query)
    }

    /// Specify the output type for the query
    fn out(&self, out: &str) -> Result<Self, Error> {
        let mut query = self.clone();

        query.out = match out.parse() {
            Ok(value) => Some(value),
            Err(..) => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Invalid output type: {out}"),
                ));
            }
        };

        Ok(query)
    }

    /// Generate a Cypher query for the query
    pub fn generate(&self) -> String {
        if let Some(cypher) = &self.cypher {
            return cypher.clone();
        }

        let mut cypher = if let Some(out) = &self.out {
            ["// @out ", &out.to_string(), "\n"].concat()
        } else {
            String::new()
        };

        if let Some(call) = &self.call {
            cypher.push_str(call);
        } else {
            cypher.push_str("MATCH ");
            cypher.push_str(self.pattern.as_deref().unwrap_or("(node)"));
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
                let alias = alias_for_table(table);
                cypher = cypher.replace(&[&alias, "."].concat(), "node.");
            }
            cypher.push_str("\nRETURN node");
        } else {
            cypher.push_str("\nRETURN ");
            if self.return_distinct.unwrap_or_default() {
                cypher.push_str("DISTINCT ");
            }
            let r#return = self
                .r#return
                .clone()
                .or_else(|| self.node_table_used.clone().map(alias_for_table))
                .unwrap_or("*".to_string());
            cypher.push_str(&r#return);
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
        } else if matches!(self.out, None | Some(QueryResultTransform::Excerpts)) {
            // If no limit is defined and output is excerpts (the default) then
            // apply a limit of 10
            cypher.push_str("\nLIMIT 10");
        }

        for other in &self.union {
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

    /// Execute the query and return the resulting [`Node`]s
    pub fn nodes(&self) -> Vec<Node> {
        let cypher = self.generate();

        let kernel = self.db.clone();
        let cypher_clone = cypher.clone();
        let parameters = self.parameters.clone();
        let (outputs, mut messages) = match task::block_in_place(move || {
            runtime::Handle::current().block_on(async move {
                let kernel = &mut kernel.lock().await;
                for (index, value) in parameters.iter().enumerate() {
                    let name = format!("par{}", index + 1);
                    kernel.set(&name, value).await?;
                }
                kernel.execute(&cypher_clone).await
            })
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

    /// Execute the query and return the combined text representations of all nodes.
    fn text(&self) -> Result<Value, Error> {
        let nodes = self.nodes();
        try_messages(&self.messages)?;
        Ok(Value::from(nodes.iter().map(to_text).join(" ")))
    }

    /// Execute the query and return [`NodeProxies`] for all results
    fn all(&self) -> Value {
        all(self.nodes(), &self.messages)
    }

    /// Execute and return first result as [`NodeProxy`]
    fn first(&self) -> Result<Value, Error> {
        first(self.limit(1).nodes(), &self.messages)
    }

    /// Execute and return last result as [`NodeProxy`]  
    fn last(&self) -> Result<Value, Error> {
        last(self.nodes(), &self.messages)
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
                if first < 0 { 0usize } else { first as usize }
            } else {
                first as usize
            };

            let last = last.unwrap_or(nodes.len() as i32);
            let last = if last < 0 {
                let last = nodes.len() as i32 + last;
                if last < 0 { 0usize } else { last as usize }
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

    /// Add documents to the workspace database
    ///
    /// Accepts either a direct URL string or query results containing documents
    fn add(&self, source: Value) -> Result<(), Error> {
        // Only allow on workspace database
        if self.db_name != "workspace" {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "add() can only be called on workspace database",
            ));
        }

        // Collect document identifiers from the source
        let documents = if let Some(identifier) = source.as_str() {
            vec![vec![identifier.to_string()]]
        } else if let Some(query) = source.downcast_object_ref::<OpenAlexQuery>() {
            // OpenAlex query results - multiple documents with multiple identifiers each
            query
                .nodes()
                .iter()
                .map(|node| extract_identifiers(node))
                .filter(|ids| !ids.is_empty())
                .collect()
        } else if let Some(query) = source.downcast_object_ref::<GitHubQuery>() {
            // GitHub query results
            query
                .nodes()
                .iter()
                .map(|node| extract_identifiers(node))
                .filter(|ids| !ids.is_empty())
                .collect()
        } else {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "add() expects a string or a query",
            ));
        };

        if documents.is_empty() {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "No documents with supported identifiers found in query results",
            ));
        }

        // Execute the add operation
        let result: Result<usize, Error> = task::block_in_place(|| {
            runtime::Handle::current().block_on(async {
                let mut db_kernel = self.db.lock().await;
                match db_kernel.add_documents(&documents).await {
                    Ok(count) => Ok(count),
                    Err(e) => Err(Error::new(ErrorKind::InvalidOperation, e.to_string())),
                }
            })
        });

        match result {
            Ok(added_count) => {
                let message = format!("Added {} documents to workspace", added_count);
                if let Some(mut messages) = lock_messages(&self.messages) {
                    messages.push(ExecutionMessage::new(MessageLevel::Info, message));
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

/// Generate a table name for a method
fn table_for_method(method: &str) -> (String, Option<String>) {
    (
        match method {
            "affiliations" => "Organization".into(),
            "authors" => "Person".into(),
            "audios" => "AudioObject".into(),
            "cells" => "TableCell".into(),
            "chunks" => "CodeChunks".into(),
            "expressions" => "CodeExpression".into(),
            "equations" => "MathBlock".into(),
            "images" => "ImageObject".into(),
            "items" => "ListItem".into(),
            "organizations" => "Organization".into(),
            "people" => "Person".into(),
            "references" => "Reference".into(),
            "rows" => "TableRow".into(),
            "videos" => "VideoObject".into(),
            "abstracts" | "introductions" | "methods" | "results" | "discussions" => {
                let section_type = match method {
                    "methods" => "Methods".into(),
                    "results" => "Results".into(),
                    _ => escape_keyword(&method.to_singular().to_pascal_case()),
                };
                return (
                    "Section".into(),
                    Some(format!("section.sectionType = '{section_type}'")),
                );
            }
            _ => escape_keyword(&method.to_singular().to_pascal_case()),
        },
        None,
    )
}

/// Generate an alias for a table
pub(super) fn alias_for_table<S: AsRef<str>>(table: S) -> String {
    let alias = table.as_ref().to_camel_case();

    match alias.as_str() {
        "audioObject" => "audio".into(),
        "codeChunk" => "chunk".into(),
        "codeExpression" => "expr".into(),
        "imageObject" => "image".into(),
        "listItem" => "item".into(),
        "mathBlock" => "eqn".into(),
        "organization" => "org".into(),
        "reference" => "ref".into(),
        "tableCell" => "cell".into(),
        "tableRow" => "row".into(),
        "videoObject" => "video".into(),
        _ => escape_keyword(&alias),
    }
}

pub(super) const DEFAULT_RELATION: &str = "[:content|:items* acyclic]";

/// Generate the relation between to tables
fn relation_between_tables(table1: &str, table2: &str) -> String {
    match (table1, table2) {
        ("CitationGroup", "Citation") => "[:items]",
        (_, "Citation") => "[:content|:items*]",
        ("Citation", "Reference") => "[:cites]",
        (_, "Reference") => "[:content|:items|:cites*]",
        ("Table", "TableRow") => "[:rows]",
        ("TableRow", "TableCell") => "[:cells]",
        ("Table", "TableCell") => "[:rows]-(row:TableRow)-[:cells]",
        ("Article", "Person") => "[:authors]",
        ("Reference", "Person") => "[:authors]",
        ("Person", "Organization") => "[:affiliations]",
        _ => DEFAULT_RELATION,
    }
    .into()
}

/// Escape an expression if it is a keyword
///
/// Only escaped whole words, not words withing an expression.
/// See https://docs.kuzudb.com/cypher/syntax/#reserved-keywords
fn escape_keyword(word: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "ALL",
        "AND",
        "ASC",
        "ASCENDING",
        "CASE",
        "CAST",
        "COLUMN",
        "CREATE",
        "DBTYPE",
        "DEFAULT",
        "DESC",
        "DESCENDING",
        "DISTINCT",
        "ELSE",
        "END",
        "ENDS",
        "EXISTS",
        "FALSE",
        "FROM",
        "GLOB",
        "GROUP",
        "HEADERS",
        "IN",
        "INSTALL",
        "IS",
        "LIMIT",
        "MACRO",
        "NOT",
        "NULL",
        "ON",
        "ONLY",
        "OPTIONAL",
        "OR",
        "ORDER",
        "PRIMARY",
        "PROFILE",
        "SHORTEST",
        "STARTS",
        "TABLE",
        "THEN",
        "TO",
        "TRUE",
        "UNION",
        "UNWIND",
        "WHEN",
        "WHERE",
        "WITH",
        "XOR",
    ];

    if KEYWORDS.contains(&word.to_uppercase().as_str()) {
        ["`", word, "`"].concat()
    } else {
        word.to_string()
    }
}

/// Translate a filter argument into a Cypher `WHERE` clause
pub(super) fn apply_filter(
    alias: &str,
    arg_name: &str,
    arg_value: Value,
    for_subquery: bool,
) -> Result<String, Error> {
    if arg_name == "_" {
        if let Some(subquery) = arg_value.downcast_object_ref::<Subquery>() {
            return process_subquery_for_cypher(subquery, alias);
        }
    }

    let mut chars = arg_name.chars().collect_vec();

    let last = *chars.last().expect("always has at least one char");
    if last.is_numeric() || last == '_' {
        chars.pop();
    }

    let col = if arg_name.starts_with("_C") {
        if !for_subquery {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "count filters (*) can only be used with subqueries".to_string(),
            ));
        }

        if last.is_numeric() && last > '4' && last != '9' {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "only numeric comparison operators (e.g. <=) can be used in count filters (*)"
                    .to_string(),
            ));
        }

        "_COUNT".to_string()
    } else {
        [alias, ".", &chars.iter().join("").to_camel_case()].concat()
    };

    let val_str = || ["'", &arg_value.to_string(), "'"].concat();

    let val_lit = || {
        if arg_value.as_str().is_some() {
            val_str()
        } else {
            arg_value.to_string()
        }
    };

    Ok(match last {
        '5' => ["regexp_matches(", &col, ", ", &val_str(), ")"].concat(),
        '6' => ["NOT regexp_matches(", &col, ", ", &val_str(), ")"].concat(),
        '7' => ["starts_with(", &col, ", ", &val_str(), ")"].concat(),
        '8' => ["ends_with(", &col, ", ", &val_str(), ")"].concat(),
        '9' => {
            if col == "_COUNT" {
                [&col, " IN ", &val_lit()].concat()
            } else {
                ["list_contains(", &val_lit(), ", ", &col, ")"].concat()
            }
        }
        '_' => ["list_contains(", &col, ", ", &val_lit(), ")"].concat(),
        _ => {
            let op = match last {
                '0' => "<>",
                '1' => "<",
                '2' => "<=",
                '3' => ">",
                '4' => ">=",
                _ => "=",
            };
            [&col, " ", op, " ", &val_lit()].concat()
        }
    })
}

/// Process a subquery for Cypher using name and args
pub(super) fn process_subquery_for_cypher(
    subquery: &Subquery,
    outer_alias: &str,
) -> Result<String, Error> {
    // Get relation and table from subquery name
    let (relation, table) = match subquery.name.as_str() {
        // Note: these should be consistent with subqueries
        // listed in add_subquery_functions

        // Content subqueries
        "code_blocks" => (DEFAULT_RELATION, "CodeBlock"),
        "code_inlines" => (DEFAULT_RELATION, "CodeInline"),
        "code_chunks" | "chunks" => (DEFAULT_RELATION, "CodeChunk"),
        "code_expressions" | "expressions" => (DEFAULT_RELATION, "CodeExpression"),
        "math_blocks" | "equations" => (DEFAULT_RELATION, "MathBlock"),
        "math_inlines" => (DEFAULT_RELATION, "MathInline"),
        "images" => (DEFAULT_RELATION, "ImageObject"),
        "audios" => (DEFAULT_RELATION, "AudioObject"),
        "videos" => (DEFAULT_RELATION, "VideoObject"),
        "admonitions" => (DEFAULT_RELATION, "Admonition"),
        "claims" => (DEFAULT_RELATION, "Claim"),
        "lists" => (DEFAULT_RELATION, "List"),
        "paragraphs" => (DEFAULT_RELATION, "Paragraph"),
        "sections" => (DEFAULT_RELATION, "Section"),
        "sentences" => (DEFAULT_RELATION, "Sentence"),

        // Metadata subqueries
        "authored_by" | "authors" => ("[authors]", "Person"),
        "affiliated_with" | "affiliations" => ("[affiliations]", "Organization"),
        "cites" | "references" => ("[references]", "Reference"),
        "cited_by" => ("[citedBy]", "Reference"),
        "published_in" => ("[publishedIn]", "Periodical"),

        _ => {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported subquery for Cypher: {}", subquery.name),
            ));
        }
    };

    // Generate alias for the subquery table
    let alias = alias_for_table(table);

    // Build the base MATCH pattern
    let mut cypher = format!("MATCH ({outer_alias})-{relation}->({alias}:{table})");

    // Process args to build WHERE conditions
    let mut ands = Vec::new();
    let mut count_condition = None;

    for (arg_name, arg_value) in &subquery.args {
        if !arg_name.is_empty() {
            // This is a keyword argument - process as filter
            let filter = apply_filter(&alias, arg_name, arg_value.clone(), true)?;

            if let Some(rest) = filter.strip_prefix("_COUNT") {
                if count_condition.is_some() {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "only one count filter (*) allowed per call",
                    ));
                }
                count_condition = Some(rest.trim().to_string());
            } else {
                ands.push(filter);
            }
        }
        // Non-keyword arguments (query objects) are handled by the respective modules
    }

    // Add WHERE clause if there are conditions
    if !ands.is_empty() {
        cypher.push_str(" WHERE ");
        cypher.push_str(&ands.join(" AND "));
    }

    // Wrap in COUNT or EXISTS based on whether there's a count condition
    if let Some(count) = count_condition {
        Ok(format!("COUNT {{ {cypher} }} {count}"))
    } else {
        Ok(format!("EXISTS {{ {cypher} }}"))
    }
}

impl Object for CypherQuery {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let no_args = || -> Result<(), Error> {
            if args.is_empty() {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::TooManyArguments,
                    format!("Method `{name}` takes no arguments."),
                ))
            }
        };

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
            "select" => {
                let (args, kwargs): (&[Value], Kwargs) = from_args(args)?;
                self.select(args, kwargs)?
            }
            "count" => {
                no_args()?;
                self.count()?
            }
            "sort" => {
                let (order_by, order) = from_args(args)?;
                self.order_by(order_by, order)?
            }
            "skip" => {
                let (skip,) = from_args(args)?;
                self.skip(skip)
            }
            "limit" => {
                let (count,) = from_args(args)?;
                self.limit(count)
            }
            "sample" => {
                let (count,) = from_args(args)?;
                self.sample(count)
            }
            "union" => {
                let (union, all) = from_args(args)?;
                self.union(union, all)?
            }

            // Specify output type
            "out" => {
                let (out,): (&str,) = from_args(args)?;
                self.out(out)?
            }

            // Return the generated Cypher
            "explain" => {
                no_args()?;
                return Ok(self.explain());
            }

            // Methods that execute the query and return values
            "text" => {
                no_args()?;
                return self.text();
            }

            // Methods that execute the query and return node proxies
            "all" | "first" | "last" => {
                no_args()?;
                return Ok(match name {
                    "all" => self.all(),
                    "first" => self.first()?,
                    "last" => self.last()?,
                    _ => unreachable!(),
                });
            }
            "slice" => {
                let (first, last) = from_args(args)?;
                return self.slice(first, last);
            }

            // Method to add documents to workspace
            "add" => {
                let (source,) = from_args(args)?;
                self.add(source)?;
                return Ok(Value::from(()));
            }

            // Fallback to node adding a MATCH pattern for a node table
            _ => {
                let (args, kwargs) = from_args(args)?;
                self.table(name, args, kwargs)?
            }
        };

        Ok(Value::from_object(query))
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(property) = key.as_str() {
            extend_messages(
                &self.messages,
                format!("Cypher query does not have property `{property}`"),
            );
            None
        } else if let Some(index) = key.as_i64() {
            get(index, self.nodes(), &self.messages)
        } else {
            None
        }
    }
}

/// Query the current document for a type with a label
///
/// Allows for a label filter to be provided without the keyword. e.g.
///
///   figure(1)
///
/// is equivalent to
///
///   document.figures(.label == '1')
///
/// Other filters can be used as well e.g.
///
///   figure(.caption ^= 'Plot of')
#[derive(Debug)]
pub(super) struct CypherQueryLabelled {
    table: String,
    document: Arc<CypherQuery>,
    one: bool,
}

impl CypherQueryLabelled {
    fn new(table: &str, document: Arc<CypherQuery>, one: bool) -> Self {
        Self {
            table: table.into(),
            document,
            one,
        }
    }
}

impl Object for CypherQueryLabelled {
    fn call(self: &Arc<Self>, _state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (args, mut kwargs): (&[Value], Kwargs) = from_args(args)?;

        let mut args = args.to_vec();
        if let Some(first) = args.first() {
            let label = if let Some(label) = first.as_str() {
                label.to_string()
            } else if let Some(num) = first.as_i64() {
                num.to_string()
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!(
                        "argument should be string or integer {} label",
                        &self.table[..(self.table.len() - 1)]
                    ),
                ));
            };

            if !GLOBAL_CONSTS.contains(&label.as_str()) {
                args.remove(0);
                kwargs = kwargs_insert(kwargs, "label", Value::from(label))
            }
        }

        let mut query = self.document.table(&self.table, &args, kwargs.clone())?;

        if self.one {
            query.limit = Some(1);
        }

        if &self.table == "table" {
            query.pattern = Some("(`table`:`Table`:CodeChunk)".into());
            query.ands.push(
                "(starts_with(`table`.nodeId, 'tbl') OR `table`.labelType = 'TableLabel')".into(),
            );
        } else if &self.table == "figure" {
            query.pattern = Some("(figure:Figure:CodeChunk)".into());
            query.ands.push(
                "(starts_with(figure.nodeId, 'fig') OR figure.labelType = 'FigureLabel')".into(),
            );
        }

        Ok(Value::from_object(query))
    }
}

/// Query the current document for a variable
///
/// Allows for a label filter to be provided without the keyword. e.g.
///
///   variable('a')
///
/// is equivalent to
///
///   document.variables(.name == '1')
///
/// Other filters can be used as well e.g.
///
///   variable(.nodeType == 'Integer')
#[derive(Debug)]
pub(super) struct CypherQueryVariables {
    document: Arc<CypherQuery>,
    one: bool,
}

impl CypherQueryVariables {
    fn new(document: Arc<CypherQuery>, one: bool) -> Self {
        Self { document, one }
    }
}

impl Object for CypherQueryVariables {
    fn call(self: &Arc<Self>, _state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (args, mut kwargs): (&[Value], Kwargs) = from_args(args)?;

        let mut args = args.to_vec();
        if let Some(first) = args.first().and_then(|first| first.as_str()) {
            if !GLOBAL_CONSTS.contains(&first) {
                let first = args.remove(0);
                kwargs = kwargs_insert(kwargs, "name", first)
            }
        }

        let query = self.document.table("variables", &args, kwargs)?;
        let query = if self.one { query.limit(1) } else { query };

        Ok(Value::from_object(query))
    }
}

/// Query the current document for a section of a particular type
#[derive(Debug)]
pub(super) struct CypherQuerySectionType {
    section_type: SectionType,
    document: Arc<CypherQuery>,
}

impl CypherQuerySectionType {
    fn new(section_type: SectionType, document: Arc<CypherQuery>) -> Self {
        Self {
            section_type,
            document,
        }
    }
}

impl Object for CypherQuerySectionType {
    fn call(self: &Arc<Self>, _state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let (args, kwargs) = from_args(args)?;
        let kwargs = kwargs_insert(
            kwargs,
            "sectionType",
            Value::from(self.section_type.to_string()),
        );

        let query = self.document.table("sections", args, kwargs)?;

        Ok(Value::from_object(query))
    }
}

/// Query the current document for a node matching filters
#[derive(Debug)]
pub(super) struct CypherQueryNodeType {
    node_type: NodeType,
    document: Arc<CypherQuery>,
    one: bool,
}

impl CypherQueryNodeType {
    fn new(node_type: NodeType, document: Arc<CypherQuery>, one: bool) -> Self {
        Self {
            node_type,
            document,
            one,
        }
    }
}

impl Object for CypherQueryNodeType {
    fn call(self: &Arc<Self>, _state: &State<'_, '_>, args: &[Value]) -> Result<Value, Error> {
        let method = self.node_type.to_string().to_camel_case();
        let (args, kwargs) = from_args(args)?;

        let query = self.document.table(&method, args, kwargs)?;
        let query = if self.one { query.limit(1) } else { query };

        Ok(Value::from_object(query))
    }
}

/// Insert a key/value pair into some minijinja [`Kwargs`]
fn kwargs_insert(kwargs: Kwargs, key: &str, value: Value) -> Kwargs {
    Kwargs::from_iter(
        kwargs
            .args()
            .map(|key| (key, kwargs.get(key).expect("")))
            .chain([(key, value)]),
    )
}

/// Extract identifiers from a single node
///
/// Returns alternative identifiers for the document that can be used to fetch it.
/// Handles both PropertyValue objects and plain string URLs in the identifiers array.
fn extract_identifiers(node: &Node) -> Vec<String> {
    match node {
        Node::Article(article) => {
            let mut identifiers = Vec::new();

            // Try DOI first (most reliable)
            if let Some(doi) = &article.doi {
                identifiers.push(format!("https://doi.org/{}", doi));
            }

            // Then try other identifiers from the identifiers array
            for id in article.options.identifiers.iter().flatten() {
                match id {
                    PropertyValueOrString::String(value)
                    | PropertyValueOrString::PropertyValue(PropertyValue {
                        value: Primitive::String(value),
                        ..
                    }) if value.starts_with("http") => {
                        // Check if this URL is supported by one of our codecs
                        if codecs::codec_for_identifier(value).is_some() {
                            identifiers.push(value.clone());
                        }
                    }
                    PropertyValueOrString::PropertyValue(PropertyValue {
                        property_id: Some(prop_id),
                        value: Primitive::String(value),
                        ..
                    }) => match prop_id.as_str() {
                        "pmc" | "pmcid" => {
                            identifiers.push(format!(
                                "https://www.ncbi.nlm.nih.gov/pmc/articles/{}/",
                                value
                            ));
                        }
                        "arxiv" => {
                            identifiers.push(format!("https://arxiv.org/abs/{}", value));
                        }
                        "biorxiv" => {
                            identifiers.push(format!("https://www.biorxiv.org/content/{}", value));
                        }
                        "medrxiv" => {
                            identifiers.push(format!("https://www.medrxiv.org/content/{}", value));
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            identifiers
        }
        // Future: Add support for other node types like SoftwareApplication
        _ => Vec::new(),
    }
}
