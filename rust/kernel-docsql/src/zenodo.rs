use std::sync::{Arc, Mutex as SyncMutex};

use codec_zenodo::{SearchRecordsResponse, request};
use kernel_jinja::{
    kernel::{
        common::{
            eyre::Result,
            tokio::{runtime, task},
            tracing,
        },
        schema::{CodeChunk, ExecutionMessage, MessageLevel, Node},
    },
    minijinja::{
        Environment, Error, ErrorKind, State, Value,
        value::{Kwargs, Object, ValueKind, from_args},
    },
};

use crate::{
    NodeProxy,
    docsql::{Operator, PropertyType, decode_filter},
    extend_messages,
    nodes::{all, first, get, last},
};

/// Add Zenodo functions to the Jinja environment
pub(crate) fn add_zenodo_functions(
    env: &mut Environment,
    messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>,
) {
    let zenodo = Arc::new(ZenodoQuery::new(messages.clone()));
    env.add_global("zenodo", Value::from_object((*zenodo).clone()));
}

/// Zenodo query builder for generating API calls
///
/// See https://developers.zenodo.org/ for details
#[derive(Debug, Clone)]
pub(crate) struct ZenodoQuery {
    /// Execution messages to be added to when executing the query
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,

    /// The Zenodo object type (e.g. records)
    object_type: String,

    /// Search query parameter
    search: Option<String>,

    /// Filter parameters
    filters: Vec<(String, String)>,

    /// Sort parameter (bestmatch or mostrecent)
    sort: Option<String>,

    /// Number of result items to skip
    skip: Option<usize>,

    /// Number of items to limit result to
    limit: Option<usize>,
}

impl ZenodoQuery {
    /// Create a new Zenodo query
    pub fn new(messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self {
            messages,
            object_type: String::new(),
            search: None,
            filters: Vec::new(),
            sort: None,
            skip: None,
            limit: None,
        }
    }

    /// Create a new Zenodo query for an object type
    pub fn clone_for(&self, object_type: &str) -> Self {
        Self {
            messages: self.messages.clone(),
            object_type: object_type.into(),
            search: None,
            filters: Vec::new(),
            sort: None,
            skip: None,
            limit: None,
        }
    }

    /// Whether this is the base query for which no method has been called yet
    pub fn is_base(&self) -> bool {
        self.object_type.is_empty()
    }

