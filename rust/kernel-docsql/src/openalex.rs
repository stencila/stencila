use std::sync::{Arc, Mutex as SyncMutex};

use codec_openalex::{
    AuthorsResponse, FundersResponse, InstitutionsResponse, PublishersResponse, SelectResponse,
    SourcesResponse, WorksResponse, request, url_for_list,
};
use kernel_jinja::{
    kernel::{
        common::{
            eyre::{Result, bail},
            serde_json,
            tokio::{runtime, task},
            tracing,
        },
        schema::{CodeChunk, Datatable, ExecutionMessage, MessageLevel, Node},
    },
    minijinja::{
        Environment, Error, ErrorKind, State, Value,
        value::{Kwargs, Object, ValueKind, from_args},
    },
};

use crate::{
    cypher::{NodeProxies, NodeProxy, Subquery},
    docsql::decode_filter,
};

/// Add OpenAlex functions to the Jinja environment
pub(crate) fn add_openalex_functions(
    env: &mut Environment,
    messages: &Arc<SyncMutex<Vec<ExecutionMessage>>>,
) {
    let openalex = Arc::new(OpenAlexQuery::new(messages.clone()));
    env.add_global("openalex", Value::from_object((*openalex).clone()));
}

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
    fn filter(&self, entity_type: &str, arg_name: &str, arg_value: Value) -> Result<Self, Error> {
        let mut query = self.clone();

        // Handle subquery filters (e.g., ...authors(.name ^= "Smith"))
        if arg_name == "_" {
            if let Some(subquery) = arg_value.downcast_object_ref::<Subquery>() {
                return self.apply_subquery_filter(subquery);
            }
        }

        // Extract the property name an operator from the arg
        let (property_name, mut operator) = decode_filter(arg_name);

        // Error early for unhandled operators with advice
        if operator == "~!" || operator == "^=" || operator == "$=" {
            let message = match operator {
                "~!" => "Negated search operator ~! is not supported for OpenAlex queries.",
                "^=" => {
                    "Starts-with operator ^= is not supported for OpenAlex queries. Perhaps use search operator ~= instead."
                }
                "$=" => {
                    "Ends-with operator $= is not supported for OpenAlex queries. Perhaps use search operator ~= instead."
                }
                _ => "Unsupported operator",
            };
            return Err(Error::new(ErrorKind::InvalidOperation, message));
        }

        // Map the property name to the OpenAlex filter name
        let filter_name = match entity_type {
            "works" => match property_name {
                // See https://docs.openalex.org/api-entities/works/filter-works
                // In OpenAlex it is not possible to test equality for `display_name`, only `display_name.search`
                // it available, which is also aliased to `title.search`
                "title" => "title.search",
                "name" => "display_name.search",
                // Abstract is available to search https://docs.openalex.org/api-entities/works/filter-works#abstract.search
                "abstract" => "abstract.search",
                // Renames
                "year" => "publication_year",
                "date" => "publication_date",
                // Properties on `primary_location` that we hoist up
                "license" => "primary_location.license",
                "is_accepted" => "primary_location.is_accepted",
                "is_published" => "primary_location.is_published",
                "version" => "primary_location.version",
                // Properties which do not need mapping, including convenience filters
                //  https://docs.openalex.org/api-entities/works/filter-works#works-convenience-filters
                "doi" | "is_oa" | "oa_status" | "has_abstract" | "has_references" | "has_doi"
                | "has_orcid" | "has_pmcid" | "has_pmid" | "cited_by_count" => property_name,
                // Error for all others
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unhandled filter property for OpenAlex works: {property_name}"),
                    ));
                }
            },
            "authors" => match property_name {
                // See https://docs.openalex.org/api-entities/authors/filter-authors
                "name" => "display_name.search",
                // Properties on `summary_stats` that we hoist up
                "h_index" => "summary_stats.h_index",
                "i10_index" => "summary_stats.i10_index",
                // Properties which do not need mapping, including convenience filters
                //  https://docs.openalex.org/api-entities/authors/filter-authors#authors-convenience-filters
                "orcid" | "has_orcid" | "works_count" | "cited_by_count" => property_name,
                // Error for all others
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unhandled filter property for OpenAlex authors: {property_name}"),
                    ));
                }
            },
            "institutions" => match property_name {
                // See https://docs.openalex.org/api-entities/institutions/filter-institutions
                "name" => "display_name.search",
                // Properties on `summary_stats` that we hoist up
                "h_index" => "summary_stats.h_index",
                "i10_index" => "summary_stats.i10_index",
                // Properties which do not need mapping, including convenience filters
                //  https://docs.openalex.org/api-entities/institutions/filter-institutions#institutions-convenience-filters
                "ror" | "has_ror" | "works_count" | "cited_by_count" | "type"
                | "is_global_south" => property_name,
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!(
                            "Unhandled filter property for OpenAlex institutions: {property_name}"
                        ),
                    ));
                }
            },
            "sources" => match property_name {
                // See https://docs.openalex.org/api-entities/sources/filter-sources
                "name" => "display_name.search",
                // Properties on `summary_stats` that we hoist up
                "h_index" => "summary_stats.h_index",
                "i10_index" => "summary_stats.i10_index",
                // Properties which do not need mapping, including convenience filters
                //  https://docs.openalex.org/api-entities/sources/filter-sources#sources-convenience-filters
                "issn" | "has_issn" | "is_oa" | "works_count" | "cited_by_count"
                | "is_global_south" => property_name,
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unhandled filter property for OpenAlex sources: {property_name}"),
                    ));
                }
            },
            "publishers" => match property_name {
                // See https://docs.openalex.org/api-entities/publishers/filter-publishers
                "name" => "display_name.search",
                // Properties on `summary_stats` that we hoist up
                "h_index" => "summary_stats.h_index",
                "i10_index" => "summary_stats.i10_index",
                // Properties which do not need mapping, including convenience filters
                //  https://docs.openalex.org/api-entities/publishers/filter-publishers#publishers-convenience-filters
                "ror" | "works_count" | "cited_by_count" => property_name,
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!(
                            "Unhandled filter property for OpenAlex publishers: {property_name}"
                        ),
                    ));
                }
            },
            "funders" => match property_name {
                // See https://docs.openalex.org/api-entities/funders/filter-funders
                "name" => "display_name.search",
                "description" => "description.search",
                // Properties on `summary_stats` that we hoist up
                "h_index" => "summary_stats.h_index",
                "i10_index" => "summary_stats.i10_index",
                // Properties which do not need mapping, including convenience filters
                //  https://docs.openalex.org/api-entities/funders/filter-funders#funders-convenience-filters
                "ror" | "grants_count" | "works_count" | "cited_by_count" | "is_global_south" => {
                    property_name
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unhandled filter property for OpenAlex funders: {property_name}"),
                    ));
                }
            },
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!(
                        "Unhandled filter property for OpenAlex {entity_type}: {property_name}"
                    ),
                ));
            }
        };

        // Transform the minijinja argument value into a string
        let mut filter_value = format_filter_value(&arg_value);

        // Further entity_type and property transformations for user convenience
        if entity_type == "works" && property_name == "version" {
            if !filter_value.ends_with("Version") {
                // published => publishedVersion etc
                filter_value.push_str("Version");
            }
        }

        // Support <= and >= operators by transforming to < and > respectively
        if matches!(arg_value.kind(), ValueKind::Number)
            && let Some(num) = arg_value.as_i64()
        {
            if operator == "<=" {
                operator = "<";
                filter_value = num.saturating_add(1).to_string()
            } else if operator == ">=" {
                operator = ">";
                filter_value = num.saturating_sub(1).to_string()
            }
        }

        // Generate the filter string
        let filter_string = match operator {
            "==" => format!("{filter_name}:{filter_value}"),
            "!=" => format!("{filter_name}:!{filter_value}"),

            "<" => format!("{filter_name}:<{filter_value}"),
            ">" => format!("{filter_name}:>{filter_value}"),

            "~=" => {
                if filter_name.ends_with(".search") {
                    format!("{filter_name}:{filter_value}")
                } else {
                    format!("{filter_name}.search:{filter_value}")
                }
            }

            "in" => {
                // Handle list values for 'in' operator
                if arg_value.is_true() || arg_value.is_none() {
                    // Not actually a sequence, treat as single value
                    format!("{filter_name}:{filter_value}")
                } else {
                    // Try to handle as array
                    format!("{filter_name}:{filter_value}")
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
    fn order_by(&self, property: &str, direction: Option<String>) -> Result<Self, Error> {
        let mut query = self.clone();

        // Map the field name to the OpenAlex attribute name
        let attribute = match self.entity_type.as_str() {
            "works" => match property {
                "title" | "name" => "display_name",
                "year" => "publication_year",
                "date" => "publication_date",
                "license" => "primary_location.license",
                "is_accepted" => "primary_location.is_accepted",
                "is_published" => "primary_location.is_published",
                "version" => "primary_location.version",
                _ => property,
            },
            _ => match property {
                "name" => "display_name",
                "h_index" => "summary_stats.h_index",
                "i10_index" => "summary_stats.i10_index",
                _ => property,
            },
        };

        let sort_string = match direction {
            Some(dir) if dir.to_uppercase() == "DESC" => format!("{attribute}:desc"),
            _ => format!("{attribute}:asc"),
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

    /// Apply a subquery filter to the OpenAlex query
    fn apply_subquery_filter(&self, subquery: &Subquery) -> Result<Self, Error> {
        let mut query = self.clone();

        // Map the subquery relation to OpenAlex filter prefix
        let filter_prefix = match (
            subquery.first_table.as_str(),
            subquery.first_relation.as_str(),
        ) {
            ("Person", _) => "authorships.author", // Authors subquery
            ("Reference", "[references]") => "references", // References subquery maps to reference count
            ("Reference", "[citedBy]") => "citedBy", // CitedBy subquery maps to cited_by_count
            ("Organization", "[affiliations]") => "authorships.institutions", // Affiliations subquery
            ("Organization", _) => "authorships.institutions", // Default organization subquery
            ("Periodical", "[publishedIn]") => "primary_location.source", // PublishedIn subquery maps to source
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!(
                        "Unsupported subquery type: {} with relation {}",
                        subquery.first_table, subquery.first_relation
                    ),
                ));
            }
        };

        // Handle query objects for ID-based filtering (e.g., citedBy with OpenAlex query)
        if !subquery.query_objects.is_empty() && subquery.first_relation == "[citedBy]" {
            // Extract IDs from query objects and use them in cited_by filter
            let work_ids = self.extract_work_ids_from_query_objects(&subquery.query_objects)?;
            if !work_ids.is_empty() {
                let ids_filter = work_ids.join("|");
                query.filters.push(format!("cited_by:{ids_filter}"));
            }
        }

        // Handle query objects for publishedIn subqueries
        if !subquery.query_objects.is_empty() && subquery.first_relation == "[publishedIn]" {
            // Extract source IDs from query objects and use them in primary_location.source.id filter
            let source_ids = self.extract_source_ids_from_query_objects(&subquery.query_objects)?;
            if !source_ids.is_empty() {
                let ids_filter = source_ids.join("|");
                query
                    .filters
                    .push(format!("primary_location.source.id:{ids_filter}"));
            }
        }

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
                let openalex_filter = self.build_openalex_subquery_filter(
                    property,
                    operator,
                    value,
                    filter_prefix,
                    &subquery.first_table,
                )?;
                query.filters.push(openalex_filter);
            }
        }

        // Handle count filters if present
        if let Some(count_filter) = &subquery.count {
            // Convert count filter to OpenAlex API format
            let count_property = match (
                subquery.first_table.as_str(),
                subquery.first_relation.as_str(),
            ) {
                ("Reference", "[references]") => "referenced_works_count",
                ("Reference", "[citedBy]") => "cited_by_count",
                ("Person", _) => "authors_count",
                ("Organization", _) => "institutions_distinct_count",
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!(
                            "Count subqueries not supported for {} with relation {}",
                            subquery.first_table, subquery.first_relation
                        ),
                    ));
                }
            };

            // Parse the count filter (e.g., "> 10", "= 5", "<= 20")
            // Remove spaces to get ">" + "10" format
            let clean_count_filter = count_filter.replace(" ", "");

            // OpenAlex doesn't consistently support >= and <= operators
            // Convert them to equivalent > and < operators
            let converted_filter = self.convert_count_filter_for_openalex(&clean_count_filter)?;
            let count_filter_str = format!("{count_property}:{converted_filter}");
            query.filters.push(count_filter_str);
        }

        Ok(query)
    }

    /// Build OpenAlex API filter from original property, operator, and value for subqueries
    fn build_openalex_subquery_filter(
        &self,
        property: &str,
        operator: &str,
        value: &Value,
        prefix: &str,
        table: &str,
    ) -> Result<String, Error> {
        // Handle different property mappings based on the subquery type
        match (table, property) {
            ("Person", "name") => {
                // For author names, use raw_author_name.search instead of authorships.author.display_name
                self.build_author_name_filter(operator, value)
            }
            ("Periodical", "name") | ("Periodical", "display_name") => {
                // For publishedIn source names, there's no direct search field available
                // Return an error suggesting to use query objects instead
                Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "Source name filtering not supported in publishedIn subqueries. Use query objects like: ...publishedIn(openalex.sources(.display_name == \"bioRxiv\"))".to_string(),
                ))
            }
            ("Periodical", property) => {
                // For other periodical properties, use the primary_location.source prefix
                let filter_value = format_filter_value(&value);

                match operator {
                    "==" => Ok(format!("{prefix}.{property}:{filter_value}")),
                    "!=" => Ok(format!("{prefix}.{property}:!{filter_value}")),
                    "<" => Ok(format!("{prefix}.{property}:<{filter_value}")),
                    "<=" => Ok(format!("{prefix}.{property}:<={filter_value}")),
                    ">" => Ok(format!("{prefix}.{property}:>{filter_value}")),
                    ">=" => Ok(format!("{prefix}.{property}:>={filter_value}")),
                    _ => Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unsupported operator for periodical property: {operator}"),
                    )),
                }
            }
            ("Organization", "name") => {
                // For organization/institution names, use raw_affiliation_strings.search
                self.build_organization_name_filter(operator, value)
            }
            ("Organization", property) => {
                // For other organization properties, use the authorships.institutions prefix
                let filter_value = format_filter_value(&value);

                match operator {
                    "==" => Ok(format!("{prefix}.{property}:{filter_value}")),
                    "!=" => Ok(format!("{prefix}.{property}:!{filter_value}")),
                    "<" => Ok(format!("{prefix}.{property}:<{filter_value}")),
                    "<=" => Ok(format!("{prefix}.{property}:<={filter_value}")),
                    ">" => Ok(format!("{prefix}.{property}:>{filter_value}")),
                    ">=" => Ok(format!("{prefix}.{property}:>={filter_value}")),
                    _ => Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unsupported operator for organization property: {operator}"),
                    )),
                }
            }
            _ => {
                // Default mapping
                let openalex_property = property;
                let filter_value = format_filter_value(&value);

                match operator {
                    "==" => Ok(format!("{prefix}.{openalex_property}:{filter_value}")),
                    "!=" => Ok(format!("{prefix}.{openalex_property}:!{filter_value}")),
                    "<" => Ok(format!("{prefix}.{openalex_property}:<{filter_value}")),
                    "<=" => Ok(format!("{prefix}.{openalex_property}:<={filter_value}")),
                    ">" => Ok(format!("{prefix}.{openalex_property}:>{filter_value}")),
                    ">=" => Ok(format!("{prefix}.{openalex_property}:>={filter_value}")),
                    _ => Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!("Unsupported operator for subquery: {operator}"),
                    )),
                }
            }
        }
    }

    /// Helper method to build author name filters using raw_author_name.search
    fn build_author_name_filter(&self, operator: &str, value: &Value) -> Result<String, Error> {
        let filter_value = format_filter_value(&value);

        match operator {
            "==" => Ok(format!("raw_author_name.search:{filter_value}")),
            "^=" => Ok(format!("raw_author_name.search:{filter_value}*")),
            "$=" => Ok(format!("raw_author_name.search:*{filter_value}")),
            _ => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported operator for author name: {operator}"),
            )),
        }
    }

    /// Helper method to build organization name filters using raw_affiliation_strings.search
    fn build_organization_name_filter(
        &self,
        operator: &str,
        value: &Value,
    ) -> Result<String, Error> {
        let filter_value = format_filter_value(&value);

        match operator {
            "==" => Ok(format!("raw_affiliation_strings.search:{filter_value}")),
            "^=" => Ok(format!("raw_affiliation_strings.search:{filter_value}*")),
            "$=" => Ok(format!("raw_affiliation_strings.search:*{filter_value}")),
            _ => Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Unsupported operator for organization name: {operator}"),
            )),
        }
    }

    /// Extract OpenAlex work IDs from query objects
    ///
    /// Executes OpenAlex and workspace queries to extract work IDs for use in
    /// cited_by filters. Supports up to 100 IDs per OpenAlex API limitations.
    fn extract_work_ids_from_query_objects(
        &self,
        query_objects: &[Value],
    ) -> Result<Vec<String>, Error> {
        let mut work_ids = Vec::new();

        for query_obj in query_objects {
            // Handle OpenAlex query objects
            if let Some(openalex_query) = query_obj.downcast_object_ref::<OpenAlexQuery>() {
                // Execute the query and extract work IDs from the results
                let nodes = openalex_query.nodes();
                for node in nodes {
                    if let Some(work_id) = self.extract_work_id_from_node(&node) {
                        work_ids.push(work_id);
                        // Limit to 100 IDs due to OpenAlex API restrictions
                        if work_ids.len() >= 100 {
                            break;
                        }
                    }
                }
            }

            // Handle workspace query objects (future implementation)
            if let Some(_workspace_query) = query_obj.downcast_object_ref::<crate::cypher::Query>()
            {
                // TODO: Implement workspace query ID extraction
                // This would involve executing the workspace query and extracting OpenAlex IDs
                // from the resulting documents
                tracing::warn!("Workspace query ID extraction not yet implemented");
            }

            // Break early if we hit the limit
            if work_ids.len() >= 100 {
                break;
            }
        }

        Ok(work_ids)
    }

    /// Extract OpenAlex work ID from a Node
    ///
    /// Looks for OpenAlex ID in various possible fields and formats
    fn extract_work_id_from_node(&self, node: &Node) -> Option<String> {
        // Convert Node to JSON Value for easier field access
        if let Ok(json_value) = serde_json::to_value(node) {
            if let Some(obj) = json_value.as_object() {
                // Try to get the 'id' field which should contain the full OpenAlex URL
                if let Some(id_value) = obj.get("id") {
                    if let Some(id_str) = id_value.as_str() {
                        // Extract work ID from OpenAlex URL format
                        if let Some(work_id) = id_str.strip_prefix("https://openalex.org/") {
                            return Some(work_id.to_string());
                        }
                        // Also handle direct work ID format
                        if id_str.starts_with("W") && id_str.len() > 1 {
                            return Some(id_str.to_string());
                        }
                    }
                }

                // Fallback: try other potential ID fields
                for field_name in ["openalex_id", "work_id", "doi"] {
                    if let Some(field_value) = obj.get(field_name) {
                        if let Some(field_str) = field_value.as_str() {
                            if field_str.starts_with("W") && field_str.len() > 1 {
                                return Some(field_str.to_string());
                            }
                            if let Some(work_id) = field_str.strip_prefix("https://openalex.org/") {
                                return Some(work_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract source IDs from query objects for publishedBy subqueries
    fn extract_source_ids_from_query_objects(
        &self,
        query_objects: &[Value],
    ) -> Result<Vec<String>, Error> {
        let mut source_ids = Vec::new();
        for query_obj in query_objects {
            // Handle OpenAlex query objects
            if let Some(openalex_query) = query_obj.downcast_object_ref::<OpenAlexQuery>() {
                // Execute the query and extract source IDs from the results
                let nodes = openalex_query.nodes();
                for node in nodes {
                    if let Some(source_id) = self.extract_source_id_from_node(&node) {
                        source_ids.push(source_id);
                        // Limit to 100 IDs due to OpenAlex API restrictions
                        if source_ids.len() >= 100 {
                            break;
                        }
                    }
                }
            }
            // Handle workspace query objects (future implementation)
            if let Some(_workspace_query) = query_obj.downcast_object_ref::<crate::cypher::Query>()
            {
                // TODO: Implement workspace query ID extraction for sources
                tracing::warn!("Workspace query ID extraction for sources not yet implemented");
            }
            // Break early if we hit the limit
            if source_ids.len() >= 100 {
                break;
            }
        }
        Ok(source_ids)
    }

    /// Extract a source ID from a Node
    fn extract_source_id_from_node(&self, node: &Node) -> Option<String> {
        // Convert Node to JSON Value for easier field access
        if let Ok(json_value) = serde_json::to_value(node) {
            if let Some(obj) = json_value.as_object() {
                // Try to get the 'id' field which should contain the full OpenAlex URL
                if let Some(id_value) = obj.get("id") {
                    if let Some(id_str) = id_value.as_str() {
                        // Extract source ID from OpenAlex URL format
                        if let Some(source_id) = id_str.strip_prefix("https://openalex.org/") {
                            return Some(source_id.to_string());
                        }
                        // Also handle direct source ID format
                        if id_str.starts_with("S") && id_str.len() > 1 {
                            return Some(id_str.to_string());
                        }
                    }
                }

                // Fallback: try other potential ID fields
                for field_name in ["openalex_id", "source_id"] {
                    if let Some(field_value) = obj.get(field_name) {
                        if let Some(field_str) = field_value.as_str() {
                            if field_str.starts_with("S") && field_str.len() > 1 {
                                return Some(field_str.to_string());
                            }
                            if let Some(source_id) = field_str.strip_prefix("https://openalex.org/")
                            {
                                return Some(source_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Convert count filters for OpenAlex compatibility
    ///
    /// The >= and <= operators don't work consistently and return 403 errors.
    /// We convert them to equivalent expressions:
    /// - ">=N" becomes ">N-1" (e.g., ">=10" becomes ">9")
    /// - "<=N" becomes "<N+1" (e.g., "<=5" becomes "<6")
    fn convert_count_filter_for_openalex(&self, filter: &str) -> Result<String, Error> {
        if let Some(number_str) = filter.strip_prefix(">=") {
            // Convert ">=N" to ">N-1"
            if let Ok(number) = number_str.parse::<i64>() {
                if number > 0 {
                    Ok(format!(">{}", number - 1))
                } else {
                    // >=0 means all positive numbers, equivalent to ">-1" but use ">=0"
                    Ok(">=0".to_string())
                }
            } else {
                Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Invalid number in count filter: {filter}"),
                ))
            }
        } else if let Some(number_str) = filter.strip_prefix("<=") {
            // Convert "<=N" to "<N+1"
            if let Ok(number) = number_str.parse::<i64>() {
                Ok(format!("<{}", number + 1))
            } else {
                Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Invalid number in count filter: {filter}"),
                ))
            }
        } else if let Some(stripped) = filter.strip_prefix("=") {
            // Handle equality - remove the leading = for OpenAlex format
            if stripped.parse::<i64>().is_ok() {
                Ok(stripped.to_string())
            } else {
                Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Invalid number in count filter: {filter}"),
                ))
            }
        } else {
            // Keep other operators as-is (>, <)
            Ok(filter.to_string())
        }
    }

    /// Generate the OpenAlex API URL (for backwards compatibility and debugging)
    pub fn generate(&self) -> String {
        let mut params = Vec::new();

        // Add filters
        if !self.filters.is_empty() {
            let filter_string = self.filters.join(",");
            params.push(("filter", filter_string));
        }

        // Add search
        if !self.search_terms.is_empty() {
            let search_string = self.search_terms.join(" ");
            params.push(("search", search_string));
        }

        // Add sort
        if let Some(sort) = &self.sort {
            params.push(("sort", sort.clone()));
        }

        // Add pagination
        if let Some(page) = self.page {
            params.push(("page", page.to_string()));
        }
        if let Some(per_page) = self.per_page {
            params.push(("per-page", per_page.to_string()));
        }

        // Add cursor pagination if enabled
        if self.use_cursor {
            if let Some(cursor) = &self.cursor {
                params.push(("cursor", cursor.clone()));
            }
        }

        // Add field selection
        if !self.select_fields.is_empty() {
            let select_string = self.select_fields.join(",");
            params.push(("select", select_string));
        }

        url_for_list(&self.entity_type, params)
    }

    /// Execute the query and return the resulting [`Node`]s
    #[tracing::instrument(skip(self))]
    pub fn nodes(&self) -> Vec<Node> {
        let url = self.generate();
        let entity_type = self.entity_type.as_str();

        let result: Result<_> = task::block_in_place(|| {
            runtime::Handle::current().block_on(async {
                if !self.select_fields.is_empty() {
                    let response = request::<SelectResponse>(&url).await?;
                    let datatable = Datatable::from(response.results);
                    return Ok((response.meta, vec![Node::Datatable(datatable)]));
                }

                Ok(match entity_type {
                    "works" => {
                        let response = request::<WorksResponse>(&url).await?;
                        let nodes: Vec<Node> =
                            response.results.into_iter().map(Into::into).collect();
                        (response.meta, nodes)
                    }
                    "authors" => {
                        let response = request::<AuthorsResponse>(&url).await?;
                        let nodes: Vec<Node> =
                            response.results.into_iter().map(Into::into).collect();
                        (response.meta, nodes)
                    }
                    "institutions" => {
                        let response = request::<InstitutionsResponse>(&url).await?;
                        let nodes: Vec<Node> =
                            response.results.into_iter().map(Into::into).collect();
                        (response.meta, nodes)
                    }
                    "sources" => {
                        let response = request::<SourcesResponse>(&url).await?;
                        let nodes: Vec<Node> =
                            response.results.into_iter().map(Into::into).collect();
                        (response.meta, nodes)
                    }
                    "publishers" => {
                        let response = request::<PublishersResponse>(&url).await?;
                        let nodes: Vec<Node> =
                            response.results.into_iter().map(Into::into).collect();
                        (response.meta, nodes)
                    }
                    "funders" => {
                        let response = request::<FundersResponse>(&url).await?;
                        let nodes: Vec<Node> =
                            response.results.into_iter().map(Into::into).collect();
                        (response.meta, nodes)
                    }
                    _ => {
                        bail!("Fetching of OpenAlex `{entity_type}` objects not yet enabled")
                    }
                })
            })
        });

        match result {
            Ok((meta, nodes)) => {
                if self.per_page == Some(0) {
                    if let Some(meta) = meta {
                        if let Some(count) = meta.count {
                            return vec![Node::Integer(count)];
                        }
                    }
                    return Vec::new();
                }
                nodes
            }
            Err(error) => {
                self.add_error_message(format!("OpenAlex API request failed: {error}"));
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
            | "reviews" | "standards" | "grants" | "retractions" | "datasets" | "authors"
            | "people" | "institutions" | "organizations" | "sources" | "journals"
            | "publishers" | "funders" => {
                let (entity_type, type_filter) = match name {
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

                    "authors" | "people" => ("authors", None),
                    "institutions" | "organizations" => ("institutions", None),

                    "sources" | "journals" => ("sources", None),
                    "publishers" => ("publishers", None),
                    "funders" => ("funders", None),

                    _ => {
                        return Err(Error::new(
                            ErrorKind::UnknownMethod,
                            format!("Unknown method: {name}"),
                        ));
                    }
                };

                let mut query = self.entity(entity_type);

                if let Some(value) = type_filter {
                    query.filters.push(["type:", value].concat());
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
                            return Err(Error::new(
                                ErrorKind::UnknownMethod,
                                "semantic similarity filtering is not available for OpenAlex, use `search` instead",
                            ));
                        }
                        // Handle transformed DocsQL filters
                        _ => query = query.filter(entity_type, arg, value)?,
                    }
                }

                query
            }

            // Query methods
            "orderBy" | "order_by" => {
                let (property, direction): (String, Option<String>) = from_args(args)?;
                self.order_by(&property, direction)?
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

/// Format a filter value for OpenAlex API
fn format_filter_value(value: &Value) -> String {
    match value.kind() {
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
        _ => value.to_string(),
    }
}
