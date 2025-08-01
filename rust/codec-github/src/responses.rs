use serde::Deserialize;

use crate::search_code::CodeSearchItem;

/// The response from GitHub search API endpoints
///
/// See https://docs.github.com/en/rest/search/search
#[derive(Deserialize)]
pub struct SearchResponse<T> {
    /// Total number of search results
    pub total_count: i64,

    /// Whether the search results are incomplete
    pub incomplete_results: bool,

    /// Array of search result items
    pub items: Vec<T>,
}

/// Response for search code API calls
pub type SearchCodeResponse = SearchResponse<CodeSearchItem>;
