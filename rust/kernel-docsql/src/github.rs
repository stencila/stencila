use std::sync::{Arc, Mutex as SyncMutex};

use codec_github::{
    SearchCodeResponse, SearchRepositoriesResponse, SearchUsersResponse, request, search_url,
};
use kernel_jinja::{
    kernel::{
        common::{
            eyre::{Result, bail},
            itertools::Itertools,
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
    docsql::{Operator, decode_filter},
    extend_messages,
    nodes::{all, first, get, last},
    subquery::Subquery,
};

/// Add GitHub functions to the Jinja environment
pub(crate) fn add_github_functions(
    env: &mut Environment,
    messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>,
) {
    let github = Arc::new(GitHubQuery::new(messages.clone()));
    env.add_global("github", Value::from_object((*github).clone()));
}

/// GitHub query builder for generating API calls
///
/// See https://docs.github.com/en/search-github/github-code-search/understanding-github-code-search-syntax
#[derive(Debug, Clone)]
pub(crate) struct GitHubQuery {
    /// Execution messages to be added to when executing the query
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,

    /// The GitHub object type (e.g. code, repositories)
    object_type: String,

    /// Filter parameters
    filters: Vec<String>,

    /// Search term
    search: Option<String>,

    /// Sort parameter (e.g., "stars")
    sort: Option<String>,

    /// Sort order ('desc' or 'asc')
    order: Option<String>,

    /// Number of result items to skip
    skip: Option<usize>,

    /// Number of items to limit result to
    limit: Option<usize>,
}

impl GitHubQuery {
    /// Create a new GitHub query
    pub fn new(messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self {
            messages,
            object_type: String::new(),
            filters: Vec::new(),
            search: None,
            sort: None,
            order: None,
            skip: None,
            limit: None,
        }
    }

    /// Whether this is the base query for which no method has been called yet
    pub fn is_base(&self) -> bool {
        self.object_type.is_empty()
    }

    /// Set the type for the query (code, repositories, users, etc.)
    fn object_type(&self, object_type: &str) -> Self {
        let mut query = self.clone();
        query.object_type = object_type.into();
        query
    }

    /// Add a filter to the query
    fn filter(&mut self, arg_name: &str, arg_value: Value) -> Result<(), Error> {
        // Handle subquery filters (e.g., ...authors(.name ~= "Smith"))
        if arg_name == "_" {
            if let Some(_subquery) = arg_value.downcast_object_ref::<Subquery>() {
                //return self.subquery_filters(subquery);
            }
        }

        // Handle search (for when called for subquery)
        if arg_name == "search"
            && let Some(search) = arg_value.to_str().as_deref()
        {
            self.search = Some(search.into());
            return Ok(());
        }

        // Extract the property name an operator from the arg
        let (property_name, operator) = decode_filter(arg_name);

        // Error early for unhandled operators
        if operator == Operator::Has {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("The `{operator}` operator is not supported for GitHub queries"),
            ));
        }

        let unsupported_property = || {
            Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "Unsupported filter property for GitHub {}: {property_name}",
                    self.object_type
                ),
            ))
        };

        // Map the property name to the GitHub qualifier name
        let (qualifier_name, is_boolean) = match self.object_type.as_str() {
            "code" => match property_name {
                // See https://docs.github.com/en/search-github/searching-on-github/searching-code
                "user" | "org" | "repo" | "path" | "language" | "size" | "filename"
                | "extension" => (property_name.to_string(), false),
                _ => return unsupported_property(),
            },
            "users" => match property_name {
                // See https://docs.github.com/en/search-github/searching-on-github/searching-users
                "type" | "user" | "org" | "fullname" | "repos" | "location" | "language"
                | "created" | "followers" => (property_name.to_string(), false),
                _ => return unsupported_property(),
            },
            "repositories" => match property_name {
                // See https://docs.github.com/en/search-github/searching-on-github/searching-for-repositories
                "repo" | "user" | "org" | "size" | "followers" | "forks" | "stars" | "created"
                | "pushed" | "language" | "topic" | "license" => (property_name.to_string(), false),
                // Boolean properties
                "is_public" | "is_private" | "is_mirror" | "is_template" | "is_archived" => {
                    (property_name.replace("_", ":"), true)
                }
                _ => return unsupported_property(),
            },
            // Error for all others
            _ => return unsupported_property(),
        };

        // Handle boolean operators
        if is_boolean {
            if arg_value.kind() == ValueKind::Bool {
                let filter = if arg_value.is_true() {
                    qualifier_name
                } else {
                    format!("NOT {qualifier_name}")
                };
                self.filters.push(filter);
                return Ok(());
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("The `{property_name}` filter can only be used with boolean values"),
                ));
            }
        }

        // Handle the `in` operator by expanding it into qualifiers joined by OR
        if operator == Operator::In {
            if arg_value.kind() == ValueKind::Seq
                && let Ok(iter) = arg_value.try_iter()
            {
                let joined = iter
                    .filter_map(|value| format_filter_value(&value).ok())
                    .map(|value| format!("{property_name}:{value}"))
                    .join(" OR ");
                self.filters.push(joined);
                return Ok(());
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "The `in` operator can only be used with sequence values",
                ));
            }
        }

        // Transform the minijinja argument value into a string
        let qualifier_value = format_filter_value(&arg_value)?;

        // Generate the filter string
        let filter_string = match operator {
            Operator::Eq => format!("{qualifier_name}:{qualifier_value}"),
            Operator::Neq => format!("NOT {qualifier_name}:{qualifier_value}"),
            Operator::Lt => format!("{qualifier_name}:<{qualifier_value}"),
            Operator::Lte => format!("{qualifier_name}:<={qualifier_value}"),
            Operator::Gt => format!("{qualifier_name}:>{qualifier_value}"),
            Operator::Gte => format!("{qualifier_name}:>={qualifier_value}"),

            // Regex-based filters for code search
            // See https://docs.github.com/en/search-github/github-code-search/understanding-github-code-search-syntax#using-regular-expressions
            Operator::Match => format!("{qualifier_name}:/{qualifier_value}/"),
            Operator::NoMatch => format!("NOT {qualifier_name}:/{qualifier_value}/"),
            Operator::Starts => format!("{qualifier_name}:/^{qualifier_value}/"),
            Operator::Ends => format!("{qualifier_name}:/{qualifier_value}$/"),

            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Unsupported operator: {operator}"),
                ));
            }
        };
        self.filters.push(filter_string);

        Ok(())
    }

    /// Set the search term
    fn search(&self, term: &str) -> Self {
        let mut query = self.clone();
        query.search = Some(term.into());
        query
    }

    /// Set sort parameter
    fn sort(&self, property: &str, direction: Option<String>) -> Result<Self, Error> {
        let mut query = self.clone();

        query.sort = Some(property.to_string());

        if let Some(direction) = direction {
            let order = match direction.to_lowercase().as_str() {
                "a" | "asc" | "ascending" => "asc",
                "d" | "des" | "dec" | "desc" | "descending" => "desc",
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "Sort order should be `asc` or `desc`",
                    ));
                }
            };
            query.order = Some(order.into());
        }

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

    /// Generate the GitHub API URL
    pub fn generate(&self) -> String {
        let mut query = self.search.clone().unwrap_or_default();

        for filter in &self.filters {
            if !query.is_empty() {
                query.push(' ');
            }
            query.push_str(filter);
        }

        if query.is_empty() {
            // There has to be something in the q parameter
            query.push('.');
        }

        let mut params = vec![("q", query)];

        // Add sort
        if let Some(sort) = &self.sort {
            params.push(("sort", sort.to_string()));
        }

        // Add pagination parameters based on skip and/or limit
        if let (Some(skip), Some(limit)) = (self.skip, self.limit) {
            let page = (skip / limit) + 1;
            params.extend([("per_page", limit.to_string()), ("page", page.to_string())]);
        } else if let Some(skip) = self.skip {
            params.extend([("per_page", skip.to_string()), ("page", "2".to_string())]);
        } else if let Some(limit) = self.limit {
            params.push(("per_page", limit.to_string()));
        }

        search_url(&self.object_type, &params)
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
        let object_type = self.object_type.as_str();

        let result: Result<_> = task::block_in_place(|| {
            runtime::Handle::current().block_on(async {
                Ok(match object_type {
                    "code" => {
                        let response = request::<SearchCodeResponse>(&url).await?;
                        let nodes = response.items.into_iter().map(Node::from).collect();
                        (response.total_count, nodes)
                    }
                    "users" => {
                        let response = request::<SearchUsersResponse>(&url).await?;
                        let nodes = response.items.into_iter().map(Node::from).collect();
                        (response.total_count, nodes)
                    }
                    "repositories" => {
                        let response = request::<SearchRepositoriesResponse>(&url).await?;
                        let nodes = response.items.into_iter().map(Node::from).collect();
                        (response.total_count, nodes)
                    }
                    _ => {
                        bail!("Fetching of GitHub `{object_type}` objects not yet enabled")
                    }
                })
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
                self.add_error_message(format!("GitHub API request failed: {error}"));
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

/// Format a filter value for GitHub API
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

impl Object for GitHubQuery {
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
            // Core API URL building methods
            "code" | "users" | "repositories" => {
                let object_type = name;

                let mut query = self.object_type(object_type);

                // Handle `search` and `like` arguments and apply all others as filters
                let (arg, kwargs): (Option<Value>, Kwargs) = from_args(args)?;
                if let Some(value) = arg {
                    if let Some(value) = value.as_str() {
                        query = query.search(value)
                    }
                }
                for arg in kwargs.args() {
                    let value: Value = kwargs.get(arg)?;
                    match arg {
                        "search" => {
                            if let Some(value) = value.as_str() {
                                query = query.search(value)
                            }
                        }
                        "like" => {
                            return Err(Error::new(
                                ErrorKind::UnknownMethod,
                                "Semantic similarity filtering is not available for GitHub, use `search` instead",
                            ));
                        }
                        _ => query.filter(arg, value)?,
                    }
                }

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
                format!("GitHub query does not have property `{property}`"),
            );
            None
        } else if let Some(index) = key.as_i64() {
            get(index, self.nodes(), &self.messages)
        } else {
            None
        }
    }
}