    /// Apply a filter to the query
    fn apply_filter(&mut self, arg_name: &str, arg_value: Value) -> Result<(), Error> {
        // Handle search (for when called for subquery or directly)
        if arg_name == "search"
            && let Some(search) = arg_value.to_str().as_deref()
        {
            self.search = Some(search.into());
            return Ok(());
        }

        // Extract the property name and operator from the arg
        let (property_name, operator) = decode_filter(arg_name);

        // Remove leading dot from property name if present (from .property syntax)
        let property_name = property_name.trim_start_matches('.');

        // Error early for unhandled operators
        if matches!(
            operator,
            Operator::Has | Operator::NoMatch | Operator::Starts | Operator::Ends
        ) {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("The {operator} operator is not supported for Zenodo queries"),
            ));
        }

        // Return an error for an unsupported property
        let unsupported_property = || {
            Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "Unsupported filter property for Zenodo {}: {property_name}",
                    self.object_type
                ),
            ))
        };

        // Map the property name to the Zenodo filter name and determine property type
        let (filter_name, property_type) = match self.object_type.as_str() {
            "records" => match property_name {
                // Title and description map to search query
                "title" | "name" => {
                    if operator == Operator::Match {
                        self.search = Some(format_filter_value(&arg_value)?);
                        return Ok(());
                    }
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "For title filtering, use the ~= operator for search",
                    ));
                }
                "description" => {
                    if operator == Operator::Match {
                        self.search = Some(format_filter_value(&arg_value)?);
                        return Ok(());
                    }
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "For description filtering, use the ~= operator for search",
                    ));
                }
                // Authors/creators
                "creators" | "authors" => {
                    if operator == Operator::Match {
                        self.search = Some(format_filter_value(&arg_value)?);
                        return Ok(());
                    }
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "For creator filtering, use the ~= operator for search",
                    ));
                }
                // Keywords
                "keywords" => {
                    if operator == Operator::Match || operator == Operator::Eq {
                        self.search = Some(format_filter_value(&arg_value)?);
                        return Ok(());
                    }
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "For keyword filtering, use the = or ~= operator",
                    ));
                }
                // DOI
                "doi" => ("doi", PropertyType::String),
                // Enum properties
                "access" | "access_right" => ("access_right", PropertyType::Enum),
                "type" | "resource_type" => ("type", PropertyType::Enum),
                "subtype" => ("subtype", PropertyType::Enum),
                "communities" | "community" => ("communities", PropertyType::Enum),
                "license" => ("license", PropertyType::Enum),
                "status" => ("status", PropertyType::Enum),
                // Date properties with aliases
                "published" | "publication_date" | "date" => {
                    // Zenodo doesn't support date range filters directly in search API
                    // We would need to use search query syntax for this
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "Date filtering is not directly supported in Zenodo search API. Consider using search with date ranges in query syntax.",
                    ));
                }
                // Boolean properties
                "is_open" => {
                    // Convenience filter for open access
                    if arg_value.kind() == ValueKind::Bool {
                        if arg_value.is_true() {
                            self.filters
                                .push(("access_right".to_string(), "open".to_string()));
                        } else {
                            // Not open could be any of: embargoed, restricted, closed
                            return Err(Error::new(
                                ErrorKind::InvalidOperation,
                                "Use access_right filter to specify non-open access types",
                            ));
                        }
                        return Ok(());
                    } else {
                        return Err(Error::new(
                            ErrorKind::InvalidOperation,
                            "The is_open filter requires a boolean value",
                        ));
                    }
                }
                _ => return unsupported_property(),
            },
            _ => return unsupported_property(),
        };

        // Check that operator is valid for property
        if !property_type.is_valid(operator) {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "The {operator} operator cannot be used with the Zenodo `{property_name}` filter"
                ),
            ));
        }

        // Transform the minijinja argument value into a string
        let filter_value = format_filter_value(&arg_value)?;

        // Handle different operators
        match operator {
            Operator::Eq => {
                self.filters.push((filter_name.to_string(), filter_value));
            }
            Operator::Neq => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "Negation is not supported in Zenodo filters",
                ));
            }
            Operator::In => {
                // For 'in' operator with arrays, just use the first value
                // Zenodo doesn't support multiple values in most filters
                if arg_value.kind() == ValueKind::Seq
                    && let Ok(mut iter) = arg_value.try_iter()
                {
                    if let Some(first) = iter.next() {
                        self.filters
                            .push((filter_name.to_string(), format_filter_value(&first)?));
                    }
                } else {
                    self.filters.push((filter_name.to_string(), filter_value));
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Unsupported operator: {operator}"),
                ));
            }
        }

        Ok(())
    }

    /// Set sort parameter
    fn sort(&self, property: &str, direction: Option<String>) -> Result<Self, Error> {
        let mut query = self.clone();

        // Zenodo supports: bestmatch (default) or mostrecent
        // Map common sort properties to Zenodo's sort options and determine default direction
        let (sort_value, default_ascending) = match property {
            "relevance" | "best" | "bestmatch" => ("bestmatch", true),
            "recent" | "mostrecent" => ("mostrecent", false), // mostrecent implies desc by default
            "date" | "published" | "publication_date" => ("mostrecent", true), // date fields default to asc
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "Zenodo only supports sorting by 'bestmatch' or 'mostrecent'",
                ));
            }
        };

        // Handle sort direction with minus prefix
        let sort_with_direction = if let Some(dir) = direction {
            let want_descending = match dir.to_lowercase().as_str() {
                "desc" | "descending" | "d" => true,
                "asc" | "ascending" | "a" => false,
                _ => !default_ascending, // use default if direction is unclear
            };

            if sort_value == "mostrecent" && want_descending {
                "mostrecent".to_string() // mostrecent is naturally descending
            } else if sort_value == "mostrecent" && !want_descending {
                "-mostrecent".to_string() // use minus to make it ascending
            } else {
                // bestmatch doesn't support direction changes
                sort_value.to_string()
            }
        } else {
            // No explicit direction - use property-specific default
            if sort_value == "mostrecent" && default_ascending {
                "-mostrecent".to_string() // make date sorts ascending by default
            } else {
                sort_value.to_string()
            }
        };

        query.sort = Some(sort_with_direction);

        Ok(query)
    }

    /// Set skip count
    fn skip(&self, count: usize) -> Self {
        let mut query = self.clone();
        query.skip = Some(count);
        query
    }

    /// Set limit count
    fn limit(&self, count: usize) -> Self {
        let mut query = self.clone();
        query.limit = Some(count);
        query
    }

    /// Return count of results
    fn count(&self) -> Self {
        let mut query = self.clone();
        // Used in `nodes` to indicate that only count should be extracted
        query.limit = Some(0);
        query
    }

    /// Generate the Zenodo API URL
    pub fn generate(&self) -> String {
        let mut params: Vec<(&str, String)> = Vec::new();

        // Add filters first (for consistent ordering)
        for (name, value) in &self.filters {
            params.push((name.as_str(), value.clone()));
        }

        // Add search query
        if let Some(search) = &self.search {
            params.push(("q", search.clone()));
        }

        // Add sort
        if let Some(sort) = &self.sort {
            params.push(("sort", sort.clone()));
        }

        // Add pagination parameters based on skip and/or limit
        if let (Some(skip), Some(limit)) = (self.skip, self.limit) {
            let page = (skip / limit) + 1;
            params.extend([("size", limit.to_string()), ("page", page.to_string())]);
        } else if let Some(skip) = self.skip {
            // If only skip is provided, use it as page size and go to page 2
            params.extend([("size", skip.to_string()), ("page", "2".to_string())]);
        } else if let Some(limit) = self.limit {
            params.push(("size", limit.to_string()));
        }

        // Build URL manually to avoid unnecessary encoding
        let mut url = "https://zenodo.org/api/records".to_string();

        if !params.is_empty() {
            url.push('?');
            for (i, (name, value)) in params.iter().enumerate() {
                if i > 0 {
                    url.push('&');
                }
                url.push_str(name);
                url.push('=');
                // Only encode spaces, leave other characters as-is for readability
                url.push_str(&value.replace(' ', "+"));
            }
        }

        url
    }

    /// Return the generated URL as an executable explanation
    fn explain(&self) -> Value {
        let url = self.generate();

        let node = Node::CodeChunk(CodeChunk {
            code: ["GET ", &url, "\n"].concat().into(),
            programming_language: Some("http".into()),
            is_echoed: Some(true), // To make visible
            ..Default::default()
        });

        Value::from_object(NodeProxy::new(node, self.messages.clone()))
    }

    /// Execute the query and return the resulting [`Node`]s
    #[tracing::instrument(skip(self))]
    pub fn nodes(&self) -> Vec<Node> {
        let url = self.generate();

        let result: Result<_> = task::block_in_place(|| {
            runtime::Handle::current().block_on(async {
                let response = request::<SearchRecordsResponse>(&url).await?;
                let nodes: Vec<Node> = response.hits.hits.into_iter().map(Node::from).collect();
                Ok((response.hits.total, nodes))
            })
        });

        match result {
            Ok((total_count, nodes)) => {
                if self.limit == Some(0) {
                    return vec![Node::Integer(total_count)];
                }
                nodes
            }
            Err(error) => {
                self.add_error_message(format!("Zenodo API request failed: {error}"));
                Vec::new()
            }
        }
    }

    /// Add an error message to the message list
    fn add_error_message(&self, message: String) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push(ExecutionMessage {
                level: MessageLevel::Error,
                message,
                ..Default::default()
            });
        }
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
}

