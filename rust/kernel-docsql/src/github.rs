use std::sync::{Arc, Mutex as SyncMutex};

use kernel_jinja::{
    kernel::{
        common::{
            eyre::Result,
            itertools::Itertools,
            once_cell::sync::Lazy,
            reqwest::Client,
            serde_json::{self, Value as JsonValue},
            tokio::{runtime, task},
            tracing,
        },
        schema::{CodeChunk, ExecutionMessage, MessageLevel, Node},
    },
    minijinja::{
        Environment, Error, ErrorKind, State, Value,
        value::{Kwargs, Object, from_args},
    },
};

use crate::cypher::{NodeProxies, NodeProxy, Subquery};

const API_BASE_URL: &str = "https://api.github.com";

// HTTP client for GitHub API calls
static CLIENT: Lazy<Client> = Lazy::new(Client::new);

/// Add GitHub functions to the Jinja environment
pub(crate) fn add_github_functions(
    env: &mut Environment,
    messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>,
) {
    let github = Arc::new(GitHubQuery::new(messages.clone()));
    env.add_global("github", Value::from_object((*github).clone()));
}

/// GitHub query builder for generating API calls
#[derive(Debug, Clone)]
pub(crate) struct GitHubQuery {
    /// The GitHub entity type (repositories, users, code, etc.)
    entity_type: String,

    /// Execution messages to be added to when executing the query
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,

    /// Whether this is a base GitHub query object
    pub is_database: bool,

    /// Search query parameters
    search_query: Vec<String>,

    /// Filter parameters for the API call
    filters: Vec<String>,

    /// Sort parameter (e.g., "stars", "updated")
    sort: Option<String>,

    /// Sort order (asc or desc)
    order: Option<String>,

    /// Pagination parameters
    page: Option<u32>,
    per_page: Option<u32>,
    skip_count: Option<usize>,

    /// Fields to select in response
    select_fields: Vec<String>,
}

impl GitHubQuery {
    /// Create a new GitHub query
    pub fn new(messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self {
            entity_type: "repositories".into(),
            messages,
            is_database: true,
            search_query: Vec::new(),
            filters: Vec::new(),
            sort: None,
            order: None,
            page: None,
            per_page: None,
            skip_count: None,
            select_fields: Vec::new(),
        }
    }

    /// Set the entity type for the query (repositories, users, code, etc.)
    fn entity(&self, entity_type: &str) -> Self {
        let mut query = self.clone();
        query.entity_type = entity_type.into();
        query.is_database = false;
        query
    }

