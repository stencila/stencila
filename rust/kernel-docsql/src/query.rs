use std::sync::Arc;

use kernel_docs::DocsKernelInstance;
use kernel_jinja::{
    kernel::{
        common::{
            eyre::Result, inflector::Inflector, itertools::Itertools, once_cell::sync::Lazy,
            regex::Regex, tracing,
        },
        schema::{ExecutionMessage, MessageLevel, Node},
        KernelInstance,
    },
    minijinja::{
        value::{from_args, DynObject, Kwargs, Object},
        Environment, Error, ErrorKind, State, Value,
    },
};

#[derive(Debug, Default, Clone)]
pub(super) struct Query {
    /// The database to query
    pub db: String,

    /// The Cypher for the query
    cypher: Option<String>,

    /// The pattern for the query
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
    pub fn new(db: &str) -> Query {
        Self {
            db: db.to_string(),
            ..Default::default()
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
                "and" | "or" => {
                    // Non-property arguments: ensure string
                    let condition = value
                        .as_str()
                        .ok_or_else(|| {
                            Error::new(
                                ErrorKind::InvalidOperation,
                                format!("argument `{arg}` is should be a string"),
                            )
                        })?
                        .to_string();

                    match arg {
                        "and" => query.ands.push(condition),
                        "or" => query.ors.push(condition),
                        _ => unreachable!(),
                    }
                }
                _ => {
                    // Property argument: ensure camel cased
                    let property = arg.to_camel_case();

                    let cypher = if let Some(op) = value.downcast_object_ref::<Operator>() {
                        op.generate(&alias, &property)
                    } else if let Some(call) = value.downcast_object_ref::<Call>() {
                        call.generate(&alias, &property)
                    } else {
                        let value = if let Some(string) = value.as_str() {
                            ["\"", &string.to_string(), "\""].concat()
                        } else {
                            value.to_string()
                        };
                        [&alias, ".", arg, " = ", &value].concat()
                    };
                    query.ands.push(cypher)
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
    /// The default is `RETURN DISTINCT *` or `RETURN DISTINCT <table>` where <table>
    /// was the last table used in the method chain. This makes sense for most queries
    /// but this method allows the user to override that if desired.
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

        let mut cypher = String::from("MATCH ");

        let pattern = self.pattern.as_deref().unwrap_or("(node)");
        cypher.push_str(pattern);

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

        cypher.push_str("\nRETURN ");
        if self.return_distinct.unwrap_or(true) {
            cypher.push_str("DISTINCT ");
        }
        let r#return = self.r#return.as_deref().unwrap_or("*");
        cypher.push_str(r#return);

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

    /// Execute the query in a kernel and optionally prepend results with a query
    ///
    /// The intention for explanations is to provide LLMs with the generated Cypher
    /// to act as few shot examples to generate their own Cypher queries for document
    /// context databases.
    #[tracing::instrument(skip_all)]
    pub async fn execute(
        &self,
        kernel: &mut DocsKernelInstance,
    ) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        let cypher = self.generate();
        tracing::trace!("Generated cypher: {cypher}");

        let (outputs, mut messages) = kernel.execute(&cypher).await?;

        // If any messages and add generated Cypher in a message
        if !messages.is_empty() {
            messages.push(ExecutionMessage {
                level: MessageLevel::Debug,
                message: format!("Generated Cypher:\n{cypher}"),
                ..Default::default()
            });
        }

        Ok((outputs, Vec::new()))
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
            _ => {
                let (kwargs,) = from_args(args)?;
                self.table(name, kwargs)?
            }
        };
        Ok(Value::from_object(query))
    }
}

#[derive(Debug)]
struct Operator {
    op: String,
    arg: Value,
}

impl Operator {
    pub fn make(op: &str, arg: Value) -> Result<Value, Error> {
        Ok(Value::from_object(Self {
            op: op.into(),
            arg: arg.into(),
        }))
    }

    /// Generate Cypher for the operator
    fn generate(&self, alias: &str, property: &str) -> String {
        let mut cypher = [alias, ".", property, " ", &self.op, " "].concat();
        if let Some(str) = self.arg.as_str() {
            cypher.push('"');
            cypher.push_str(str);
            cypher.push('"');
        } else {
            cypher.push_str(&self.arg.to_string());
        }
        cypher
    }
}

impl Object for Operator {}

#[derive(Debug)]
struct Call {
    name: String,
    args: Vec<Value>,
}

impl Call {
    pub fn make(name: &str, args: &[Value]) -> Result<Value, Error> {
        Ok(Value::from_object(Self {
            name: name.into(),
            args: args.into(),
        }))
    }

    /// Generate Cypher for the function call
    fn generate(&self, alias: &str, property: &str) -> String {
        let mut cypher = [&self.name, "(", alias, ".", property].concat();
        for arg in &self.args {
            cypher.push(',');
            if let Some(str) = arg.as_str() {
                cypher.push('"');
                cypher.push_str(str);
                cypher.push('"');
            } else {
                cypher.push_str(&arg.to_string());
            }
        }
        cypher.push_str(")");
        cypher
    }
}

impl Object for Call {}

pub fn add_to_env(env: &mut Environment) {
    env.add_global("workspace", Value::from_object(Query::new("workspace")));

    // Operators
    env.add_function("eq", |value: Value| Operator::make("=", value));
    env.add_function("neq", |value: Value| Operator::make("<>", value));
    env.add_function("lt", |value: Value| Operator::make("<", value));
    env.add_function("lte", |value: Value| Operator::make("<=", value));
    env.add_function("gt", |value: Value| Operator::make(">", value));
    env.add_function("gte", |value: Value| Operator::make(">=", value));
    env.add_function("in", |value: Value| Operator::make("IN", value));

    // String functions
    env.add_function("contains", |s: &str| {
        Call::make("contains", &[Value::from(s)])
    });

    fn starts_with(s: &str) -> Result<Value, Error> {
        Call::make("starts_with", &[Value::from(s)])
    }
    env.add_function("starts_with", starts_with);
    env.add_function("startsWith", starts_with);

    fn ends_with(s: &str) -> Result<Value, Error> {
        Call::make("ends_with", &[Value::from(s)])
    }
    env.add_function("ends_with", ends_with);
    env.add_function("endsWith", ends_with);

    env.add_function("matches", |s: &str| {
        Call::make("regexp_matches", &[Value::from(s)])
    });
}
