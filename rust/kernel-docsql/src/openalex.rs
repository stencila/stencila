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
        value::{from_args, Kwargs, Object},
        Environment, Error, ErrorKind, State, Value,
    },
};

use crate::query::{NodeProxies, NodeProxy, Subquery};

const API_BASE_URL: &str = "https://api.openalex.org";

// HTTP client for OpenAlex API calls
static CLIENT: Lazy<Client> = Lazy::new(Client::new);

/// OpenAlex query builder for generating API calls
#[derive(Debug, Clone)]
pub(crate) struct OpenAlexQuery {
    /// The OpenAlex entity type (works, authors, institutions, etc.)
    entity_type: String,

    /// Execution messages to be added to when executing the query
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,

    /// Whether this is a base OpenAlex query object
    pub is_database: bool,

    /// Filter parameters for the API call
    filters: Vec<String>,

    /// Search terms for general search
    search_terms: Vec<String>,

    /// Sort parameter (e.g., "cited_by_count:desc")
    sort: Option<String>,

    /// Pagination parameters
    page: Option<u32>,
    per_page: Option<u32>,

    /// Fields to select in response
    select_fields: Vec<String>,

    /// Whether to use cursor pagination for large result sets
    use_cursor: bool,
    cursor: Option<String>,
}

impl OpenAlexQuery {
    /// Create a new OpenAlex query
    pub fn new(messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self {
            entity_type: "works".into(),
            messages,
            is_database: true,
            filters: Vec::new(),
            search_terms: Vec::new(),
            sort: None,
            page: None,
            per_page: None,
            select_fields: Vec::new(),
            use_cursor: false,
            cursor: None,
        }
    }

    /// Set the entity type for the query (works, authors, institutions, etc.)
    fn entity(&self, entity_type: &str) -> Self {
        let mut query = self.clone();
        query.entity_type = entity_type.into();
        query.is_database = false;
        query
    }

