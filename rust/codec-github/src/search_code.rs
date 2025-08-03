use codec::schema::{Node, SoftwareSourceCode, SoftwareSourceCodeOptions, StringOrNumber};
use serde::Deserialize;

/// Code search result item from GitHub search API
#[derive(Deserialize)]
pub struct CodeSearchItem {
    /// File name
    pub name: String,
    /// File path within the repository
    pub path: String,
    /// SHA hash of the file
    pub sha: String,
    /// API URL for the file
    pub url: String,
    /// Git URL for the file
    pub git_url: String,
    /// HTML URL for viewing the file on GitHub
    pub html_url: String,
    /// Repository containing the file
    pub repository: MinimalRepository,
    /// Search relevance score
    pub score: f64,
    /// File size in bytes
    pub file_size: Option<i64>,
    /// Programming language of the file
    pub language: Option<String>,
    /// Last modified timestamp
    pub last_modified_at: Option<String>,
    /// Line numbers where matches were found
    pub line_numbers: Option<Vec<String>>,
    /// Text match highlighting information
    pub text_matches: Option<Vec<TextMatch>>,
}

/// Minimal repository information in search results
#[derive(Deserialize)]
pub struct MinimalRepository {
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
    /// Number of stargazers
    pub stargazers_count: Option<i64>,
    /// Number of watchers
    pub watchers_count: Option<i64>,
    /// Repository size in KB
    pub size: Option<i64>,
    /// Default branch name
    pub default_branch: Option<String>,
    /// Number of open issues
    pub open_issues_count: Option<i64>,
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
    /// Number of forks (alternative field)
    pub forks: Option<i64>,
    /// Number of open issues (alternative field)
    pub open_issues: Option<i64>,
    /// Number of watchers (alternative field)
    pub watchers: Option<i64>,
    /// Whether forking is allowed
    pub allow_forking: Option<bool>,
    /// Whether web commit signoff is required
    pub web_commit_signoff_required: Option<bool>,
}

/// Simple user information in GitHub API responses
#[derive(Deserialize)]
pub struct SimpleUser {
    /// User's display name
    pub name: Option<String>,
    /// User's email address
    pub email: Option<String>,
    /// Username/login
    pub login: String,
    /// User ID
    pub id: i64,
    /// Node ID for GraphQL API
    pub node_id: String,
    /// Avatar image URL
    pub avatar_url: String,
    /// Gravatar ID
    pub gravatar_id: Option<String>,
    /// API URL for the user
    pub url: String,
    /// HTML URL for the user's profile
    pub html_url: String,
    /// Followers URL
    pub followers_url: String,
    /// Following URL template
    pub following_url: String,
    /// Gists URL template
    pub gists_url: String,
    /// Starred URL template
    pub starred_url: String,
    /// Subscriptions URL
    pub subscriptions_url: String,
    /// Organizations URL
    pub organizations_url: String,
    /// Repositories URL
    pub repos_url: String,
    /// Events URL template
    pub events_url: String,
    /// Received events URL
    pub received_events_url: String,
    /// User type
    pub r#type: String,
    /// Whether user is a site admin
    pub site_admin: bool,
    /// When the user starred something
    pub starred_at: Option<String>,
    /// User view type
    pub user_view_type: Option<String>,
}

/// Text match highlighting information
#[derive(Deserialize)]
pub struct TextMatch {
    /// Object URL
    pub object_url: Option<String>,
    /// Object type
    pub object_type: Option<String>,
    /// Property name
    pub property: Option<String>,
    /// Text fragment
    pub fragment: Option<String>,
    /// Match details
    pub matches: Option<Vec<TextMatchItem>>,
}

/// Individual text match item
#[derive(Deserialize)]
pub struct TextMatchItem {
    /// Matched text
    pub text: Option<String>,
    /// Character indices of the match
    pub indices: Option<Vec<i64>>,
}

impl From<CodeSearchItem> for SoftwareSourceCode {
    fn from(code: CodeSearchItem) -> Self {
        use codec::schema::PropertyValueOrString;

        // Store the GitHub file URL in both id and identifiers for compatibility
        let github_url = code.html_url.clone();

        SoftwareSourceCode {
            // Use the GitHub URL as the id, consistent with, for example, the OpenAlex codec
            id: Some(github_url.clone()),
            name: code.name,
            programming_language: code.language.unwrap_or_default(),
            path: Some(code.path),
            repository: Some(code.repository.html_url),
            version: Some(StringOrNumber::String(code.sha)),
            options: Box::new(SoftwareSourceCodeOptions {
                // Store the API URL in url field for reference
                url: Some(code.url),
                // Also store the GitHub URL in identifiers array
                identifiers: Some(vec![PropertyValueOrString::String(github_url)]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<CodeSearchItem> for Node {
    fn from(code: CodeSearchItem) -> Self {
        Node::SoftwareSourceCode(code.into())
    }
}
