use codec::schema::{
    CreativeWorkVariantOrString, Node, SoftwareSourceCode, SoftwareSourceCodeOptions,
    StringOrNumber,
};
use serde::Deserialize;

use crate::search_code::{SimpleUser, TextMatch};

/// Repository search result item from GitHub search API
#[derive(Deserialize)]
pub struct RepositorySearchItem {
    /// Repository ID
    pub id: i64,
    /// Node ID for GraphQL API
    pub node_id: String,
    /// Repository name
    pub name: String,
    /// Full name (owner/repo)
    pub full_name: String,
    /// Repository owner
    pub owner: SimpleUser,
    /// Whether the repository is private
    pub private: bool,
    /// HTML URL for the repository
    pub html_url: String,
    /// Repository description
    pub description: Option<String>,
    /// Whether this is a fork
    pub fork: bool,
    /// API URL for the repository
    pub url: String,
    /// Archive URL template
    pub archive_url: String,
    /// Assignees URL template
    pub assignees_url: String,
    /// Blobs URL template
    pub blobs_url: String,
    /// Branches URL template
    pub branches_url: String,
    /// Collaborators URL template
    pub collaborators_url: String,
    /// Comments URL template
    pub comments_url: String,
    /// Commits URL template
    pub commits_url: String,
    /// Compare URL template
    pub compare_url: String,
    /// Contents URL template
    pub contents_url: String,
    /// Contributors URL
    pub contributors_url: String,
    /// Deployments URL
    pub deployments_url: String,
    /// Downloads URL
    pub downloads_url: String,
    /// Events URL
    pub events_url: String,
    /// Forks URL
    pub forks_url: String,
    /// Git commits URL template
    pub git_commits_url: String,
    /// Git refs URL template
    pub git_refs_url: String,
    /// Git tags URL template
    pub git_tags_url: String,
    /// Git URL for cloning
    pub git_url: Option<String>,
    /// Issue comment URL template
    pub issue_comment_url: String,
    /// Issue events URL template
    pub issue_events_url: String,
    /// Issues URL template
    pub issues_url: String,
    /// Keys URL template
    pub keys_url: String,
    /// Labels URL template
    pub labels_url: String,
    /// Languages URL
    pub languages_url: String,
    /// Merges URL
    pub merges_url: String,
    /// Milestones URL template
    pub milestones_url: String,
    /// Notifications URL template
    pub notifications_url: String,
    /// Pull requests URL template
    pub pulls_url: String,
    /// Releases URL template
    pub releases_url: String,
    /// SSH URL for cloning
    pub ssh_url: Option<String>,
    /// Stargazers URL
    pub stargazers_url: String,
    /// Statuses URL template
    pub statuses_url: String,
    /// Subscribers URL
    pub subscribers_url: String,
    /// Subscription URL
    pub subscription_url: String,
    /// Tags URL
    pub tags_url: String,
    /// Teams URL
    pub teams_url: String,
    /// Trees URL template
    pub trees_url: String,
    /// Clone URL
    pub clone_url: Option<String>,
    /// Mirror URL
    pub mirror_url: Option<String>,
    /// Hooks URL
    pub hooks_url: String,
    /// SVN URL
    pub svn_url: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Primary language
    pub language: Option<String>,
    /// Number of forks
    pub forks_count: Option<i64>,
    /// Alternative field for forks
    pub forks: Option<i64>,
    /// Number of stargazers
    pub stargazers_count: Option<i64>,
    /// Number of watchers
    pub watchers_count: Option<i64>,
    /// Alternative field for watchers
    pub watchers: Option<i64>,
    /// Repository size in KB
    pub size: Option<i64>,
    /// Default branch name
    pub default_branch: Option<String>,
    /// Number of open issues
    pub open_issues_count: Option<i64>,
    /// Alternative field for open issues
    pub open_issues: Option<i64>,
    /// Whether this is a template repository
    pub is_template: Option<bool>,
    /// Repository topics
    pub topics: Option<Vec<String>>,
    /// Whether issues are enabled
    pub has_issues: Option<bool>,
    /// Whether projects are enabled
    pub has_projects: Option<bool>,
    /// Whether wiki is enabled
    pub has_wiki: Option<bool>,
    /// Whether pages are enabled
    pub has_pages: Option<bool>,
    /// Whether downloads are enabled
    pub has_downloads: Option<bool>,
    /// Whether discussions are enabled
    pub has_discussions: Option<bool>,
    /// Whether the repository is archived
    pub archived: Option<bool>,
    /// Whether the repository is disabled
    pub disabled: Option<bool>,
    /// Repository visibility
    pub visibility: Option<String>,
    /// Last push timestamp
    pub pushed_at: Option<String>,
    /// Created timestamp
    pub created_at: Option<String>,
    /// Last updated timestamp
    pub updated_at: Option<String>,
    /// Permissions
    pub permissions: Option<RepositoryPermissions>,
    /// Whether forking is allowed
    pub allow_forking: Option<bool>,
    /// Whether web commit signoff is required
    pub web_commit_signoff_required: Option<bool>,
    /// License information
    pub license: Option<RepositoryLicense>,
    /// Search relevance score
    pub score: f64,
    /// Text match highlighting information
    pub text_matches: Option<Vec<TextMatch>>,
}

/// Repository permissions in search results
#[derive(Deserialize)]
pub struct RepositoryPermissions {
    /// Admin permission
    pub admin: Option<bool>,
    /// Push permission
    pub push: Option<bool>,
    /// Pull permission
    pub pull: Option<bool>,
}

/// License information in repository search results
#[derive(Deserialize)]
pub struct RepositoryLicense {
    /// License key (e.g., "mit")
    pub key: Option<String>,
    /// License name (e.g., "MIT License")
    pub name: Option<String>,
    /// License URL
    pub url: Option<String>,
    /// SPDX ID
    pub spdx_id: Option<String>,
    /// Node ID
    pub node_id: Option<String>,
}

impl From<RepositorySearchItem> for SoftwareSourceCode {
    fn from(repo: RepositorySearchItem) -> Self {
        // Map version to default branch
        let version = repo.default_branch.map(StringOrNumber::String);

        // Map license information
        let licenses = repo.license.and_then(|license| {
            license
                .spdx_id
                .map(|spdx_id| vec![CreativeWorkVariantOrString::String(spdx_id)])
        });

        // Map topics as keywords
        let keywords = repo.topics.filter(|topics| !topics.is_empty());

        SoftwareSourceCode {
            name: repo.name,
            repository: Some(repo.html_url),
            programming_language: repo.language.unwrap_or_default(),
            version,
            options: Box::new(SoftwareSourceCodeOptions {
                description: repo.description,
                url: Some(repo.url),
                licenses,
                keywords,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<RepositorySearchItem> for Node {
    fn from(repo: RepositorySearchItem) -> Self {
        Node::SoftwareSourceCode(repo.into())
    }
}
