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
    docsql::{Operator, PropertyType, decode_filter},
    extend_messages,
    nodes::{all, first, get, last},
    subquery::Subquery,
    testing,
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

    /// Create a new GitHub query for an object type
    pub fn clone_for(&self, object_type: &str) -> Self {
        Self {
            messages: self.messages.clone(),
            object_type: object_type.into(),
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

    /// Apply a filter to the query
    fn apply_filter(&mut self, arg_name: &str, arg_value: Value) -> Result<(), Error> {
        // Handle subquery filters (e.g., ...authors(.name ~= "Smith"))
        if arg_name == "_" {
            if let Some(subquery) = arg_value.downcast_object_ref::<Subquery>() {
                return self.apply_subquery_filters(subquery);
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
                format!("The {operator} operator is not supported for GitHub queries"),
            ));
        }

        // Handle a filter property implemented via an `in:` qualifier
        let mut in_property = || match operator {
            Operator::Match => {
                if let Some(needle) = arg_value.as_str() {
                    self.filters.push(format!("{needle} in:{property_name}"));
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("The {operator} operator only supports string values"),
                    ))
                }
            }
            _ => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "The GitHub `{property_name}` filter does not support the {operator} operator"
                ),
            )),
        };

        // Return an error for an unsupported property
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
        let (qualifier_name, property_type) = match self.object_type.as_str() {
            "code" => match property_name {
                // See https://docs.github.com/en/search-github/searching-on-github/searching-code
                "path" | "filename" => (property_name.to_string(), PropertyType::String),
                "user" | "org" | "repo" | "language" | "extension" => {
                    (property_name.to_string(), PropertyType::Enum)
                }
                "size" => (property_name.to_string(), PropertyType::Number),
                _ => return unsupported_property(),
            },
            "users" => match property_name {
                // See https://docs.github.com/en/search-github/searching-on-github/searching-users
                "name" | "login" | "email" => return in_property(),
                "fullname" | "location" => (property_name.to_string(), PropertyType::String),
                "type" | "user" | "org" | "language" => {
                    (property_name.to_string(), PropertyType::Enum)
                }
                "repos" | "followers" => (property_name.to_string(), PropertyType::Number),
                "created" => (property_name.to_string(), PropertyType::Date),
                _ => return unsupported_property(),
            },
            "repositories" => match property_name {
                // See https://docs.github.com/en/search-github/searching-on-github/searching-for-repositories
                "name" | "description" | "readme" => return in_property(),
                "repo" | "user" | "org" | "license" | "language" | "topic" => {
                    (property_name.to_string(), PropertyType::Enum)
                }
                "size" | "followers" | "forks" | "stars" => {
                    (property_name.to_string(), PropertyType::Number)
                }
                "created" | "pushed" => (property_name.to_string(), PropertyType::Date),
                "is_public" | "is_private" | "is_mirror" | "is_template" | "is_archived" => {
                    (property_name.replace("_", ":"), PropertyType::Boolean)
                }
                _ => return unsupported_property(),
            },
            // Error for all others
            _ => return unsupported_property(),
        };

        // Check that operator is valid for property
        if !property_type.is_valid(operator) {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "The {operator} operator can not be used with the GitHub `{property_name}` filter"
                ),
            ));
        }

        // Handle boolean operators
        if property_type == PropertyType::Boolean {
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

    /// Apply filters specified in a subquery
    fn apply_subquery_filters(&mut self, subquery: &Subquery) -> Result<(), Error> {
        let object_type = self.object_type.clone();
        let subquery_name = subquery.name.as_str();

        match (object_type.as_str(), subquery_name) {
            ("code", "part_of") => {
                let mut query = self.clone_for("repositories");
                for (arg_name, arg_value) in &subquery.args {
                    query.apply_filter(arg_name, arg_value.clone())?;
                }

                self.apply_ids_filter(
                    Some(query),
                    "repositories",
                    "repo",
                    get_test_ids("repositories"),
                )?;
            }
            ("code" | "repositories", "owned_by") => {
                let mut query = self.clone_for("users");
                for (arg_name, arg_value) in &subquery.args {
                    query.apply_filter(arg_name, arg_value.clone())?;
                }

                self.apply_ids_filter(Some(query), "users", "user", get_test_ids("users"))?;
            }
            _ => {
                if matches!(object_type.as_str(), "code" | "repositories") {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!(
                            "Subquery `{subquery_name}` is not supported for GitHub `{object_type}`"
                        ),
                    ));
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Subqueries are not supported for GitHub `{object_type}`"),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Apply IDs as a filter with the given qualifier
    ///
    /// If a query is provided, it will be used to fetch IDs. Otherwise, a new query
    /// for the entity_type will be created from self.
    fn apply_ids_filter(
        &mut self,
        query: Option<GitHubQuery>,
        entity_type: &str,
        qualifier: &str,
        test_ids: Vec<String>,
    ) -> Result<(), Error> {
        // GitHub search API has a limit of 5 AND/OR/NOT operators per query
        // See https://docs.github.com/en/rest/search/search?apiVersion=2022-11-28#limitations-on-query-length
        let ids = if testing() {
            Some(test_ids)
        } else {
            let q = query.unwrap_or_else(|| self.clone_for(entity_type));
            q.ids(5)
        };

        if let Some(mut ids) = ids {
            if !ids.is_empty() {
                ids.truncate(5);
                let filter = ids
                    .into_iter()
                    .map(|id| format!("{qualifier}:{id}"))
                    .join(" OR ");
                self.filters.push(filter);
            }
        }
        Ok(())
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

        // Avoid the error "The search contains only logical operators (AND / OR / NOT) without any search terms."
        if query.is_empty() {
            query.push('.');
        } else if query.starts_with("NOT") || query.starts_with("OR") {
            query.insert_str(0, ". ");
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

    /// Get identifiers from the query results
    ///
    /// Returns a list of identifiers based on the object type:
    /// - For repositories: returns full repo names (owner/name)
    /// - For users: returns login names
    ///
    /// Note: Results are limited to 100 items due to GitHub API pagination limits.
    /// When used in subqueries, only the first 5 IDs will be used due to GitHub's
    /// limitation on boolean operators in search queries.
    #[tracing::instrument(skip(self))]
    pub fn ids(&self, limit: usize) -> Option<Vec<String>> {
        let mut query = self.clone();
        query.limit = Some(limit);

        let url = query.generate();
        let object_type = query.object_type.as_str();

        let result: Result<Vec<String>> = task::block_in_place(|| {
            runtime::Handle::current().block_on(async {
                Ok(match object_type {
                    "repositories" => {
                        let response = request::<SearchRepositoriesResponse>(&url).await?;
                        response
                            .items
                            .into_iter()
                            .map(|repo| repo.full_name)
                            .collect()
                    }
                    "users" => {
                        let response = request::<SearchUsersResponse>(&url).await?;
                        response.items.into_iter().map(|user| user.login).collect()
                    }
                    _ => {
                        bail!("Getting IDs for GitHub `{object_type}` objects not yet enabled")
                    }
                })
            })
        });

        match result {
            Ok(ids) => {
                if ids.is_empty() {
                    None
                } else {
                    Some(ids)
                }
            }
            Err(error) => {
                self.add_error_message(format!("GitHub API request for IDs failed: {error}"));
                None
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

/// Get test IDs for a given object type
fn get_test_ids(object_type: &str) -> Vec<String> {
    match object_type {
        "users" => vec!["octocat", "github"],
        "repositories" => vec!["pandas-dev/pandas", "pola-rs/polars"],
        _ => vec![],
    }
    .into_iter()
    .map(String::from)
    .collect()
}

impl Object for GitHubQuery {
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
        let apply_method_args = |query: &mut GitHubQuery| -> Result<(), Error> {
            let (arg, kwargs): (Option<Value>, Kwargs) = from_args(args)?;
            if let Some(value) = arg {
                if let Some(value) = value.as_str() {
                    query.search = Some(value.into());
                }
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
                            "Semantic similarity filtering is not available for GitHub, use `search` instead",
                        ));
                    }
                    _ => query.apply_filter(arg, value)?,
                }
            }
            Ok(())
        };

        let query = match name {
            // Core API URL building methods
            "code" | "users" | "repositories" | "repos" => {
                let object_type = match name {
                    "repos" => "repositories",
                    _ => name,
                };

                // Check if this is a chained call (e.g., users().repositories())
                if !self.is_base() {
                    // Validate the chain is allowed
                    let valid_chain = matches!(
                        (self.object_type.as_str(), object_type),
                        ("users", "repositories") | ("users", "code") | ("repositories", "code")
                    );

                    if !valid_chain {
                        return Err(Error::new(
                            ErrorKind::InvalidOperation,
                            format!(
                                "Cannot query for `{}` within `{}`",
                                object_type, self.object_type
                            ),
                        ));
                    }

                    let mut query = self.clone_for(object_type);

                    // Add filters based on the parent query's IDs
                    let qualifier = match self.object_type.as_str() {
                        "users" => "user",
                        "repositories" => "repo",
                        _ => unreachable!(),
                    };

                    query.apply_ids_filter(
                        Some((**self).clone()),
                        &self.object_type,
                        qualifier,
                        get_test_ids(&self.object_type),
                    )?;

                    // Apply any additional filters from the method arguments
                    apply_method_args(&mut query)?;

                    query
                } else {
                    // Normal (non-chained) call
                    let mut query = self.clone_for(object_type);

                    // Apply method arguments
                    apply_method_args(&mut query)?;

                    query
                }
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
