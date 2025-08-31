use serde::Deserialize;

use codec::common::serde_json;

/// The response from Zenodo search API endpoints
#[derive(Debug, Deserialize)]
pub struct SearchResponse<T> {
    /// Total number of search results
    pub hits: Hits<T>,

    /// Aggregations (facets) for filtering
    #[serde(default)]
    pub aggregations: serde_json::Value,

    /// Links for pagination
    pub links: SearchLinks,
}

#[derive(Debug, Deserialize)]
pub struct Hits<T> {
    /// Total number of results
    pub total: i64,

    /// Array of search result items
    pub hits: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct SearchLinks {
    /// URL for the current page
    #[serde(rename = "self")]
    pub self_: String,

    /// URL for the next page
    #[serde(default)]
    pub next: Option<String>,

    /// URL for the previous page
    #[serde(default)]
    pub prev: Option<String>,
}

/// A Zenodo record (publication, dataset, software, etc.)
#[derive(Debug, Deserialize)]
pub struct Record {
    /// Record ID
    pub id: i64,

    /// DOI
    #[serde(default)]
    pub doi: Option<String>,

    /// Concept DOI (for all versions)
    #[serde(default)]
    pub conceptdoi: Option<String>,

    /// Record metadata
    pub metadata: RecordMetadata,

    /// Files associated with the record
    #[serde(default)]
    pub files: Vec<FileInfo>,

    /// Links
    pub links: RecordLinks,

    /// Creation date
    pub created: String,

    /// Last modified date
    #[serde(default)]
    pub modified: Option<String>,

    /// Revision number
    #[serde(default)]
    pub revision: Option<i64>,

    /// Owners
    #[serde(default)]
    pub owners: Option<Vec<Owner>>,

    /// Record stats
    #[serde(default)]
    pub stats: Option<RecordStats>,
}

#[derive(Debug, Deserialize)]
pub struct Owner {
    /// Owner ID
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordMetadata {
    /// Title
    pub title: String,

    /// DOI
    #[serde(default)]
    pub doi: Option<String>,

    /// Publication date
    pub publication_date: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Access right (open, embargoed, restricted, closed)
    pub access_right: String,

    /// Creators
    pub creators: Vec<Creator>,

    /// Keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Resource type
    pub resource_type: ResourceType,

    /// License
    #[serde(default)]
    pub license: Option<License>,

    /// Version
    #[serde(default)]
    pub version: Option<String>,

    /// Language
    #[serde(default)]
    pub language: Option<String>,

    /// Communities
    #[serde(default)]
    pub communities: Vec<Community>,

    /// Grants
    #[serde(default)]
    pub grants: Vec<Grant>,

    /// Related identifiers
    #[serde(default)]
    pub related_identifiers: Vec<RelatedIdentifier>,

    /// Contributors
    #[serde(default)]
    pub contributors: Vec<Contributor>,

    /// References
    #[serde(default)]
    pub references: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Creator {
    /// Name
    pub name: String,

    /// Affiliation
    #[serde(default)]
    pub affiliation: Option<String>,

    /// ORCID
    #[serde(default)]
    pub orcid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Contributor {
    /// Name
    pub name: String,

    /// Type (e.g., Editor, DataCollector, etc.)
    #[serde(rename = "type")]
    pub contributor_type: String,

    /// Affiliation
    #[serde(default)]
    pub affiliation: Option<String>,

    /// ORCID
    #[serde(default)]
    pub orcid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceType {
    /// Type identifier
    #[serde(rename = "type")]
    pub type_: String,

    /// Subtype identifier
    #[serde(default)]
    pub subtype: Option<String>,

    /// Human-readable title
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct License {
    /// License identifier (e.g., cc-by-4.0)
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Community {
    /// Community identifier
    pub id: String,

    /// Community title (optional)
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Grant {
    /// Grant ID
    #[serde(default)]
    pub id: Option<String>,

    /// Internal ID
    #[serde(default)]
    pub internal_id: Option<String>,

    /// Grant code
    #[serde(default)]
    pub code: Option<String>,

    /// Grant title
    #[serde(default)]
    pub title: Option<String>,

    /// Funder
    #[serde(default)]
    pub funder: Option<Funder>,
}

#[derive(Debug, Deserialize)]
pub struct Funder {
    /// DOI
    #[serde(default)]
    pub doi: Option<String>,

    /// Name
    #[serde(default)]
    pub name: Option<String>,

    /// Acronym
    #[serde(default)]
    pub acronym: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RelatedIdentifier {
    /// Identifier
    pub identifier: String,

    /// Relation type (e.g., isSupplementTo, isVersionOf)
    pub relation: String,

    /// Resource type
    #[serde(default)]
    pub resource_type: Option<String>,

    /// Scheme (e.g., doi, url, arxiv)
    #[serde(default)]
    pub scheme: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileInfo {
    /// File ID
    pub id: String,

    /// File key (filename)
    pub key: String,

    /// File size in bytes
    pub size: i64,

    /// MD5 checksum
    pub checksum: String,

    /// Links for the file
    pub links: FileLinks,
}

#[derive(Debug, Deserialize)]
pub struct FileLinks {
    /// Direct download URL
    #[serde(rename = "self")]
    pub self_: String,
}

#[derive(Debug, Deserialize)]
pub struct RecordLinks {
    /// URL for the record
    #[serde(rename = "self")]
    pub self_: String,

    /// HTML URL for viewing the record on Zenodo
    #[serde(rename = "self_html")]
    pub html: String,

    /// DOI URL
    #[serde(default)]
    pub doi: Option<String>,

    /// Badge URL
    #[serde(default)]
    pub badge: Option<String>,

    /// Files URL
    #[serde(default)]
    pub files: Option<String>,

    /// Latest version URL
    #[serde(default)]
    pub latest: Option<String>,

    /// Latest HTML URL
    #[serde(default)]
    pub latest_html: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordStats {
    /// Download count
    #[serde(default)]
    pub downloads: i64,

    /// View count
    #[serde(default)]
    pub views: i64,

    /// Unique download count
    #[serde(default)]
    pub unique_downloads: i64,

    /// Unique view count
    #[serde(default)]
    pub unique_views: i64,

    /// Version stats
    #[serde(default)]
    pub version_stats: Option<VersionStats>,
}

#[derive(Debug, Deserialize)]
pub struct VersionStats {
    /// Total downloads for all versions
    pub downloads: i64,

    /// Total views for all versions
    pub views: i64,
}

/// Response for search records API calls
pub type SearchRecordsResponse = SearchResponse<Record>;