    /// Add a search term or filter to the query
    fn filter(&self, property: &str, operator: &str, value: Value) -> Result<Self, Error> {
        let mut query = self.clone();

        let filter_string = match operator {
            "search" => {
                if let Some(search_value) = value.as_str() {
                    query.search_query.push(search_value.to_string());
                    return Ok(query);
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "Search value must be a string",
                    ));
                }
            }
            "==" | "" => {
                // Map properties to GitHub search qualifiers
                match property {
                    "language" => format!("language:{}", format_filter_value(value)?),
                    "stars" => format!("stars:{}", format_filter_value(value)?),
                    "size" => format!("size:{}", format_filter_value(value)?),
                    "created" => format!("created:{}", format_filter_value(value)?),
                    "updated" => format!("pushed:{}", format_filter_value(value)?),
                    "extension" => format!("extension:{}", format_filter_value(value)?),
                    "filename" => format!("filename:{}", format_filter_value(value)?),
                    "path" => format!("path:{}", format_filter_value(value)?),
                    "user" => format!("user:{}", format_filter_value(value)?),
                    "org" => format!("org:{}", format_filter_value(value)?),
                    "topic" => format!("topic:{}", format_filter_value(value)?),
                    "license" => format!("license:{}", format_filter_value(value)?),
                    "is_public" => {
                        if value.is_true() {
                            "is:public".to_string()
                        } else {
                            "is:private".to_string()
                        }
                    }
                    _ => format!("{}:{}", property, format_filter_value(value)?),
                }
            }
            "!=" => match property {
                "language" => format!("-language:{}", format_filter_value(value)?),
                "user" => format!("-user:{}", format_filter_value(value)?),
                "org" => format!("-org:{}", format_filter_value(value)?),
                _ => format!("-{}:{}", property, format_filter_value(value)?),
            },
            ">" => format!("{}:>{}", property, format_filter_value(value)?),
            ">=" => format!("{}:>={}", property, format_filter_value(value)?),
            "<" => format!("{}:<{}", property, format_filter_value(value)?),
            "<=" => format!("{}:<={}", property, format_filter_value(value)?),
            "^=" => format!("{}:{}*", property, format_filter_value(value)?),
            "$=" => format!("{}:*{}", property, format_filter_value(value)?),
            "in" => {
                // Handle list values for 'in' operator
                // For now, treat as single value since minijinja Value doesn't have as_seq()
                // TODO: Implement proper array handling if needed
                format!("{}:{}", property, format_filter_value(value)?)
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Unsupported operator: {operator}"),
                ));
            }
        };

        if !filter_string.is_empty() {
            query.filters.push(filter_string);
        }
        Ok(query)
    }

    /// Add a search term
    fn search(&self, term: String) -> Self {
        let mut query = self.clone();
        query.search_query.push(term);
        query
    }

    /// Set sort parameter
    fn order_by(&self, field: String, direction: Option<String>) -> Result<Self, Error> {
        let mut query = self.clone();

        // Map DocsQL property names to GitHub API sort field names
        let github_field = self.map_property_to_github(&field)?;

        query.sort = Some(github_field);
        query.order = Some(match direction {
            Some(dir) if dir.to_uppercase() == "DESC" => "desc".to_string(),
            _ => "asc".to_string(),
        });
        Ok(query)
    }

    /// Set pagination limit
    fn limit(&self, count: usize) -> Self {
        let mut query = self.clone();
        query.per_page = Some(count as u32);
        query
    }

    /// Set pagination offset
    fn skip(&self, count: usize) -> Self {
        let mut query = self.clone();
        query.skip_count = Some(count);
        query
    }

    /// Select specific fields
    fn select(&self, fields: &[Value]) -> Result<Self, Error> {
        let mut query = self.clone();

        for field in fields {
            if let Some(field_name) = field.as_str() {
                query.select_fields.push(field_name.to_string());
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "Field names must be strings",
                ));
            }
        }

        Ok(query)
    }

    /// Return count of results (using GitHub's search API count)
    fn count(&self) -> Self {
        let mut query = self.clone();
        query.per_page = Some(1); // Minimal results to get total count
        query
    }

    /// Apply a DocsQL filter with transformed syntax
    fn apply_docsql_filter(&self, property: &str, value: Value) -> Result<Self, Error> {
        // Handle subquery filters (e.g., ...topics(.name == "data-science"))
        if property == "_" {
            if let Some(subquery) = value.downcast_object_ref::<Subquery>() {
                return self.apply_subquery_filter(subquery);
            }
        }

        // Handle transformed DocsQL filter syntax
        let (clean_property, operator) = if property.len() > 1 {
            if let Some(last_char) = property.chars().last() {
                match last_char {
                    '0' => (property.trim_end_matches('0'), "!="),
                    '1' => (property.trim_end_matches('1'), "<"),
                    '2' => (property.trim_end_matches('2'), "<="),
                    '3' => (property.trim_end_matches('3'), ">"),
                    '4' => (property.trim_end_matches('4'), ">="),
                    '5' => (property.trim_end_matches('5'), "~="),
                    '6' => (property.trim_end_matches('6'), "~!"),
                    '7' => (property.trim_end_matches('7'), "^="),
                    '8' => (property.trim_end_matches('8'), "$="),
                    '9' => (property.trim_end_matches('9'), "in"),
                    '_' => (property.trim_end_matches('_'), "has"),
                    _ => (property, "=="),
                }
            } else {
                (property, "==")
            }
        } else {
            (property, "==")
        };

        // Handle different operators
        match operator {
            "~=" => {
                // Regex not directly supported, convert to search
                let search_value = value
                    .as_str()
                    .map(String::from)
                    .unwrap_or_else(|| value.to_string());
                Ok(self.search(format!("{clean_property}:{search_value}")))
            }
            _ => self.filter(clean_property, operator, value),
        }
    }

    /// Apply a subquery filter to the GitHub query
    fn apply_subquery_filter(&self, subquery: &Subquery) -> Result<Self, Error> {
        let mut query = self.clone();

        // Map the subquery relation to GitHub search qualifier
        match (
            subquery.first_table.as_str(),
            subquery.first_relation.as_str(),
        ) {
            ("String", "[topics]") => {
                // Topics subquery
                for (property, operator, value) in &subquery.raw_filters {
                    if property == "name" {
                        let topic_filter =
                            self.build_github_subquery_filter("topic", operator, value)?;
                        if !topic_filter.is_empty() {
                            query.filters.push(topic_filter);
                        }
                    }
                }
            }
            ("Person", "[owners]") => {
                // Owner subquery
                for (property, operator, value) in &subquery.raw_filters {
                    match property.as_str() {
                        "name" => {
                            let user_filter =
                                self.build_github_subquery_filter("user", operator, value)?;
                            if !user_filter.is_empty() {
                                query.filters.push(user_filter);
                            }
                        }
                        "type" => {
                            // Handle org vs user type
                            if let Some(type_str) = value.as_str() {
                                if type_str == "Organization" {
                                    // This is an organization filter, not user
                                    // We'll need to handle this differently
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            ("File", _) => {
                // Files subquery - this would need special handling
                // as GitHub API doesn't directly support file subqueries in repo search
                for (property, operator, value) in &subquery.raw_filters {
                    match property.as_str() {
                        "extension" => {
                            let ext_filter =
                                self.build_github_subquery_filter("extension", operator, value)?;
                            if !ext_filter.is_empty() {
                                query.filters.push(ext_filter);
                            }
                        }
                        "filename" => {
                            let filename_filter =
                                self.build_github_subquery_filter("filename", operator, value)?;
                            if !filename_filter.is_empty() {
                                query.filters.push(filename_filter);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!(
                        "Unsupported subquery type: {} with relation {}",
                        subquery.first_table, subquery.first_relation
                    ),
                ));
            }
        }

        // Handle count filters if present
        if let Some(_count_filter) = &subquery.count {
            // Convert count filter to GitHub API format
            // Note: GitHub doesn't support all count-based filtering like OpenAlex
            tracing::warn!("Count-based subquery filters are limited in GitHub API");
        }

        Ok(query)
    }

    /// Build GitHub search filter from original property, operator, and value for subqueries
    fn build_github_subquery_filter(
        &self,
        property: &str,
        operator: &str,
        value: &Value,
    ) -> Result<String, Error> {
        let filter_value = format_filter_value(value.clone())?;

        match operator {
            "==" => Ok(format!("{property}:{filter_value}")),
            "!=" => Ok(format!("-{property}:{filter_value}")),
            "^=" => Ok(format!("{property}:{filter_value}*")),
            "$=" => Ok(format!("{property}:*{filter_value}")),
            "in" => {
                // For GitHub, "in" operator should create multiple OR conditions
                // For now, treat as single value since we don't have proper array handling
                Ok(format!("{property}:{filter_value}"))
            }
            _ => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported operator for GitHub subquery: {operator}"),
            )),
        }
    }

    /// Map DocsQL property names to GitHub API field names
    fn map_property_to_github(&self, property: &str) -> Result<String, Error> {
        let mapped = match property {
            // Repository properties
            "name" => "name",
            "description" => "description",
            "stars" => "stars",
            "forks" => "forks",
            "size" => "size",
            "created" => "created",
            "updated" => "updated",
            "language" => "language",

            // Sort-specific mappings
            "stargazers_count" => "stars",
            "forks_count" => "forks",
            "updated_at" => "updated",
            "created_at" => "created",

            // User properties
            "login" => "login",
            "followers" => "followers",
            "following" => "following",
            "public_repos" => "repos",

            // File properties
            "filename" => "filename",
            "path" => "path",
            "extension" => "extension",

            // If no mapping found, use as-is
            _ => property,
        };

        Ok(mapped.to_string())
    }

    /// Generate the GitHub API URL
    pub fn generate(&self) -> String {
        let mut url = match self.entity_type.as_str() {
            "repositories" => format!("{API_BASE_URL}/search/repositories"),
            "users" => format!("{API_BASE_URL}/search/users"),
            "code" | "files" => format!("{API_BASE_URL}/search/code"),
            _ => format!("{API_BASE_URL}/search/repositories"), // default
        };

        let mut query_params = Vec::new();

        // Build search query
        let mut search_parts = Vec::new();
        search_parts.extend(self.search_query.clone());
        search_parts.extend(self.filters.clone());

        // GitHub API requires a 'q' parameter for search endpoints
        // If no specific search terms, use a wildcard that matches all
        let search_string = if search_parts.is_empty() {
            "*".to_string() // Wildcard to match all results
        } else {
            search_parts.join(" ")
        };
        query_params.push(("q".to_string(), search_string));

        // Add sort
        if let Some(sort) = &self.sort {
            query_params.push(("sort".to_string(), sort.clone()));
        }

        // Add order
        if let Some(order) = &self.order {
            query_params.push(("order".to_string(), order.clone()));
        }

        // Add pagination - combine skip and limit into per_page
        let final_per_page = match (self.skip_count, self.per_page) {
            (Some(skip), Some(limit)) => {
                // When both skip and limit are present, GitHub API approximation
                // uses per_page = skip + limit to get enough results
                // Note: This is an approximation and may not give exact results
                Some(skip as u32 + limit)
            }
            (Some(skip), None) => {
                // Skip without limit - use skip as per_page (approximation)
                Some(skip as u32)
            }
            (None, Some(limit)) => {
                // Limit without skip - use limit as per_page
                Some(limit)
            }
            (None, None) => {
                // No pagination specified
                None
            }
        };

        if let Some(page) = self.page {
            query_params.push(("page".to_string(), page.to_string()));
        }
        if let Some(per_page) = final_per_page {
            query_params.push(("per_page".to_string(), per_page.to_string()));
        }

        // Build query string
        if !query_params.is_empty() {
            let query_string = query_params
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(&v)))
                .join("&");
            url.push('?');
            url.push_str(&query_string);
        }

        url
    }

    /// Execute the query and return the resulting [`Node`]s
    #[tracing::instrument(skip(self))]
    pub fn nodes(&self) -> Vec<Node> {
        let url = self.generate();

        tracing::debug!("GitHub API request: {}", url);

        let response = match task::block_in_place(move || {
            runtime::Handle::current().block_on(async move {
                let mut request = CLIENT.get(&url).header("User-Agent", "Stencila-DocsQL/1.0");

                // Add GitHub token if available
                if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                    request = request.header("Authorization", format!("Bearer {token}"));
                }

                request.send().await
            })
        }) {
            Ok(response) => response,
            Err(error) => {
                self.add_error_message(format!("HTTP request failed: {error}"));
                return Vec::new();
            }
        };

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = task::block_in_place(move || {
                runtime::Handle::current()
                    .block_on(async move { response.text().await.unwrap_or_default() })
            });
            self.add_error_message(format!("GitHub API error {status}: {error_text}"));
            return Vec::new();
        }

        // Parse JSON response
        let json: JsonValue = match task::block_in_place(move || {
            runtime::Handle::current().block_on(async move { response.json().await })
        }) {
            Ok(json) => json,
            Err(error) => {
                self.add_error_message(format!("Failed to parse JSON response: {error}"));
                return Vec::new();
            }
        };

        // Extract results from GitHub response format
        let results = if let Some(items) = json.get("items") {
            if let Some(array) = items.as_array() {
                array.clone()
            } else {
                Vec::new()
            }
        } else {
            // Single entity response or error
            Vec::new()
        };

        // Convert GitHub JSON objects to Stencila nodes
        let nodes: Vec<Node> = results
            .into_iter()
            .filter_map(|item| self.json_to_node(item))
            .collect();

        nodes
    }

    /// Convert GitHub JSON object to Stencila Node
    fn json_to_node(&self, json: JsonValue) -> Option<Node> {
        // For now, convert to a generic Object node
        // In the future, we could create specific node types for different GitHub entities
        match serde_json::from_value(json) {
            Ok(node) => Some(node),
            Err(error) => {
                tracing::warn!("Failed to convert GitHub response to Node: {}", error);
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

    /// Execute the query and return [`NodeProxies`] for all results
    fn all(&self) -> Value {
        Value::from_object(NodeProxies::new(self.nodes(), self.messages.clone()))
    }

    /// Execute and return first result as [`NodeProxy`]
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

    /// Execute and return last result as [`NodeProxy`]  
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

impl Object for GitHubQuery {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let mut query = match name {
            "repositories" | "repos" | "code" | "files" | "users" => {
                let mut query = self.entity(name);

                // Handle search argument and other special arguments
                let (kwargs,): (Kwargs,) = from_args(args)?;
                for arg in kwargs.args() {
                    let value: Value = kwargs.get(arg)?;
                    match arg {
                        "search" => {
                            if let Some(search_value) = value.as_str() {
                                query = query.search(search_value.to_string())
                            }
                        }
                        "like" => {
                            return Err(Error::new(
                                ErrorKind::UnknownMethod,
                                "semantic similarity filtering is not available for GitHub, use `search` instead",
                            ));
                        }
                        // Handle transformed DocsQL filters
                        _ => query = query.apply_docsql_filter(arg, value)?,
                    }
                }

                query
            }

            // Query methods
            "orderBy" | "order_by" => {
                let (field, direction): (String, Option<String>) = from_args(args)?;
                self.order_by(field, direction)?
            }
            "limit" => {
                let (count,): (usize,) = from_args(args)?;
                self.limit(count)
            }
            "skip" => {
                let (count,): (usize,) = from_args(args)?;
                self.skip(count)
            }
            "select" => {
                let (fields,): (&[Value],) = from_args(args)?;
                self.select(fields)?
            }
            "count" => {
                if !args.is_empty() {
                    return Err(Error::new(
                        ErrorKind::TooManyArguments,
                        format!("Method `{name}` takes no arguments."),
                    ));
                }
                self.count()
            }

            // Result retrieval methods
            "explain" => return Ok(self.explain()),
            "all" => return Ok(self.all()),
            "first" => return self.first(),
            "last" => return self.last(),

            _ => {
                return Err(Error::new(
                    ErrorKind::UnknownMethod,
                    format!("Unknown method: {name}"),
                ));
            }
        };

        query.is_database = false;
        Ok(Value::from_object(query))
    }

    fn get_value(self: &Arc<Self>, key: &Value) -> Option<Value> {
        // Handle numeric indexing
        if let Some(index) = key.as_i64() {
            let nodes = self.nodes();
            let index = if index < 0 {
                let abs_index = (-index) as usize;
                if abs_index > nodes.len() {
                    return None;
                }
                nodes.len() - abs_index
            } else {
                index as usize
            };

            if index < nodes.len() {
                let node = nodes[index].clone();
                return Some(Value::from_object(NodeProxy::new(
                    node,
                    self.messages.clone(),
                )));
            }
        }

        None
    }
}

/// Format a filter value for GitHub API
fn format_filter_value(value: Value) -> Result<String, Error> {
    if let Some(s) = value.as_str() {
        Ok(s.to_string())
    } else if let Some(n) = value.as_i64() {
        Ok(n.to_string())
    } else if value.is_none() {
        Ok("null".to_string())
    } else {
        // For complex values, convert to string representation
        Ok(value.to_string())
    }
}
