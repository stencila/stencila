use std::sync::{Arc, Mutex as SyncMutex};

use codec_openalex::{
    AuthorsResponse, FundersResponse, InstitutionsResponse, PublishersResponse, SelectResponse,
    SourcesResponse, WorksResponse, list_url, request, request_ids,
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
    docsql::{Operator, PropertyType, decode_filter, encode_filter},
    extend_messages,
    nodes::{NodeProxy, all, first, get, last},
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
    /// Execution messages to be added to when executing the query
    messages: Arc<SyncMutex<Vec<ExecutionMessage>>>,

    /// The OpenAlex entity type (works, authors, institutions, etc.)
    entity_type: String,

    /// Filter parameters
    filters: Vec<String>,

    /// Search term
    search: Option<String>,

    /// Sort parameter (e.g., "cited_by_count:desc")
    sort: Option<String>,

    /// Number of result items to skip
    skip: Option<usize>,

    /// Number of items to limit result to
    limit: Option<usize>,

    // Sample count
    sample: Option<usize>,

    /// Fields to select in response
    select: Vec<String>,
}

impl OpenAlexQuery {
    /// Create a new OpenAlex query
    pub fn new(messages: Arc<SyncMutex<Vec<ExecutionMessage>>>) -> Self {
        Self {
            messages,
            entity_type: String::new(),
            filters: Vec::new(),
            search: None,
            sort: None,
            skip: None,
            limit: None,
            sample: None,
            select: Vec::new(),
        }
    }

    /// Create a new OpenAlex query for an entity type
    pub fn clone_for(&self, entity_type: &str) -> Self {
        Self {
            messages: self.messages.clone(),
            entity_type: entity_type.into(),
            filters: Vec::new(),
            search: None,
            sort: None,
            skip: None,
            limit: None,
            sample: None,
            select: Vec::new(),
        }
    }

    /// Whether this is the base query for which no method has been called yet
    pub fn is_base(&self) -> bool {
        self.entity_type.is_empty()
    }

    /// Apply a filter to the query
    fn apply_filter(&mut self, arg_name: &str, arg_value: Value) -> Result<(), Error> {
        // Handle subquery filters (e.g., ...authors(.name ^= "Smith"))
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
        let (property_name, mut operator) = decode_filter(arg_name);

        // Error early for unhandled operators with advice
        let message = match operator {
            Operator::NoMatch => {
                Some("Negated search operator ~! is not supported for OpenAlex queries.")
            }
            Operator::Starts => Some(
                "Starts-with operator ^= is not supported for OpenAlex queries. Perhaps use search operator ~= instead.",
            ),
            Operator::Ends => Some(
                "Ends-with operator $= is not supported for OpenAlex queries. Perhaps use search operator ~= instead.",
            ),
            Operator::Has => Some("The `has` operator is not supported for OpenAlex queries"),
            _ => None,
        };
        if let Some(msg) = message {
            return Err(Error::new(ErrorKind::InvalidOperation, msg));
        }

        // Handle keywords specially as a combination of searching for the
        // keywords and then filtering for the mapped keywords
        if self.entity_type == "works" && property_name == "keywords" {
            if !matches!(operator, Operator::Eq | Operator::Match) {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Unsupported operator for OpenAlex keywords: {operator}. Use = or ==."),
                ));
            }

