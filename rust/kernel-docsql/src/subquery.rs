use std::{ops::Deref, sync::Arc};

use kernel_jinja::{
    kernel::common::{eyre::Result, inflector::Inflector},
    minijinja::{
        Environment, Error, State, Value,
        value::{Kwargs, Object, from_args},
    },
};

/// Add functions for subqueries
///
/// These functions are all prefixed with an underscore because they are not intended
/// to be used directly by users but are rather invocated via the ... syntax for
/// subquery filters.
///
/// Note: leading underscore intentional and important         
#[rustfmt::skip]
pub(super) fn add_subquery_functions(env: &mut Environment) {
    // Names on the same row are usually, but not necessarily, used as aliases
    // Exact behavior will depend upon the context eg. CypherQuery vs OpenAlexQuery
    for name in [
        // Content
        // Static code
        "code_blocks",
        "code_inlines",
        // Executable code
        "code_chunks", "chunks",
        "code_expressions", "expressions",
        // Math
        "math_blocks", "equations",
        "math_inlines",
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
        "authors", "people",
        "affiliations", "organizations",
        "references", "cites",
        "cited_by",
        "published_in",
        "published_by",
        "funded_by",
        "part_of",
        "owned_by",
    ] {
        env.add_global(
            ["_", name].concat(),
            Value::from_object(Subquery::new(name)),
        );

        let camel_case = name.to_camel_case();
        if camel_case != name {
            env.add_global(
                ["_", &camel_case].concat(),
                Value::from_object(Subquery::new(name)),
            );
        }
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

        let (arg, kwargs): (Option<Value>, Kwargs) = from_args(args)?;

        if let Some(arg) = arg {
            subquery.args.push(("search".to_string(), arg));
        }

        for name in kwargs.args() {
            subquery.args.push((name.to_string(), kwargs.get(name)?));
        }

        Ok(Value::from_object(subquery))
    }
}