    /// Add a filter to the query
    fn filter(&self, property: &str, operator: &str, value: Value) -> Result<Self, Error> {
        let mut query = self.clone();

        let filter_string = match operator {
            "==" | "" => format!("{}:{}", property, format_filter_value(value)?),
            "!=" => format!("{}:!{}", property, format_filter_value(value)?),
            "<" => format!("{}:<{}", property, format_filter_value(value)?),
            "<=" => format!("{}:<={}", property, format_filter_value(value)?),
            ">" => format!("{}:>{}", property, format_filter_value(value)?),
            ">=" => format!("{}:>={}", property, format_filter_value(value)?),
            "search" => {
                if let Some(search_value) = value.as_str() {
                    query.search_terms.push(search_value.to_string());
                    return Ok(query);
                } else {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        "Search value must be a string",
                    ));
                }
            }
            "in" => {
                // Handle list values for 'in' operator
                if value.is_true() || value.is_none() {
                    // Not actually a sequence, treat as single value
                    format!("{}:{}", property, format_filter_value(value)?)
                } else {
                    // Try to handle as array
                    format!("{}:{}", property, format_filter_value(value)?)
                }
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Unsupported operator: {operator}"),
                ));
            }
        };

        query.filters.push(filter_string);
        Ok(query)
    }

    /// Add a search term
    fn search(&self, term: String) -> Self {
        let mut query = self.clone();
        query.search_terms.push(term);
        query
    }

    /// Set sort parameter
    fn order_by(&self, field: String, direction: Option<String>) -> Result<Self, Error> {
        let mut query = self.clone();
        
        // Map DocsQL property names to OpenAlex API sort field names
        let openalex_field = self.map_property_to_openalex(&field)?;
        
        let sort_string = match direction {
            Some(dir) if dir.to_uppercase() == "DESC" => format!("{}:desc", openalex_field),
            _ => format!("{}:asc", openalex_field),
        };
        query.sort = Some(sort_string);
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
        query.page = Some((count / query.per_page.unwrap_or(25) as usize) as u32 + 1);
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

    /// Return count of results
    fn count(&self) -> Self {
        let mut query = self.clone();
        query.per_page = Some(0); // OpenAlex returns count in meta when per_page=0
        query
    }

    /// Apply a DocsQL filter with transformed syntax
    fn apply_docsql_filter(&self, property: &str, value: Value) -> Result<Self, Error> {
        // Handle subquery filters (e.g., ...authors(.name ^= "Smith"))
        if property == "_" {
            if let Some(subquery) = value.downcast_object_ref::<Subquery>() {
                return self.apply_subquery_filter(subquery);
            }
        }

        // Handle transformed DocsQL filter syntax
        // The property name contains encoded operator information from transform_filters

        let (clean_property, operator) = if property.len() > 1 {
            if let Some(last_char) = property.chars().last() {
                match last_char {
                    '0' => (property.trim_end_matches('0'), "!="),
                    '1' => (property.trim_end_matches('1'), "<"),
                    '2' => (property.trim_end_matches('2'), "<="),
                    '3' => (property.trim_end_matches('3'), ">"),
                    '4' => (property.trim_end_matches('4'), ">="),
                    '5' => (property.trim_end_matches('5'), "~="),
                    '6' => (property.trim_end_matches('6'), "!~"),
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

        // Map DocsQL property names to OpenAlex API filters
        let openalex_property = self.map_property_to_openalex(clean_property)?;

        // Handle different operators
        match operator {
            "^=" | "$=" | "!~" => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported operator: {operator}"),
            )),
            "~=" => {
                // Regex match: use search for now
                let search_value = value
                    .as_str()
                    .map(String::from)
                    .unwrap_or_else(|| value.to_string());
                Ok(self.search(format!("{}:{}", openalex_property, search_value)))
            }
            _ => self.filter(&openalex_property, operator, value),
        }
    }

    /// Apply a subquery filter to the OpenAlex query
    fn apply_subquery_filter(&self, subquery: &Subquery) -> Result<Self, Error> {
        let mut query = self.clone();

        // Map the subquery relation to OpenAlex filter prefix
        let filter_prefix = match subquery.first_table.as_str() {
            "Person" => "authorships.author", // Authors subquery
            "Reference" => "references", // References subquery maps to reference count
            "Organization" => "authorships.institutions", // Affiliations subquery
            _ => return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported subquery type: {}", subquery.first_table),
            )),
        };

        // Process the raw filters within the subquery
        for (property, operator, value) in &subquery.raw_filters {
            // Handle nested subqueries (properties that start with _)
            if property == "_" {
                // This is a nested subquery
                if let Some(nested_subquery) = value.downcast_object_ref::<Subquery>() {
                    return self.apply_subquery_filter(nested_subquery);
                }
            } else {
                // Build OpenAlex filter directly from original property, operator, and value
                let openalex_filter = self.build_openalex_subquery_filter(property, operator, value, filter_prefix, &subquery.first_table)?;
                query.filters.push(openalex_filter);
            }
        }

        // Handle count filters if present
        if let Some(count_filter) = &subquery.count {
            // Convert count filter to OpenAlex API format
            let count_property = match subquery.first_table.as_str() {
                "Reference" => "referenced_works_count",
                "Person" => "authors_count",
                "Organization" => "institutions_distinct_count",
                _ => return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Count subqueries not supported for {}", subquery.first_table),
                )),
            };
            
            // Parse the count filter (e.g., "> 10", "= 5", "<= 20")
            // Remove spaces to get ">" + "10" format
            let clean_count_filter = count_filter.replace(" ", "");
            let count_filter_str = format!("{}:{}", count_property, clean_count_filter);
            query.filters.push(count_filter_str);
        }

        Ok(query)
    }

    /// Build OpenAlex API filter from original property, operator, and value for subqueries
    fn build_openalex_subquery_filter(&self, property: &str, operator: &str, value: &Value, prefix: &str, table: &str) -> Result<String, Error> {
        // Handle different property mappings based on the subquery type
        match (table, property) {
            ("Person", "name") => {
                // For author names, use raw_author_name.search instead of authorships.author.display_name
                return self.build_author_name_filter(operator, value);
            },
            ("Organization", "name") => {
                // For organization/institution names, use raw_affiliation_strings.search
                return self.build_organization_name_filter(operator, value);
            },
            ("Organization", property) => {
                // For other organization properties, use the authorships.institutions prefix
                let filter_value = format_filter_value(value.clone())?;
                
                match operator {
                    "==" => Ok(format!("{}.{}:{}", prefix, property, filter_value)),
                    "!=" => Ok(format!("{}.{}:!{}", prefix, property, filter_value)),
                    "<" => Ok(format!("{}.{}:<{}", prefix, property, filter_value)),
                    "<=" => Ok(format!("{}.{}:<={}", prefix, property, filter_value)),
                    ">" => Ok(format!("{}.{}:>{}", prefix, property, filter_value)),
                    ">=" => Ok(format!("{}.{}:>={}", prefix, property, filter_value)),
                    _ => Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unsupported operator for organization property: {}", operator),
                    )),
                }
            },
            _ => {
                // Default mapping
                let openalex_property = property;
                let filter_value = format_filter_value(value.clone())?;
                
                match operator {
                    "==" => Ok(format!("{}.{}:{}", prefix, openalex_property, filter_value)),
                    "!=" => Ok(format!("{}.{}:!{}", prefix, openalex_property, filter_value)),
                    "<" => Ok(format!("{}.{}:<{}", prefix, openalex_property, filter_value)),
                    "<=" => Ok(format!("{}.{}:<={}", prefix, openalex_property, filter_value)),
                    ">" => Ok(format!("{}.{}:>{}", prefix, openalex_property, filter_value)),
                    ">=" => Ok(format!("{}.{}:>={}", prefix, openalex_property, filter_value)),
                    _ => Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unsupported operator for subquery: {}", operator),
                    )),
                }
            }
        }
    }
    
    /// Helper method to build author name filters using raw_author_name.search
    fn build_author_name_filter(&self, operator: &str, value: &Value) -> Result<String, Error> {
        let filter_value = format_filter_value(value.clone())?;
        
        match operator {
            "==" => Ok(format!("raw_author_name.search:{}", filter_value)),
            "^=" => Ok(format!("raw_author_name.search:{}*", filter_value)),
            "$=" => Ok(format!("raw_author_name.search:*{}", filter_value)),
            _ => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported operator for author name: {}", operator),
            )),
        }
    }

    /// Helper method to build organization name filters using raw_affiliation_strings.search
    fn build_organization_name_filter(&self, operator: &str, value: &Value) -> Result<String, Error> {
        let filter_value = format_filter_value(value.clone())?;
        
        match operator {
            "==" => Ok(format!("raw_affiliation_strings.search:{}", filter_value)),
            "^=" => Ok(format!("raw_affiliation_strings.search:{}*", filter_value)),
            "$=" => Ok(format!("raw_affiliation_strings.search:*{}", filter_value)),
            _ => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported operator for organization name: {}", operator),
            )),
        }
    }


    /// Map DocsQL property names to OpenAlex API filter names
    fn map_property_to_openalex(&self, property: &str) -> Result<String, Error> {
        let mapped = match property {
            // Common mappings across entities
            "title" => match self.entity_type.as_str() {
                "works" => "display_name", // Works use display_name for both filtering and sorting
                _ => "display_name",
            },
            "name" => match self.entity_type.as_str() {
                "authors" => "display_name", // For authors, name maps to display_name
                _ => "display_name.search",
            },
            "text" => match self.entity_type.as_str() {
                "works" => "title_and_abstract.search",
                _ => "display_name.search",
            },
            "year" | "publication_year" => "publication_year",
            "date" | "publication_date" => "publication_date",
            "doi" => "doi",
            "orcid" => "orcid",
            "ror" => "ror",
            "country_code" => "country_code",
            "continent" => "continent",
            "authors_count" => "authors_count",
            "cited_by_count" => "cited_by_count",
            "is_oa" => "open_access.is_oa",
            "type" => "type",
            "language" => "language",

            // Works-specific
            "abstract" => "abstract.search",
            "cited_by" => "cited_by",
            "cites" => "cites",
            "journal" => "primary_location.source.display_name.search",
            "venue" => "primary_location.source.display_name.search",

            // Authors-specific
            "h_index" => "summary_stats.h_index",
            "i10_index" => "summary_stats.i10_index",

            // If no mapping found, use as-is
            _ => property,
        };

        Ok(mapped.to_string())
    }

    /// Generate the OpenAlex API URL
    pub fn generate(&self) -> String {
        let mut url = format!("{}/{}", API_BASE_URL, self.entity_type);
        let mut query_params = Vec::new();

        // Add filters
        if !self.filters.is_empty() {
            let filter_string = self.filters.join(",");
            query_params.push(("filter".to_string(), filter_string));
        }

        // Add search
        if !self.search_terms.is_empty() {
            let search_string = self.search_terms.join(" ");
            query_params.push(("search".to_string(), search_string));
        }

        // Add sort
        if let Some(sort) = &self.sort {
            query_params.push(("sort".to_string(), sort.clone()));
        }

        // Add pagination
        if let Some(page) = self.page {
            query_params.push(("page".to_string(), page.to_string()));
        }
        if let Some(per_page) = self.per_page {
            query_params.push(("per-page".to_string(), per_page.to_string()));
        }

        // Add cursor pagination if enabled
        if self.use_cursor {
            if let Some(cursor) = &self.cursor {
                query_params.push(("cursor".to_string(), cursor.clone()));
            }
        }

        // Add field selection
        if !self.select_fields.is_empty() {
            let select_string = self.select_fields.join(",");
            query_params.push(("select".to_string(), select_string));
        }

        // Add email if available from environment
        if let Ok(email) = std::env::var("OPENALEX_EMAIL") {
            query_params.push(("mailto".to_string(), email));
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
    pub fn nodes(&self) -> Vec<Node> {
        let url = self.generate();

        tracing::debug!("OpenAlex API request: {}", url);

        let response = match task::block_in_place(move || {
            runtime::Handle::current().block_on(async move { CLIENT.get(&url).send().await })
        }) {
            Ok(response) => response,
            Err(error) => {
                self.add_error_message(format!("HTTP request failed: {}", error));
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
            self.add_error_message(format!("OpenAlex API error {}: {}", status, error_text));
            return Vec::new();
        }

        // Parse JSON response
        let json: JsonValue = match task::block_in_place(move || {
            runtime::Handle::current().block_on(async move { response.json().await })
        }) {
            Ok(json) => json,
            Err(error) => {
                self.add_error_message(format!("Failed to parse JSON response: {}", error));
                return Vec::new();
            }
        };

        // Extract results from OpenAlex response format
        let results = if self.per_page == Some(0) {
            // Count query - return the count as an integer node
            if let Some(meta) = json.get("meta") {
                if let Some(count) = meta.get("count") {
                    if let Some(count_val) = count.as_u64() {
                        return vec![Node::Integer(count_val as i64)];
                    }
                }
            }
            Vec::new()
        } else if let Some(results) = json.get("results") {
            if let Some(array) = results.as_array() {
                array.clone()
            } else {
                Vec::new()
            }
        } else {
            // Single entity response
            vec![json]
        };

        // Convert OpenAlex JSON objects to Stencila nodes
        results
            .into_iter()
            .filter_map(|item| self.json_to_node(item))
            .collect()
    }

    /// Convert OpenAlex JSON object to Stencila Node
    fn json_to_node(&self, json: JsonValue) -> Option<Node> {
        // For now, convert to a generic Object node
        // In the future, we could create specific node types for different OpenAlex entities
        match serde_json::from_value(json) {
            Ok(node) => Some(node),
            Err(error) => {
                tracing::warn!("Failed to convert OpenAlex response to Node: {}", error);
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

impl Object for OpenAlexQuery {
    fn call_method(
        self: &Arc<Self>,
        _state: &State,
        name: &str,
        args: &[Value],
    ) -> Result<Value, Error> {
        let mut query = match name {
            "works" | "articles" | "books" | "chapters" | "preprints" | "dissertations"
            | "reviews" | "standards" | "grants" | "retractions" | "datasets" | "people"
            | "organizations" => {
                let (entity_type, type_equals) = match name {
                    "works" => ("works", None),

                    // Types of creative works
                    // See https://docs.openalex.org/api-entities/works/work-object#type
                    // See https://api.openalex.org/works?group_by=type for counts
                    "articles" => ("works", Some("article")),
                    "books" => ("works", Some("book")),
                    "chapters" => ("works", Some("book-chapter")),
                    "preprints" => ("works", Some("preprint")),
                    "dissertations" => ("works", Some("dissertation")),
                    "reviews" => ("works", Some("review")),
                    "standards" => ("works", Some("standard")),
                    "grants" => ("works", Some("grant")),
                    "retractions" => ("works", Some("retraction")),
                    "datasets" => ("works", Some("dataset")),
                    "people" => ("authors", None),
                    "organizations" => ("institutions", None),

                    _ => {
                        return Err(Error::new(
                            ErrorKind::UnknownMethod,
                            format!("Unknown method: {}", name),
                        ));
                    }
                };

                let mut query = self.entity(entity_type);

                if let Some(value) = type_equals {
                    query = query.filter("type", "==", Value::from(value))?;
                }

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
                            // For semantic similarity, we'll use search for now
                            // In the future, this could use embeddings
                            if let Some(search_value) = value.as_str() {
                                query = query.search(search_value.to_string())
                            }
                        }
                        "limit" => {
                            if let Some(limit_val) = value.as_usize() {
                                query = query.limit(limit_val)
                            }
                        }
                        "skip" => {
                            if let Some(skip_val) = value.as_usize() {
                                query = query.skip(skip_val)
                            }
                        }
                        // Handle transformed DocsQL filters
                        _ => query = query.apply_docsql_filter(arg, value)?,
                    }
                }

                query
            }

            // Query methods
            "filter" => {
                let (property, operator, value): (String, String, Value) = from_args(args)?;
                self.filter(&property, &operator, value)?
            }
            "search" => {
                let (term,): (String,) = from_args(args)?;
                self.search(term)
            }
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
                        format!("Method `{}` takes no arguments.", name),
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
                    format!("Unknown method: {}", name),
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

/// Format a filter value for OpenAlex API
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

/// Add OpenAlex functions to the Jinja environment
pub(crate) fn add_openalex_functions(env: &mut Environment, openalex: Arc<OpenAlexQuery>) {
    env.add_global("openalex", Value::from_object((*openalex).clone()));

    // TODO: consider whether to add these
    // Add convenient aliases for common entity types
    //env.add_global("articles", Value::from_object(OpenAlexEntityQuery::new("works", openalex.clone())));
    //env.add_global("people", Value::from_object(OpenAlexEntityQuery::new("authors", openalex.clone())));
    //env.add_global("organizations", Value::from_object(OpenAlexEntityQuery::new("institutions", openalex.clone())));
    //env.add_global("journals", Value::from_object(OpenAlexEntityQuery::new("sources", openalex.clone())));
}

/// Helper struct for entity-specific queries
#[derive(Debug, Clone)]
pub(crate) struct OpenAlexEntityQuery {
    entity_type: String,
    base_query: Arc<OpenAlexQuery>,
}

impl Object for OpenAlexEntityQuery {
    fn call(self: &Arc<Self>, _state: &State, args: &[Value]) -> Result<Value, Error> {
        let (_args, kwargs): (&[Value], Kwargs) = from_args(args)?;
        let mut query = self.base_query.entity(&self.entity_type);

        // Process filters from kwargs
        for arg in kwargs.args() {
            let value: Value = kwargs.get(arg)?;
            match arg {
                "search" => {
                    if let Some(search_value) = value.as_str() {
                        query = query.search(search_value.to_string());
                    }
                }
                "like" => {
                    if let Some(search_value) = value.as_str() {
                        query = query.search(search_value.to_string());
                    }
                }
                "limit" => {
                    if let Some(limit_val) = value.as_usize() {
                        query = query.limit(limit_val);
                    }
                }
                "skip" => {
                    if let Some(skip_val) = value.as_usize() {
                        query = query.skip(skip_val);
                    }
                }
                _ => {
                    query = query.apply_docsql_filter(arg, value)?;
                }
            }
        }

        Ok(Value::from_object(query))
    }
}