            let search = if let Some(value) = arg_value.as_str() {
                value.to_string()
            } else if let Ok(iter) = arg_value.try_iter() {
                iter.map(|item| format_filter_value(&item)).join(" ")
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    format!("Unsupported value for OpenAlex keywords: {arg_value}"),
                ));
            };

            let keyword_query = self.clone_for("keywords").search(&search);
            if let Some(ids) = if testing() {
                Some(vec!["keywords/diagnosis".to_string()])
            } else {
                keyword_query.ids()
            } {
                self.filters.push(format!("keywords.id:{}", ids.join("|")));
                return Ok(());
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidOperation,
                    "No matching keywords",
                ));
            }
        }

        let unsupported_property = || {
            Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "Unsupported filter property for OpenAlex {}: {property_name}",
                    self.entity_type
                ),
            ))
        };

        // Map the property name to the OpenAlex filter name
        let filter_name =
            match self.entity_type.as_str() {
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
                    "institutions_count"
                    | "organizations_count"
                    | "institutions_distinct_count" => "institutions_distinct_count",
                    // Properties which do not need mapping, including convenience filters
                    //  https://docs.openalex.org/api-entities/works/filter-works#works-convenience-filters
                    "doi" | "is_oa" | "oa_status" | "has_abstract" | "has_references"
                    | "has_doi" | "has_orcid" | "has_pmcid" | "has_pmid" | "authors_count"
                    | "cited_by_count" => property_name,
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
                    _ => return unsupported_property(),
                },
                "authors" => match property_name {
                    // See https://docs.openalex.org/api-entities/authors/filter-authors
                    "name" => "display_name.search",
                    // Properties on `summary_stats` that we hoist up
                    "impact_factor" => "summary_status.2yr_mean_citedness",
                    "h_index" => "summary_stats.h_index",
                    "i10_index" => "summary_stats.i10_index",
                    // Properties which do not need mapping, including convenience filters
                    //  https://docs.openalex.org/api-entities/authors/filter-authors#authors-convenience-filters
                    "orcid" | "has_orcid" | "works_count" | "cited_by_count" => property_name,
                    // Compound properties used by subqueries
                    "affiliations.institution.ror"
                    | "affiliations.institution.type"
                    | "last_known_institutions.is_global_south" => property_name,
                    // Error for all others
                    _ => return unsupported_property(),
                },
                "institutions" => match property_name {
                    // See https://docs.openalex.org/api-entities/institutions/filter-institutions
                    "name" => "display_name.search",
                    // Properties on `summary_stats` that we hoist up
                    "impact_factor" => "summary_status.2yr_mean_citedness",
                    "h_index" => "summary_stats.h_index",
                    "i10_index" => "summary_stats.i10_index",
                    // Properties which do not need mapping, including convenience filters
                    //  https://docs.openalex.org/api-entities/institutions/filter-institutions#institutions-convenience-filters
                    "ror" | "has_ror" | "works_count" | "cited_by_count" | "type"
                    | "is_global_south" => property_name,
                    // Error for all others
                    _ => return unsupported_property(),
                },
                "sources" => match property_name {
                    // See https://docs.openalex.org/api-entities/sources/filter-sources
                    "name" => "display_name.search",
                    // Properties on `summary_stats` that we hoist up
                    "impact_factor" => "summary_status.2yr_mean_citedness",
                    "h_index" => "summary_stats.h_index",
                    "i10_index" => "summary_stats.i10_index",
                    // Properties which do not need mapping, including convenience filters
                    //  https://docs.openalex.org/api-entities/sources/filter-sources#sources-convenience-filters
                    "issn" | "has_issn" | "is_oa" | "works_count" | "cited_by_count"
                    | "is_global_south" => property_name,
                    // Error for all others
                    _ => return unsupported_property(),
                },
                "publishers" => match property_name {
                    // See https://docs.openalex.org/api-entities/publishers/filter-publishers
                    "name" => "display_name.search",
                    // Properties on `summary_stats` that we hoist up
                    "impact_factor" => "summary_status.2yr_mean_citedness",
                    "h_index" => "summary_stats.h_index",
                    "i10_index" => "summary_stats.i10_index",
                    // Properties which do not need mapping, including convenience filters
                    //  https://docs.openalex.org/api-entities/publishers/filter-publishers#publishers-convenience-filters
                    "ror" | "works_count" | "cited_by_count" => property_name,
                    // Error for all others
                    _ => return unsupported_property(),
                },
                "funders" => match property_name {
                    // See https://docs.openalex.org/api-entities/funders/filter-funders
                    "name" => "display_name.search",
                    "description" => "description.search",
                    // Properties on `summary_stats` that we hoist up
                    "impact_factor" => "summary_status.2yr_mean_citedness",
                    "h_index" => "summary_stats.h_index",
                    "i10_index" => "summary_stats.i10_index",
                    // Properties which do not need mapping, including convenience filters
                    //  https://docs.openalex.org/api-entities/funders/filter-funders#funders-convenience-filters
                    "ror" | "grants_count" | "works_count" | "cited_by_count"
                    | "is_global_south" => property_name,
                    // Error for all others
                    _ => return unsupported_property(),
                },
                // Error for all others
                _ => return unsupported_property(),
            };

        // Check that operator is valid for property
        let property_type = if property_name.starts_with("is_") || property_name.starts_with("has_")
        {
            PropertyType::Boolean
        } else if property_name.ends_with("_count")
            || property_name.ends_with("_index")
            || matches!(property_name, "impact_factor" | "year")
        {
            PropertyType::Number
        } else if matches!(property_name, "date") {
            PropertyType::Date
        } else {
            PropertyType::String
        };
        if !property_type.is_valid(operator) {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "The `{operator}` operator can not be used with the OpenAlex `{property_name}` filter"
                ),
            ));
        }

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
            if operator == Operator::Lte {
                operator = Operator::Lt;
                filter_value = num.saturating_add(1).to_string()
            } else if operator == Operator::Gte {
                operator = Operator::Gt;
                filter_value = num.saturating_sub(1).to_string()
            }
        }

        // Generate the filter string
        let filter_string = match operator {
            Operator::Eq => format!("{filter_name}:{filter_value}"),
            Operator::Neq => format!("{filter_name}:!{filter_value}"),

            Operator::Lt => format!("{filter_name}:<{filter_value}"),
            Operator::Gt => format!("{filter_name}:>{filter_value}"),

            Operator::Match => {
                if filter_name.ends_with(".search") {
                    format!("{filter_name}:{filter_value}")
                } else {
                    format!("{filter_name}.search:{filter_value}")
                }
            }

            Operator::In => format!("{filter_name}:{filter_value}"),

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
        let entity_type = self.entity_type.clone();
        let subquery_name = subquery.name.as_str();

        let unsupported_subquery = || {
            Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("Subquery `{subquery_name}` is not supported for OpenAlex `{entity_type}`"),
            ))
        };

        let unsupported_property = |property: &str| {
            Err(Error::new(
                ErrorKind::InvalidOperation,
                format!(
                    "Filter `{property}` in subquery `{subquery_name}` is not supported for OpenAlex `{entity_type}`"
                ),
            ))
        };

        let ids_maybe = |query: OpenAlexQuery, entity_type: &str| {
            if query.filters.is_empty() && query.search.is_none() {
                None
            } else if testing() {
                Some(get_test_ids(entity_type).join("|"))
            } else {
                // If no ids returned then use a valid but
                // non-existent author id so that filter is still
                // applied but results in an empty set
                query
                    .ids()
                    .map(|ids| ids.join("|"))
                    .or_else(|| Some(get_default_id(entity_type)))
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
                            "search" | "name" => "raw_author_name.search",
                            "orcid" => "authorships.author.orcid",
                            "_C" => "authors_count",
                            // Remaining filter attributes on authors (see
                            // above), and nested subqueries, require an id
                            // query
                            "has_orcid" | "impact_factor" | "h_index" | "i10_index"
                            | "works_count" | "cited_by_count" | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.apply_filter(
                            &encode_filter(property, operator.as_str()),
                            arg_value.clone(),
                        )?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "authors") {
                        self.filters.push(format!("authorships.author:{ids}"));
                    }
                }
                "affiliations" => {
                    let mut ids_query = self.clone_for("institutions");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, operator) = decode_filter(arg_name);
                        let property = match property {
                            "search" | "name" => "raw_affiliation_strings.search",
                            "ror" => "authorships.institutions.ror",
                            "type" => "authorships.institutions.type",
                            "is_global_south" => "authorships.institutions.is_global_south",
                            "_C" => "institutions_distinct_count",
                            // Remaining filter attributes on institutions (see
                            // above), and nested subqueries, require an id
                            // query
                            "has_ror" | "impact_factor" | "h_index" | "i10_index"
                            | "works_count" | "cited_by_count" | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.apply_filter(
                            &encode_filter(property, operator.as_str()),
                            arg_value.clone(),
                        )?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "institutions") {
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
                            // Remaining filter attributes on works (see above),
                            // and nested subqueries, require an id query
                            "search"
                            | "keywords"
                            | "title"
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
                            | "cited_by_count"
                            | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.apply_filter(
                            &encode_filter(property, operator.as_str()),
                            arg_value.clone(),
                        )?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "works") {
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
                            // Remaining filter attributes on sources (see
                            // above), and nested subqueries, require an id
                            // query
                            "search" | "name" | "impact_factor" | "h_index" | "i10_index"
                            | "has_issn" | "works_count" | "cited_by_count" | "is_global_south"
                            | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.apply_filter(
                            &encode_filter(property, operator.as_str()),
                            arg_value.clone(),
                        )?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "sources") {
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
                            // All filter attributes on publishers (see above),
                            // and nested subqueries, require an id query
                            "search" | "name" | "impact_factor" | "h_index" | "i10_index"
                            | "ror" | "works_count" | "cited_by_count" | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                    }

                    if let Some(ids) = ids_maybe(ids_query, "publishers") {
                        self.filters
                            .push(format!("primary_location.source.host_organization:{ids}"));
                    }
                }
                "funded_by" => {
                    let mut ids_query = self.clone_for("funders");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, ..) = decode_filter(arg_name);
                        match property {
                            "_C" => return unsupported_property("count (*)"),
                            // All filter attributes on funders (see above), and
                            // nested subqueries, require an id query
                            "search" | "name" | "description" | "impact_factor" | "h_index"
                            | "i10_index" | "ror" | "grants_count" | "works_count"
                            | "cited_by_count" | "is_global_south" | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                    }

                    if let Some(ids) = ids_maybe(ids_query, "funders") {
                        self.filters.push(format!("grants.funder:{ids}"));
                    }
                }
                _ => return unsupported_subquery(),
            },
            "authors" => match subquery_name {
                "affiliations" => {
                    let mut ids_query = self.clone_for("institutions");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, operator) = decode_filter(arg_name);
                        let property = match property {
                            "ror" => "affiliations.institution.ror",
                            "type" => "affiliations.institution.type",
                            "is_global_south" => "last_known_institutions.is_global_south",
                            "_C" => return unsupported_property("count (*)"),
                            // Remaining filter attributes on institutions (see
                            // above), and nested subqueries, require an id
                            // query
                            "search" | "name" | "has_ror" | "impact_factor" | "h_index"
                            | "i10_index" | "works_count" | "cited_by_count" | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                        self.apply_filter(
                            &encode_filter(property, operator.as_str()),
                            arg_value.clone(),
                        )?;
                    }

                    if let Some(ids) = ids_maybe(ids_query, "institutions") {
                        self.filters
                            .push(format!("affiliations.institution.id:{ids}"));
                    }
                }
                _ => return unsupported_subquery(),
            },
            "sources" | "publishers" => match (entity_type.as_str(), subquery_name) {
                ("sources", "published_by") | ("publishers", "part_of") => {
                    let mut ids_query = self.clone_for("publishers");
                    for (arg_name, arg_value) in &subquery.args {
                        let (property, ..) = decode_filter(arg_name);
                        match property {
                            "_C" => return unsupported_property("count (*)"),
                            // All filter attributes on publishers (see above),
                            // and nested subqueries, require an id query
                            "search" | "name" | "impact_factor" | "h_index" | "i10_index"
                            | "ror" | "works_count" | "cited_by_count" | "_" => {
                                ids_query.apply_filter(arg_name, arg_value.clone())?;
                                continue;
                            }
                            _ => return unsupported_property(property),
                        };
                    }

                    if let Some(ids) = ids_maybe(ids_query, "publishers") {
                        self.filters.push(if entity_type == "sources" {
                            format!("host_organization_lineage:{ids}")
                        } else {
                            format!("lineage:{ids}")
                        });
                    }
                }
                _ => return unsupported_subquery(),
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

    /// Set the search term
    fn search(&self, term: &str) -> Self {
        let mut query = self.clone();
        query.search = Some(term.into());
        query
    }

    /// Set sort parameter
    fn sort(&self, property: &str, direction: Option<String>) -> Result<Self, Error> {
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
                "impact_factor" => "summary_status.2yr_mean_citedness",
                "h_index" => "summary_stats.h_index",
                "i10_index" => "summary_stats.i10_index",
                _ => property,
            },
        };

        let sort = match direction {
            Some(dir) if dir.to_uppercase() == "DESC" => format!("{attribute}:desc"),
            _ => format!("{attribute}:asc"),
        };
        query.sort = Some(sort);

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

    fn sample(&self, count: Option<usize>) -> Self {
        let mut query = self.clone();
        let count = count.unwrap_or(10);
        query.sample = Some(count);
        query.limit = Some(count);
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
        // Used in `nodes` to indicate that only count should be extracted
        query.limit = Some(0);
        query
    }

    /// Generate the OpenAlex API URL
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

        // Add pagination parameters based on skip and/or limit
        if let (Some(skip), Some(limit)) = (self.skip, self.limit) {
            let page = (skip / limit) + 1;
            params.extend([("per-page", limit.to_string()), ("page", page.to_string())]);
        } else if let Some(skip) = self.skip {
            params.extend([("per-page", skip.to_string()), ("page", "2".to_string())]);
        } else if let Some(limit) = self.limit {
            // Need to ensure >0
            params.push(("per-page", limit.max(1).to_string()));
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

        list_url(&self.entity_type, params)
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
                        let nodes = response.results.into_iter().map(Node::from).collect();
                        (response.meta, nodes)
                    }
                    "authors" => {
                        let response = request::<AuthorsResponse>(&url).await?;
                        let nodes = response.results.into_iter().map(Node::from).collect();
                        (response.meta, nodes)
                    }
                    "institutions" => {
                        let response = request::<InstitutionsResponse>(&url).await?;
                        let nodes = response.results.into_iter().map(Node::from).collect();
                        (response.meta, nodes)
                    }
                    "sources" => {
                        let response = request::<SourcesResponse>(&url).await?;
                        let nodes = response.results.into_iter().map(Node::from).collect();
                        (response.meta, nodes)
                    }
                    "publishers" => {
                        let response = request::<PublishersResponse>(&url).await?;
                        let nodes = response.results.into_iter().map(Node::from).collect();
                        (response.meta, nodes)
                    }
                    "funders" => {
                        let response = request::<FundersResponse>(&url).await?;
                        let nodes = response.results.into_iter().map(Node::from).collect();
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
                if self.limit == Some(0) {
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
    /// pay so this also sets limit to 100.
    ///
    /// See https://docs.openalex.org/how-to-use-the-api/get-lists-of-entities/filter-entity-lists#addition-or
    #[tracing::instrument(skip(self))]
    pub fn ids(&self) -> Option<Vec<String>> {
        let mut query = self.clone();
        query.select.push("id".into());
        query.limit = Some(100);

        let url = query.generate();

        let result: Result<_> = task::block_in_place(|| {
            runtime::Handle::current().block_on(async { request_ids(&url).await })
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

/// Get test IDs for a given entity type
fn get_test_ids(entity_type: &str) -> Vec<String> {
    match entity_type {
        "authors" => vec!["A5100335963", "A2289985273"],
        "funders" => vec!["F4320306076", "F4320306084"],
        "institutions" => vec!["I1294671590", "I97018004"],
        "publishers" => vec!["P4310320595", "P4310320609"],
        "sources" => vec!["S1336409049", "S124911201"],
        "works" => vec!["W2741809807", "W2360775259"],
        _ => vec![],
    }
    .into_iter()
    .map(String::from)
    .collect()
}

/// Get the default ID for a given entity type
fn get_default_id(entity_type: &str) -> String {
    [
        match entity_type {
            "authors" => "A",
            "funders" => "F",
            "institutions" => "I",
            "publishers" => "P",
            "sources" => "S",
            "works" => "W",
            _ => "X",
        },
        "0000000000",
    ]
    .concat()
}

impl Object for OpenAlexQuery {
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

        let query = match name {
            // Core API URL building methods
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

                let mut query = self.clone_for(entity_type);

                // Add filter for the type of work
                if let Some(value) = type_filter {
                    query.filters.push(["type:", value].concat());
                }

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
                                "Semantic similarity filtering is not available for OpenAlex, use `search` instead",
                            ));
                        }
                        _ => query.apply_filter(arg, value)?,
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
            "sample" => {
                let (count,): (Option<usize>,) = from_args(args)?;
                self.sample(count)
            }
            "select" => {
                let (fields,): (&[Value],) = from_args(args)?;
                self.select(fields)?
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
                format!("OpenAlex query does not have property `{property}`"),
            );
            None
        } else if let Some(index) = key.as_i64() {
            get(index, self.nodes(), &self.messages)
        } else {
            None
        }
    }
}