/// Format a filter value for Zenodo API
fn format_filter_value(value: &Value) -> Result<String, Error> {
    Ok(match value.kind() {
        ValueKind::None | ValueKind::Undefined => "null".to_string(),
        ValueKind::Bool => {
            if value.is_true() {
                "true".into()
            } else {
                "false".into()
            }
        }
        ValueKind::Number => value.to_string(),
        ValueKind::String => value.as_str().unwrap_or_default().into(),
        ValueKind::Bytes => value
            .as_bytes()
            .map(String::from_utf8_lossy)
            .unwrap_or_default()
            .into(),
        kind => {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Invalid filter value kind: {kind}"),
            ));
        }
    })
}

impl Object for ZenodoQuery {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        // Return an error for methods that have args that shouldn't
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

        // Apply method arguments to the query
        let apply_method_args = |query: &mut ZenodoQuery| -> Result<(), Error> {
            let (arg, kwargs): (Option<Value>, Kwargs) = from_args(args)?;
            if let Some(value) = arg
                && let Some(value) = value.as_str()
            {
                query.search = Some(value.into());
            }
            for arg in kwargs.args() {
                let value: Value = kwargs.get(arg)?;
                match arg {
                    "search" => {
                        if let Some(value) = value.as_str() {
                            query.search = Some(value.into());
                        }
                    }
                    "like" => {
                        return Err(Error::new(
                            ErrorKind::UnknownMethod,
                            "Semantic similarity filtering is not available for Zenodo, use `search` instead",
                        ));
                    }
                    _ => query.apply_filter(arg, value)?,
                }
            }
            Ok(())
        };

