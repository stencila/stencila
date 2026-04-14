//! Derive GitHub pull request source provenance from document nodes.
//!
//! The values extracted here describe where the reviewed document originally
//! came from in a Git repository. They are used by both export and workflow
//! code to recover the repository, path, commit, and format context needed to
//! create repository-native pull requests.

use serde::Serialize;
use serde_with::skip_serializing_none;
use stencila_codec::{
    stencila_format::Format,
    stencila_schema::{Datatable, Node, SoftwareSourceCode},
};

/// Where the reviewed document originally came from.
///
/// Populated from `Article.repository`, `Article.path`, and `Article.commit`
/// (or equivalent fields on other work types). These are preserved through
/// DOCX round-trips and become the source of truth for pull request context.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestSource {
    pub repository: Option<String>,
    pub path: Option<String>,
    pub commit: Option<String>,
    pub format: Format,
}

pub(crate) fn pull_request_source(node: &Node, format: Format) -> PullRequestSource {
    match node {
        Node::Article(article) => PullRequestSource {
            repository: article.options.repository.clone(),
            path: article.options.path.clone(),
            commit: article.options.commit.clone(),
            format,
        },
        Node::Datatable(Datatable { options, .. }) => PullRequestSource {
            repository: options.repository.clone(),
            path: options.path.clone(),
            commit: options.commit.clone(),
            format,
        },
        Node::SoftwareSourceCode(SoftwareSourceCode {
            repository,
            path,
            commit,
            ..
        }) => PullRequestSource {
            repository: repository.clone(),
            path: path.clone(),
            commit: commit.clone(),
            format,
        },
        _ => PullRequestSource {
            repository: None,
            path: None,
            commit: None,
            format,
        },
    }
}
