use std::{ops::Deref, sync::Arc};

use kernel_jinja::{
    kernel::common::eyre::Result,
    minijinja::{Environment, Error, State, Value, value::Object},
};

/// Add functions for subqueries
///
/// These functions are all prefixed with an underscore because they are not intended
/// to be used directly by users but are rather invocated via the ... syntax for
/// subquery filters.
///
/// Note: leading underscore intentional and important         
pub(super) fn add_subquery_functions(env: &mut Environment) {
    for name in [
        "authors",
        "references",
        "cites",
        "citedBy",
        "publishedIn",
        "affiliations",
        "organizations",
        // GitHub-specific subqueries
        "topics", // GitHub topics are strings
        "owners",
    ] {
        env.add_global(
            ["_", name].concat(),
            Value::from_object(Subquery::new(name)),
        );
    }

    for name in [
        // Static code
        "codeBlocks",
        "codeInlines",
        // Executable code
        "codeChunks",
        "chunks",
        "codeExpressions",
        "expressions",
        // Math
        "mathBlocks",
        "mathInlines",
        // Media
        "images",
        "audios",
        "videos",
        // Containers
        "admonitions",
        "claims",
        "lists",
        "paragraphs",
        "sections",
        "sentences",
        // Metadata
        "organizations",
        "people",
    ] {
        env.add_global(
            ["_", name].concat(),
            Value::from_object(Subquery::new(name)),
        );
    }
}

/// A subquery filter
///
/// This simply captures the arguments it was called with so that `CypherQuery`,
/// `OpenAlexQuery` etc can use those appropriately for their respective target
/// APIs.
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
}

impl Subquery {
    fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            args: Vec::new(),
        }
    }
}

impl Object for Subquery {
    fn call(self: &Arc<Self>, _state: &State, args: &[Value]) -> Result<Value, Error> {
        let mut subquery = self.deref().clone();

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

        Ok(Value::from_object(subquery))
    }
}
