use std::sync::{Arc, Mutex as SyncMutex};

use codec_openalex::{
    AuthorsResponse, FundersResponse, InstitutionsResponse, PublishersResponse, SelectResponse,
    SourcesResponse, WorksResponse, request, request_ids, url_for_list,
};
use kernel_jinja::{
    kernel::{
        common::{
            eyre::{Result, bail},
            itertools::Itertools,
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
    cypher::{NodeProxies, NodeProxy},
    docsql::{decode_filter, encode_filter},
    subquery::Subquery,
    testing,
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
    search: Option<String>,

    /// Sort parameter (e.g., "cited_by_count:desc")
    sort: Option<String>,

    /// Pagination parameters
    page: Option<u32>,
    per_page: Option<u32>,

    // Sample count
    sample: Option<u32>,

    /// Fields to select in response
    select: Vec<String>,

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
            search: None,
            sort: None,
            page: None,
            per_page: None,
            sample: None,
            select: Vec::new(),
            use_cursor: false,
            cursor: None,
        }
    }

    /// Create a new OpenAlex query for an entity type
    pub fn clone_for(&self, entity_type: &str) -> Self {
        Self {
            entity_type: entity_type.into(),
            messages: self.messages.clone(),
            is_database: false,
            filters: Vec::new(),
            search: None,
            sort: None,
            page: None,
            per_page: None,
            sample: None,
            select: Vec::new(),
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
    fn filter(&mut self, arg_name: &str, arg_value: Value) -> Result<(), Error> {
        // Handle subquery filters (e.g., ...authors(.name ^= "Smith"))
        if arg_name == "_" {
            if let Some(subquery) = arg_value.downcast_object_ref::<Subquery>() {
                return self.subquery_filters(subquery);
            }
        }

        // Extract the property name an operator from the arg
        let (property_name, mut operator) = decode_filter(arg_name);

        // Error early for unhandled operators with advice
        if operator == "~!" || operator == "^=" || operator == "$=" || operator == "has" {
            let message = match operator {
                "~!" => "Negated search operator ~! is not supported for OpenAlex queries.",
                "^=" => {
                    "Starts-with operator ^= is not supported for OpenAlex queries. Perhaps use search operator ~= instead."
                }
                "$=" => {
                    "Ends-with operator $= is not supported for OpenAlex queries. Perhaps use search operator ~= instead."
                }
                "has" => "The `has` operator is not supported for OpenAlex queries",
                _ => "Unsupported operator",
            };
            return Err(Error::new(ErrorKind::InvalidOperation, message));
        }

        // Map the property name to the OpenAlex filter name
        let filter_name = match self.entity_type.as_str() {
            "works" => match property_name {
                // See https://docs.openalex.org/api-entities/works/filter-works
                // In OpenAlex it is not possible to test equality for `display_name`, only `display_name.search`
                // it available, which is also aliased to `title.search`
                "title" => "title.search",
                "name" => "display_name.search",
                // Abstract is available to search https://docs.openalex.org/api-entities/works/filter-works#abstract.search
                "abstract" => "abstract.search",
                // Properties on `primary_location` that we hoist up
                "license" => "primary_location.license",
                "is_accepted" => "primary_location.is_accepted",
                "is_published" => "primary_location.is_published",
                "version" => "primary_location.version",
                // Aliases
                "year" => "publication_year",
                "date" => "publication_date",
                "references_count" | "cites_count" | "referenced_works_count" => {
                    "referenced_works_count"
                }
                "institutions_count" | "organizations_count" | "institutions_distinct_count" => {
                    "institutions_distinct_count"
                }
                // Properties which do not need mapping, including convenience filters
                //  https://docs.openalex.org/api-entities/works/filter-works#works-convenience-filters
                "doi" | "is_oa" | "oa_status" | "has_abstract" | "has_references" | "has_doi"
                | "has_orcid" | "has_pmcid" | "has_pmid" | "authors_count" | "cited_by_count" => {
                    property_name
                }
                // Compound properties used by subqueries
                "authorships.author.orcid"
                | "authorships.institutions.ror"
                | "authorships.institutions.type"
                | "authorships.institutions.is_global_south"
                | "locations_count"
                | "primary_location.source.issn"
                | "raw_affiliation_strings.search"
                | "raw_author_name.search" => property_name,
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
                        "Unhandled filter property for OpenAlex {}: {property_name}",
                        self.entity_type
                    ),
                ));
            }
        };

        // Transform the minijinja argument value into a string
        let mut filter_value = format_filter_value(&arg_value);

        // Further entity_type and property transformations for user convenience
        if self.entity_type == "works"
            && property_name == "version"
            && !filter_value.ends_with("Version")
        {
            // published => publishedVersion etc
            filter_value.push_str("Version");
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

            "in" => format!("{filter_name}:{filter_value}"),

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

    /// Add filters specified in a subquery
    fn subquery_filters(&mut self, subquery: &Subquery) -> Result<(), Error> {
        let entity_type = self.entity_type.clone();
        let subquery_name = subquery.name.as_str();

        let unsupported_property = |property: &str| {
            Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "Filter `{property}` in subquery `{subquery_name}` is not supported for OpenAlex `{entity_type}`"
                ),
            ))
        };

        let ids_maybe = |query: OpenAlexQuery, test: &str, none: &str| {
            if query.filters.is_empty() {
                None
            } else if testing() {
                Some(test.into())
            } else {
                // If no ids returned then use a valid but
                // non-existent author id so that filter is still
                // applied but results in an empty set
                Some(query.ids().unwrap_or_else(|| none.into()))
            }
        };

        // How each subquery and its arguments are handled depends upon (a) the
        // entity type of the current query, (b) the name (entity type) of the
        // subquery, (c) the subquery filter property. If all subqueries filters
        // can be handled in a single request, by adding a filter to this query,
        // then that is done. The fallback is to create a new query and then
        // filter by id.
        match entity_type.as_str() {
            "works" => match subquery_name {
                "authors" => {
                    let mut ids_query = self.clone_for("authors");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, operator) = decode_filter(arg_name);
                        let property = match property {
                            "name" => "raw_author_name.search",
                            "orcid" => "authorships.author.orcid",
                            "_C" => "authors_count",
                            "has_orcid" | "h_index" | "i10_index" | "works_count"
                            | "cited_by_count" => {
                                ids_query.filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.filter(&encode_filter(property, operator), arg_value.clone())?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "A5100335963", "A0000000000") {
                        self.filters.push(format!("authorships.author.id:{ids}"));
                    }
                }
                "affiliations" => {
                    let mut ids_query = self.clone_for("institutions");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, operator) = decode_filter(arg_name);
                        let property = match property {
                            "name" => "raw_affiliation_strings.search",
                            "ror" => "authorships.institutions.ror",
                            "type" => "authorships.institutions.type",
                            "is_global_south" => "authorships.institutions.is_global_south",
                            "_C" => "institutions_distinct_count",
                            "has_ror" | "h_index" | "i10_index" | "works_count"
                            | "cited_by_count" => {
                                ids_query.filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.filter(&encode_filter(property, operator), arg_value.clone())?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "I1294671590", "I0000000000") {
                        self.filters
                            .push(format!("authorships.institutions.id:{ids}"));
                    }
                }
                "references" | "cites" | "cited_by" => {
                    let mut ids_query = self.clone_for("works");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, operator) = decode_filter(arg_name);
                        let property = match property {
                            // Only count convenience filters are available (other than `cites` and `cited_by` id filters)
                            "_C" => {
                                if subquery_name == "cited_by" {
                                    "cited_by_count"
                                } else {
                                    "referenced_works_count"
                                }
                            }
                            // Remaining filter attributes on works (see above) require an id query
                            "title"
                            | "name"
                            | "abstract"
                            | "license"
                            | "is_accepted"
                            | "is_published"
                            | "version"
                            | "year"
                            | "date"
                            | "references_count"
                            | "cites_count"
                            | "institutions_count"
                            | "organizations_count"
                            | "doi"
                            | "is_oa"
                            | "oa_status"
                            | "has_abstract"
                            | "has_references"
                            | "has_doi"
                            | "has_orcid"
                            | "has_pmcid"
                            | "has_pmid"
                            | "authors_count"
                            | "cited_by_count" => {
                                ids_query.filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.filter(&encode_filter(property, operator), arg_value.clone())?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "W2582743722", "W0000000000") {
                        self.filters.push(if subquery_name == "cited_by" {
                            format!("cited_by:{ids}")
                        } else {
                            format!("cites:{ids}")
                        });
                    }
                }
                "published_in" => {
                    let mut ids_query = self.clone_for("sources");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, operator) = decode_filter(arg_name);
                        let property = match property {
                            "issn" => "primary_location.source.issn",
                            "_C" => "locations_count",
                            // Remaining filter attributes on sources (see above) require an id query
                            "name" | "h_index" | "i10_index" | "has_issn" | "works_count"
                            | "cited_by_count" | "is_global_south" => {
                                ids_query.filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.filter(&encode_filter(property, operator), arg_value.clone())?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "S1336409049", "S0000000000") {
                        self.filters
                            .push(format!("primary_location.source.id:{ids}"));
                    }
                }
                "published_by" => {
                    let mut ids_query = self.clone_for("publishers");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, ..) = decode_filter(arg_name);
                        match property {
                            "_C" => return unsupported_property("count (*)"),
                            // All filter attributes on publishers (see above) require an id query
                            "name" | "h_index" | "i10_index" | "ror" | "works_count"
                            | "cited_by_count" => {
                                ids_query.filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                    }

                    if let Some(ids) = ids_maybe(ids_query, "P4310320595", "P0000000000") {
                        self.filters
                            .push(format!("primary_location.source.host_organization:{ids}"));
                    }
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::InvalidOperation,
                        format!(
                            "Subquery `{subquery_name}` is not supported for OpenAlex `{entity_type}`"
                        ),
                    ));
                }
            },
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Subqueries are not supported for OpenAlex `{entity_type}`"),
                ));
            }
        }

        Ok(())
    }

    /// Add a search term
    fn search(&self, term: String) -> Self {
        let mut query = self.clone();
        query.search = Some(term);
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

    fn sample(&self, count: Option<u32>) -> Self {
        let mut query = self.clone();
        let count = count.unwrap_or(10);
        query.sample = Some(count);
        query.per_page = Some(count);
        query
    }

    /// Select specific fields
    fn select(&self, fields: &[Value]) -> Result<Self, Error> {
        let mut query = self.clone();

        for field in fields {
            if let Some(field_name) = field.as_str() {
                query.select.push(field_name.to_string());
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

    /// Generate the OpenAlex API URL (for backwards compatibility and debugging)
    pub fn generate(&self) -> String {
        let mut params = Vec::new();

        // Add filters
        if !self.filters.is_empty() {
            let filter_string = self.filters.join(",");
            params.push(("filter", filter_string));
        }

        // Add search
        if let Some(search) = &self.search {
            params.push(("search", search.clone()));
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

        // Add sample
        if let Some(sample) = self.sample {
            params.push(("sample", sample.to_string()));
        }

        // Add field selection
        if !self.select.is_empty() {
            let select_string = self.select.join(",");
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
                if !self.select.is_empty() {
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

    /// Get the ids of nodes matching the query
    ///
    /// This is used to construct pipe joined lists of ids. Up to 100 ids can be joined in that
    /// pay so this als set per-page to 100.
    ///
    /// See https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities/filter-entity-lists#addition-or
    #[tracing::instrument(skip(self))]
    pub fn ids(&self) -> Option<String> {
        let mut query = self.clone();
        query.select.push("id".into());
        query.per_page = Some(100);

        let url = query.generate();

        let result: Result<_> = task::block_in_place(|| {
            runtime::Handle::current().block_on(async { request_ids(&url).await })
        });

        match result {
            Ok(ids) => {
                if ids.is_empty() {
                    None
                } else {
                    Some(ids.join("|"))
                }
            }
            Err(error) => {
                self.add_error_message(format!(
                    "OpenAlex API request for entity ids failed: {error}"
                ));
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

                // Add filter for the type of work
                if let Some(value) = type_filter {
                    query.filters.push(["type:", value].concat());
                }

                // Handle `search` and `like` arguments and apply all others as filters
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
                                "Semantic similarity filtering is not available for OpenAlex, use `search` instead",
                            ));
                        }
                        _ => query.filter(arg, value)?,
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
            "sample" => {
                let (count,): (Option<u32>,) = from_args(args)?;
                self.sample(count)
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
        ValueKind::Seq => {
            // Assumes that the list of values is being used as a list of OR alternatives
            // https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities/filter-entity-lists#addition-or
            match value.try_iter() {
                Ok(iter) => iter.map(|item| format_filter_value(&item)).join("|"),
                Err(_) => value.to_string(),
            }
        }
        _ => value.to_string(),
    }
}
