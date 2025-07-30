use std::{ops::Deref, sync::Arc};

use kernel_jinja::{
    kernel::{common::eyre::Result, schema::NodeType},
    minijinja::{
        Environment, Error, ErrorKind, State, Value,
        value::{Kwargs, Object, from_args},
    },
};

use crate::{
    CypherQuery,
    cypher::{
        DEFAULT_RELATION, alias_for_table, apply_filter, relation_between_tables, table_for_method,
    },
    docsql::decode_filter,
    openalex::OpenAlexQuery,
};

/// Add functions for subqueries
///
/// These functions are all prefixed with an underscore because they are not intended
/// to be used directly by users but are rather invocated via the ... syntax for
/// subquery filters.
///
/// Note: leading underscore intentional and important         
pub(super) fn add_subquery_functions(env: &mut Environment) {
    for (name, relation, table) in [
        ("authors", "[authors]", NodeType::Person),
        ("references", "[references]", NodeType::Reference),
        ("cites", "[references]", NodeType::Reference),
        ("citedBy", "[citedBy]", NodeType::Reference),
        ("publishedIn", "[publishedIn]", NodeType::Periodical),
        ("affiliations", "[affiliations]", NodeType::Organization),
        ("organizations", "[organizations]", NodeType::Organization),
        // GitHub-specific subqueries
        ("topics", "[topics]", NodeType::String), // GitHub topics are strings
        ("owners", "[owners]", NodeType::Person),
    ] {
        env.add_global(
            ["_", name].concat(),
            Value::from_object(Subquery::new(relation, table)),
        );
    }

    for (name, table) in [
        // Static code
        ("codeBlocks", NodeType::CodeBlock),
        ("codeInlines", NodeType::CodeInline),
        // Executable code
        ("codeChunks", NodeType::CodeChunk),
        ("chunks", NodeType::CodeChunk),
        ("codeExpressions", NodeType::CodeExpression),
        ("expressions", NodeType::CodeExpression),
        // Math
        ("mathBlocks", NodeType::MathBlock),
        ("mathInlines", NodeType::MathInline),
        // Media
        ("images", NodeType::ImageObject),
        ("audios", NodeType::AudioObject),
        ("videos", NodeType::VideoObject),
        // Containers
        ("admonitions", NodeType::Admonition),
        ("claims", NodeType::Claim),
        ("lists", NodeType::List),
        ("paragraphs", NodeType::Paragraph),
        ("sections", NodeType::Section),
        ("sentences", NodeType::Sentence),
        // Metadata
        ("organizations", NodeType::Organization),
        ("people", NodeType::Person),
    ] {
        env.add_global(
            ["_", name].concat(),
            Value::from_object(Subquery::new(DEFAULT_RELATION, table)),
        );
    }
}

/// A subquery filter
#[derive(Debug, Clone)]
pub(crate) struct Subquery {
    /// The `MATCH` pattern for the subquery
    ///
    /// The front of the pattern can not be determined until the `generate`
    /// method is called.
    pub(crate) pattern: String,

    /// The initial relation involved in the subquery
    pub(crate) first_relation: String,

    /// The initial table involved in the subquery
    pub(crate) first_table: String,

    /// The last table involved in the subquery
    ///
    /// Used to determine the relation at the back of the `pattern`.
    pub(crate) last_table: String,

    /// Filters applied in the subquery (Cypher format for KuzuDB)
    pub(crate) ands: Vec<String>,

    /// Whether this is a `COUNT` subquery, and if so the conditional clause associated with it.
    ///
    /// See https://docs.kuzudb.com/cypher/subquery/#count
    pub(crate) count: Option<String>,

    /// Raw filter information for external APIs like OpenAlex
    ///
    /// Stores the original property name, operator, and value before conversion to Cypher
    pub(crate) raw_filters: Vec<(String, String, Value)>,

    /// Query objects passed to subqueries for ID-based filtering
    ///
    /// Stores query objects (OpenAlex queries, workspace queries) that should be executed
    /// to extract IDs for external API filters like OpenAlex's cited_by
    pub(crate) query_objects: Vec<Value>,
}

impl Subquery {
    /// Create a new subquery
    fn new(relation: &str, node_type: NodeType) -> Self {
        let table = node_type.to_string();
        Self {
            pattern: String::new(),
            first_relation: relation.into(),
            first_table: table.clone(),
            last_table: table,
            ands: Vec::new(),
            count: None,
            raw_filters: Vec::new(),
            query_objects: Vec::new(),
        }
    }

    /// Generate cypher for the subquery
    pub fn generate(&self, alias: &str) -> String {
        let mut cypher = format!(
            "MATCH ({alias})-{}->({}:{}){}",
            self.first_relation,
            alias_for_table(&self.first_table),
            self.first_table,
            self.pattern
        );

        if !self.ands.is_empty() {
            cypher.push_str(" WHERE ");
            cypher.push_str(&self.ands.join(" AND "));
        }

        if let Some(count) = &self.count {
            format!("COUNT {{ {cypher} }} {count}")
        } else {
            format!("EXISTS {{ {cypher} }}")
        }
    }
}

impl Object for Subquery {
    fn call(self: &Arc<Self>, _state: &State, args: &[Value]) -> Result<Value, Error> {
        let mut subquery = self.deref().clone();

        let table = &self.first_table;

        // TODO: alias needs to be different from alias used in outer
        let alias = alias_for_table(table);

        let (query, kwargs): (Option<Value>, Kwargs) = from_args(args)?;

        if let Some(query) = query {
            // Check if this is a query object that should be stored for ID extraction
            if query.downcast_object_ref::<CypherQuery>().is_some()
                || query.downcast_object_ref::<OpenAlexQuery>().is_some()
            {
                // Store query object for later ID extraction
                subquery.query_objects.push(query.clone());
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!(
                        "non-keyword arguments must be another query, got a {}",
                        query.kind()
                    ),
                ));
            }
        }

        for arg_name in kwargs.args() {
            let (property, operator) = decode_filter(arg_name);
            let arg_value: Value = kwargs.get(arg_name)?;

            let filter = apply_filter(&alias, arg_name, arg_value.clone(), true)?;

            if let Some(rest) = filter.strip_prefix("_COUNT") {
                if subquery.count.is_some() {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "only one count filter (*) allowed per call",
                    ));
                }

                subquery.count = Some(rest.trim().to_string());
            } else {
                subquery.ands.push(filter);
                // Store original filter information for external APIs
                subquery
                    .raw_filters
                    .push((property.to_string(), operator.to_string(), arg_value));
            }
        }

        Ok(Value::from_object(subquery))
    }

    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let mut subquery = self.deref().clone();

        let (table, and) = table_for_method(name);
        if let Some(and) = and {
            subquery.ands.push(and);
        }

        let alias = alias_for_table(&table);
        let relation = relation_between_tables(&self.last_table, &table);

        subquery
            .pattern
            .push_str(&format!("-{relation}->({alias}:{table})"));

        let (kwargs,): (Kwargs,) = from_args(args)?;
        for arg_name in kwargs.args() {
            let arg_value = kwargs.get(arg_name)?;
            let filter = apply_filter(&alias, arg_name, arg_value, true)?;
            subquery.ands.push(filter);
        }

        subquery.last_table = table;

        Ok(Value::from_object(subquery))
    }
}
