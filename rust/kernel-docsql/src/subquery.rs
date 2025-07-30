use std::{ops::Deref, sync::Arc};

use kernel_jinja::{
    kernel::{common::eyre::Result, schema::NodeType},
    minijinja::{
        Environment, Error, ErrorKind, State, Value,
        value::{Kwargs, Object, from_args},
    },
};

use crate::cypher::{DEFAULT_RELATION, alias_for_table, apply_filter};

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
            Value::from_object(Subquery::new(name, relation, table)),
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
            Value::from_object(Subquery::new(name, DEFAULT_RELATION, table)),
        );
    }
}

/// A subquery filter
#[derive(Debug, Clone)]
pub(crate) struct Subquery {
    /// Name of the subquery
    pub(crate) name: String,

    /// Arguments to the subquery
    ///
    /// Stores the raw argument names and values so that `CypherQuery`,
    /// `OpenAlexQuery` etc can use those appropriately for their respective
    /// target APIs.
    ///
    /// The arg name (first value in tuple) will be empty if the argument was
    /// not a keyword argument.
    pub(crate) args: Vec<(String, Value)>,

    // TODO: the following are used in the cypher, openalex and github modules. Those
    // should all the refactored to use the new `name` field above
    /// The `MATCH` pattern for the subquery
    ///
    /// The front of the pattern can not be determined until the `generate`
    /// method is called.
    pub(crate) pattern: String,

    /// The initial relation involved in the subquery
    pub(crate) first_relation: String,

    /// The initial table involved in the subquery
    pub(crate) first_table: String,

    // TODO: The following are used in the cypher module only, it should instead use the new
    // `args` field above.
    /// Filters applied in the subquery (Cypher format for KuzuDB)
    pub(crate) ands: Vec<String>,

    /// Whether this is a `COUNT` subquery, and if so the conditional clause associated with it.
    ///
    /// See https://docs.kuzudb.com/cypher/subquery/#count
    pub(crate) count: Option<String>,
}

impl Subquery {
    /// Create a new subquery
    fn new(name: &str, relation: &str, node_type: NodeType) -> Self {
        let table = node_type.to_string();
        Self {
            name: name.into(),
            args: Vec::new(),
            // TODO: refactor away these
            pattern: String::new(),
            first_relation: relation.into(),
            first_table: table.clone(),
            ands: Vec::new(),
            count: None,
        }
    }

    /// Generate cypher for the subquery
    pub fn generate(&self, alias: &str) -> String {
        // TODO: Move this to a new Cypher::apply_subquery (or similar) method
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

        // Capture all arguments
        for arg in args {
            if arg.is_kwargs()
                && let Some(kwargs) = arg.as_object()
            {
                for (name, value) in kwargs.try_iter_pairs().into_iter().flatten() {
                    let name = name.as_str().unwrap_or_default().to_string();
                    subquery.args.push((name, value));
                }
            } else {
                subquery.args.push((String::new(), arg.clone()));
            }
        }

        // TODO: Move the rest of this function to a new Cypher::apply_subquery (or similar) method

        let table = &self.first_table;

        // TODO: alias needs to be different from alias used in outer
        let alias = alias_for_table(table);

        let (query, kwargs): (Option<Value>, Kwargs) = from_args(args)?;

        for arg_name in kwargs.args() {
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
            }
        }

        Ok(Value::from_object(subquery))
    }
}