        let query = match name {
            // Core API URL building methods
            "records" | "datasets" | "publications" | "software" | "posters" | "presentations" => {
                let (object_type, type_filter) = match name {
                    "records" => ("records", None),
                    "datasets" => ("records", Some("dataset")),
                    "publications" => ("records", Some("publication")),
                    "software" => ("records", Some("software")),
                    "posters" => ("records", Some("poster")),
                    "presentations" => ("records", Some("presentation")),
                    _ => unreachable!(),
                };

                if !self.is_base() {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "Zenodo queries do not support chaining",
                    ));
                }

                let mut query = self.clone_for(object_type);

                // Add type filter if specified
                if let Some(type_value) = type_filter {
                    query
                        .filters
                        .push(("type".to_string(), type_value.to_string()));
                }

                // Apply method arguments
                apply_method_args(&mut query)?;

                query
            }
            "sort" => {
                let (property, direction): (String, Option<String>) = from_args(args)?;
                self.sort(&property, direction)?
            }
            "skip" => {
                let (count,): (usize,) = from_args(args)?;
                self.skip(count)
            }
            "limit" => {
                let (count,): (usize,) = from_args(args)?;
                self.limit(count)
            }
            "count" => {
                no_args()?;
                self.count()
            }

            // Return the generated API call
            "explain" => {
                no_args()?;
                return Ok(self.explain());
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

            _ => {
                return Err(Error::new(
                    ErrorKind::UnknownMethod,
                    format!("Unknown method: {name}"),
                ));
            }
        };

        Ok(Value::from_object(query))
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        if let Some(property) = key.as_str() {
            extend_messages(
                &self.messages,
                format!("Zenodo query does not have property `{property}`"),
            );
            None
        } else if let Some(index) = key.as_i64() {
            get(index, self.nodes(), &self.messages)
        } else {
            None
        }
    }
}
