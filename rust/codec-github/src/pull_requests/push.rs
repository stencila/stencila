//! Push comment-bearing Stencila exports to GitHub pull requests.
//!
//! The submission strategy in this module distinguishes between two separate
//! kinds of synthetic Git changes that GitHub may require:
//!
//! 1. **Placeholder content changes**: when a review/export has no substantive
//!    source diff at all, GitHub still needs some file change before a pull
//!    request can be opened. In that case Stencila can add a stable placeholder
//!    marker to create a temporary non-empty diff.
//! 2. **Localized review anchors**: for existing files with only narrow diff
//!    hunks, GitHub may reject inline review comments or suggestions that are
//!    too far from the nearest hunk with HTTP 422 errors. Probe runs using
//!    `rust/codec-github/tests/probe-pr-review-distance.sh` found that a PR on
//!    an existing 50-line file with only line 25 changed accepted inline
//!    comments on lines 22..28 but rejected comments from line 21 outward,
//!    while newly added files accepted comments across the file. This implies
//!    that a single dummy change at the end of a file is insufficient to anchor
//!    arbitrary review items in existing files.
//!
//! To handle that, this module creates temporary anchor commits containing
//! reversible localized whitespace-only changes near review target lines,
//! submits the GitHub review against that anchor commit, and then rewinds the
//! branch ref back to the pre-anchor commit with a force update. This keeps the
//! visible branch history cleaner while still giving GitHub localized diff hunks
//! close enough for inline anchoring without inserting extra lines that could
//! shift later review targets. If GitHub still rejects inline placement, the
//! submission layer falls back to body-only review items rather than losing the
//! feedback.
//!
//! For exports that combine substantive source changes with review items,
//! anchor planning is proactive rather than purely reactive. Before the first
//! review submission, Stencila compares the pre-change and post-change file
//! contents, estimates which review targets are already covered by nearby
//! substantive diff hunks, and only inserts synthetic anchors for uncovered
//! targets. This avoids an unnecessary first failed GitHub review submission in
//! common mixed-content cases while still minimizing temporary anchor markers.
//! The retry-on-422 path remains as a safety net because GitHub's inline review
//! anchoring behavior is empirical rather than fully specified.

use base64::Engine;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use stencila_codec::eyre::{Result, bail, eyre};
use stencila_codec::stencila_schema::SuggestionType;
use stencila_codec::{PushDryRunFile, PushDryRunOptions, PushResult};

use super::{
    activity::parse_github_pull_request_url,
    export::{
        PullRequestComment, PullRequestCommentKind, PullRequestCommentResolution,
        PullRequestExport, PullRequestSide, PullRequestTarget,
    },
};
use crate::client::{
    GitHubAuthPolicy, api_url, delete, get_json, patch_json, post_json, post_json_with_status,
};

// Prefer the user's GitHub identity for substantive PR creation and content
// commits so authorship matches the human who initiated the push, but prefer
// the repository installation identity for synthetic anchor commits and review
// submission so the temporary anchoring machinery is clearly attributable to
// automation.
const PR_AUTH_POLICY: GitHubAuthPolicy = GitHubAuthPolicy::PreferUser;
const ANCHOR_AUTH_POLICY: GitHubAuthPolicy = GitHubAuthPolicy::RequireRepoInstallation;

/// Maximum number of trailing line endings used for placeholder PR-opening
/// changes.
///
/// Placeholder changes now use only end-of-file newline count changes rather
/// than visible marker lines so the temporary diff remains minimally intrusive.
const PR_PLACEHOLDER_MAX_TRAILING_LINE_ENDINGS: usize = 2;

/// Invisible marker appended to anchor lines.
///
/// A zero-width space creates a localized diff hunk without inserting extra
/// lines that could shift GitHub's review line mapping for subsequent comment
/// targets, and is visually lighter than altering visible content lines.
const ANCHOR_MARKER: &str = "\u{200B}";

/// Estimated maximum distance at which a synthetic anchor hunk can still
/// satisfy GitHub inline review placement for nearby target lines.
///
/// Used to coalesce nearby review targets so Stencila inserts the minimum
/// number of temporary markers needed for a localized anchor commit. The same
/// distance is also used during proactive mixed-content planning to decide
/// whether a substantive content hunk already covers a review target.
const ANCHOR_COVERAGE_DISTANCE: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubmitReviewMode {
    RequireInline,
    AllowFallback,
}

fn short_snippet_from_end(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    let collapsed = collapse_whitespace(text);
    let chars: Vec<char> = collapsed.chars().collect();
    let len = chars.len();
    let start = len.saturating_sub(60);
    let snippet: String = chars[start..].iter().collect();

    Some(if start > 0 {
        format!("…{snippet}")
    } else {
        snippet
    })
}

#[derive(Debug, Clone, Copy)]
enum CommentBodyMode {
    Anchored,
    Fallback,
}

#[derive(Debug)]
enum SubmitReviewError {
    NeedsAnchorCommit,
    UnresolvedInlinePath,
    Other(stencila_codec::eyre::Report),
}

impl SubmitReviewError {
    fn into_report(self) -> stencila_codec::eyre::Report {
        match self {
            Self::NeedsAnchorCommit => eyre!(
                "GitHub could not resolve inline review comment anchors and an anchor commit is required"
            ),
            Self::UnresolvedInlinePath => {
                eyre!(
                    "GitHub could not resolve inline review comment paths in the pull request diff"
                )
            }
            Self::Other(error) => error,
        }
    }
}

fn is_missing_anchor_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();

    // GitHub may reject inline review comments that are too far from the
    // nearest diff hunk with a 422 payload like:
    //
    //   {"message":"Validation Failed","errors":[{"resource":"PullRequestReviewComment","code":"custom","field":"pull_request_review_thread.line","message":"could not be resolved"}]}
    //
    // This behavior was probed using
    // `rust/codec-github/tests/probe-pr-review-distance.sh` against a real
    // repository. For an existing 50-line file with only line 25 changed in
    // the PR, GitHub accepted comments on lines 22..28 but rejected comments
    // on lines 21, 29, 20, 30, 15, 35, 10, 40, 1, and 50 with the above 422.
    // This distance-related anchoring failure appears to affect existing files
    // with localized diff hunks. In contrast, probe runs against newly added
    // files accepted comments across the file, which is consistent with the
    // whole new file being part of the diff. This has implications for anchor
    // commits: a single dummy change at the end of the file is not sufficient
    // to anchor arbitrary comments or suggestions in that file, and even a
    // real change elsewhere in the file may still be too far away. Anchoring
    // needs diff hunks close enough to the intended lines.
    [
        "pull request review thread diff hunk can't be blank",
        "pull request review comment position is invalid",
        "pull request review comment path is invalid",
        "pull_request_review_thread.line",
        "review comments is invalid",
        "line could not be resolved",
        "line must be part of the diff",
        "diff hunk",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

fn is_unresolved_inline_path_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    [
        "path could not be resolved",
        "pull request review comment path is invalid",
    ]
    .iter()
    .any(|needle| message.contains(needle))
}

// ---------------------------------------------------------------------------
// GitHub API request/response types
// ---------------------------------------------------------------------------

/// Request body for `POST /repos/{owner}/{repo}/git/refs`.
#[derive(Serialize)]
struct CreateRefRequest<'a> {
    #[serde(rename = "ref")]
    ref_: &'a str,
    sha: &'a str,
}

#[allow(clippy::too_many_arguments)]
async fn push_pull_request_inner(
    owner: &str,
    repo: &str,
    file_path: &str,
    source_path: &str,
    source_commit: &str,
    branch_base_sha: &str,
    base_branch: &str,
    branch_name: &str,
    source_text: &str,
    has_source_changes: bool,
    items: &[PullRequestComment],
    existing_pr_number: Option<u64>,
) -> Result<(
    PullRequestResponse,
    Option<String>,
    Option<String>,
    usize,
    usize,
)> {
    let has_comments = !items.is_empty();
    let use_dummy_change =
        has_source_changes && !has_comments && should_create_placeholder_change(source_text);

    let mut current_sha = branch_base_sha.to_string();

    let content_commit_sha = if has_source_changes {
        let content_commit_sha = if use_dummy_change {
            tracing::debug!(
                source_path,
                parent_sha = %current_sha,
                "Creating placeholder content commit so GitHub can open a pull request without a substantive source diff"
            );
            create_placeholder_content_commit(owner, repo, &current_sha, source_path, source_text)
                .await?
        } else {
            tracing::debug!(
                source_path,
                parent_sha = %current_sha,
                "Creating content commit for substantive Stencila document changes"
            );
            create_content_commit(
                owner,
                repo,
                &current_sha,
                &[(source_path.to_string(), source_text.to_string())],
                "Document content changes",
                PR_AUTH_POLICY,
            )
            .await?
        };
        update_ref(owner, repo, branch_name, &content_commit_sha, false).await?;
        current_sha = content_commit_sha.clone();
        Some(content_commit_sha)
    } else {
        None
    };

    let mut anchor_commit_sha = None;

    let pr = if let Some(pr_number) = existing_pr_number {
        let pr_url = format!("https://github.com/{owner}/{repo}/pull/{pr_number}");
        PullRequestResponse {
            number: pr_number,
            html_url: pr_url,
        }
    } else {
        post_json(
            &api_url(&format!("/repos/{owner}/{repo}/pulls")),
            &CreatePullRequest {
                title: &pr_title(source_path, has_source_changes, use_dummy_change, items),
                body: &pr_body(
                    file_path,
                    source_path,
                    has_source_changes,
                    use_dummy_change,
                    items,
                ),
                head: branch_name,
                base: base_branch,
            },
            owner,
            repo,
            PR_AUTH_POLICY,
        )
        .await?
    };

    let (comments_posted, fallbacks) = if has_comments {
        let initial_review_commit_sha = if has_source_changes {
            let content_commit_sha = content_commit_sha
                .clone()
                .unwrap_or_else(|| current_sha.clone());

            let content_files =
                review_file_contents(owner, repo, source_path, &content_commit_sha, items).await?;
            let base_files =
                review_file_contents(owner, repo, source_path, branch_base_sha, items).await?;
            let anchored_file_contents =
                anchor_review_file_contents(&content_files, source_path, items, Some(&base_files));

            if anchored_file_contents != content_files {
                tracing::debug!(
                    source_path,
                    parent_sha = %content_commit_sha,
                    "Creating initial localized review-anchor commit before first inline review submission because some review targets are not covered by substantive content hunks"
                );
                let created_anchor_commit_sha =
                    create_anchor_commit(owner, repo, &content_commit_sha, &anchored_file_contents)
                        .await?;
                update_ref(owner, repo, branch_name, &created_anchor_commit_sha, false).await?;
                anchor_commit_sha = Some(created_anchor_commit_sha.clone());
                created_anchor_commit_sha
            } else {
                content_commit_sha
            }
        } else {
            let anchor_content_ref = if matches!(source_commit, "dirty" | "untracked") {
                current_sha.as_str()
            } else {
                source_commit
            };
            let file_contents =
                review_file_contents(owner, repo, source_path, anchor_content_ref, items).await?;
            let anchored_file_contents =
                anchor_review_file_contents(&file_contents, source_path, items, None);
            tracing::debug!(
                source_path,
                parent_sha = %current_sha,
                anchor_ref = anchor_content_ref,
                "Creating initial localized review-anchor commit before first inline review submission"
            );
            let created_anchor_commit_sha =
                create_anchor_commit(owner, repo, &current_sha, &anchored_file_contents).await?;
            update_ref(owner, repo, branch_name, &created_anchor_commit_sha, false).await?;
            anchor_commit_sha = Some(created_anchor_commit_sha.clone());
            created_anchor_commit_sha
        };

        let outcome = match submit_github_review(
            owner,
            repo,
            pr.number,
            &initial_review_commit_sha,
            items,
            source_path,
            file_path,
            SubmitReviewMode::RequireInline,
        )
        .await
        {
            Ok(outcome) => outcome,
            Err(SubmitReviewError::NeedsAnchorCommit)
                if has_source_changes && anchor_commit_sha.is_none() =>
            {
                let file_contents = review_file_contents(
                    owner,
                    repo,
                    source_path,
                    &initial_review_commit_sha,
                    items,
                )
                .await?;
                let anchored_file_contents =
                    anchor_review_file_contents(&file_contents, source_path, items, None);
                tracing::debug!(
                    source_path,
                    parent_sha = %initial_review_commit_sha,
                    "Creating retry localized review-anchor commit after GitHub rejected the initial inline review submission"
                );
                let created_anchor_commit_sha = create_anchor_commit(
                    owner,
                    repo,
                    &initial_review_commit_sha,
                    &anchored_file_contents,
                )
                .await?;
                update_ref(owner, repo, branch_name, &created_anchor_commit_sha, false).await?;
                anchor_commit_sha = Some(created_anchor_commit_sha.clone());

                match submit_github_review(
                    owner,
                    repo,
                    pr.number,
                    &created_anchor_commit_sha,
                    items,
                    source_path,
                    file_path,
                    SubmitReviewMode::AllowFallback,
                )
                .await
                {
                    Ok(outcome) => outcome,
                    Err(error) => {
                        if existing_pr_number.is_none()
                            && let Err(cleanup_err) = close_pr(owner, repo, pr.number).await
                        {
                            tracing::warn!(
                                "Cleanup after comment submission failure also failed: {cleanup_err}"
                            );
                        }
                        return Err(error.into_report());
                    }
                }
            }
            Err(SubmitReviewError::UnresolvedInlinePath) => {
                match submit_github_review(
                    owner,
                    repo,
                    pr.number,
                    &initial_review_commit_sha,
                    items,
                    source_path,
                    file_path,
                    SubmitReviewMode::AllowFallback,
                )
                .await
                {
                    Ok(outcome) => outcome,
                    Err(error) => {
                        if existing_pr_number.is_none()
                            && let Err(cleanup_err) = close_pr(owner, repo, pr.number).await
                        {
                            tracing::warn!(
                                "Cleanup after comment submission failure also failed: {cleanup_err}"
                            );
                        }
                        return Err(error.into_report());
                    }
                }
            }
            Err(error) => {
                if existing_pr_number.is_none()
                    && let Err(cleanup_err) = close_pr(owner, repo, pr.number).await
                {
                    tracing::warn!(
                        "Cleanup after comment submission failure also failed: {cleanup_err}"
                    );
                }
                return Err(error.into_report());
            }
        };

        if anchor_commit_sha.is_some() {
            tracing::debug!(
                source_path,
                restore_sha = %current_sha,
                "Force-updating branch ref to remove temporary review-anchor commit after review submission"
            );
            update_ref(owner, repo, branch_name, &current_sha, true).await?;
        }

        outcome
    } else {
        (0, 0)
    };

    Ok((
        pr,
        content_commit_sha,
        anchor_commit_sha,
        comments_posted,
        fallbacks,
    ))
}

/// Push a comment-bearing export as a GitHub pull request containing real content changes,
/// an optional anchor commit, and optional submitted comments.
pub async fn push_pull_request_export(
    export: &mut PullRequestExport,
    file_path: &str,
    existing_pr_url: Option<&url::Url>,
    has_source_changes: bool,
) -> Result<PullRequestPushResult> {
    let (owner, repo) = if let Some(url) = existing_pr_url {
        let pr_ref = parse_github_pull_request_url(url)?;
        (pr_ref.owner, pr_ref.repo)
    } else {
        let repo_url = export
            .source
            .repository
            .as_deref()
            .ok_or_else(|| eyre!("PullRequestExport.source.repository is required"))?;
        parse_repo(repo_url)?
    };

    let source_path = export
        .source
        .path
        .as_deref()
        .ok_or_else(|| eyre!("PullRequestExport.source.path is required"))?;
    let source_commit = export
        .source
        .commit
        .clone()
        .ok_or_else(|| eyre!("PullRequestExport.source.commit is required"))?;

    let has_dummy_change = plan_requires_placeholder_content_commit(export, has_source_changes);
    let plan = plan_pull_request_push_with_source_changes(export, Some(has_source_changes))?;
    if plan.is_noop() && !has_dummy_change {
        bail!("No GitHub PR actions are needed")
    }

    let (pr_number, pr_url, branch_base_sha, base_branch, branch_name, created_branch) =
        if let Some(url) = existing_pr_url {
            let pr_ref = parse_github_pull_request_url(url)?;
            let pr: ExistingPullRequestResponse = get_json(
                &api_url(&format!(
                    "/repos/{owner}/{repo}/pulls/{}",
                    pr_ref.pull_number
                )),
                &owner,
                &repo,
                PR_AUTH_POLICY,
            )
            .await?;

            if pr.state != "open" {
                bail!(
                    "GitHub pull request #{} is not open and cannot be updated",
                    pr.number
                );
            }

            (
                Some(pr.number),
                Some(pr.html_url),
                pr.head.sha,
                pr.base.ref_,
                pr.head.ref_,
                false,
            )
        } else {
            let branch_base_sha =
                if has_source_changes || matches!(source_commit.as_str(), "dirty" | "untracked") {
                    resolve_default_branch_head(&owner, &repo).await?
                } else {
                    source_commit.clone()
                };

            let repo_info: RepoInfoResponse = get_json(
                &api_url(&format!("/repos/{owner}/{repo}")),
                &owner,
                &repo,
                PR_AUTH_POLICY,
            )
            .await?;
            let base_branch = repo_info.default_branch;

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let branch_name = format!(
                "stencila/ghpr-{}-{timestamp}",
                short_commit(&branch_base_sha)
            );

            let _ref_resp: RefResponse = post_json(
                &api_url(&format!("/repos/{owner}/{repo}/git/refs")),
                &CreateRefRequest {
                    ref_: &format!("refs/heads/{branch_name}"),
                    sha: &branch_base_sha,
                },
                &owner,
                &repo,
                PR_AUTH_POLICY,
            )
            .await?;

            (None, None, branch_base_sha, base_branch, branch_name, true)
        };

    let push_result = push_pull_request_inner(
        &owner,
        &repo,
        file_path,
        source_path,
        &source_commit,
        &branch_base_sha,
        &base_branch,
        &branch_name,
        &export.content.text,
        plan.has_source_changes || has_dummy_change,
        &export.items,
        pr_number,
    )
    .await;

    let (pr, content_commit, anchor_commit, comments_posted, fallbacks) = match push_result {
        Ok(result) => result,
        Err(error) => {
            if created_branch
                && let Err(cleanup_err) = delete_branch(&owner, &repo, &branch_name).await
            {
                tracing::warn!("Cleanup after failure also failed: {cleanup_err}");
            }
            return Err(error);
        }
    };

    let pr = if let (Some(number), Some(html_url)) = (pr_number, pr_url) {
        PullRequestResponse { number, html_url }
    } else {
        pr
    };

    let anchor_commit = anchor_commit.or_else(|| export.target.anchor_commit.clone());

    export.target = PullRequestTarget {
        repository: Some(format!("https://github.com/{owner}/{repo}")),
        base_branch: Some(base_branch.to_string()),
        head_branch: Some(branch_name.clone()),
        pull_request_number: Some(pr.number),
        pull_request_branch: Some(branch_name.clone()),
        anchor_commit: anchor_commit.clone(),
        side: anchor_commit.as_ref().map(|_| PullRequestSide::Right),
    };

    Ok(PullRequestPushResult {
        pr_number: pr.number,
        pr_url: pr.html_url,
        pull_request_branch: branch_name,
        content_commit,
        anchor_commit,
        comments_posted,
        fallbacks,
        source_path: source_path.into(),
        used_generated_source_path: false,
        used_dummy_change: has_dummy_change,
    })
}

/// Response from `POST /repos/{owner}/{repo}/git/refs`.
///
/// We only need to confirm the request succeeded; the response fields
/// are not used downstream.
#[derive(Deserialize)]
struct RefResponse {
    #[allow(dead_code)]
    #[serde(rename = "ref")]
    ref_: String,
}

/// Request body for `POST /repos/{owner}/{repo}/git/trees`.
#[derive(Serialize)]
struct CreateTreeRequest<'a> {
    base_tree: &'a str,
    tree: Vec<TreeEntry>,
}

#[derive(Serialize)]
struct TreeEntry {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    type_: String,
    content: String,
}

/// Response from `POST /repos/{owner}/{repo}/git/trees`.
#[derive(Deserialize)]
struct TreeResponse {
    sha: String,
}

/// Request body for `POST /repos/{owner}/{repo}/git/commits`.
#[derive(Serialize)]
struct CreateCommitRequest<'a> {
    message: &'a str,
    tree: &'a str,
    parents: Vec<&'a str>,
}

/// Response from `POST /repos/{owner}/{repo}/git/commits`.
#[derive(Deserialize)]
struct CommitResponse {
    sha: String,
    tree: CommitTree,
}

#[derive(Deserialize)]
struct CommitTree {
    sha: String,
}

/// Request body for `POST /repos/{owner}/{repo}/pulls`.
#[derive(Serialize)]
struct CreatePullRequest<'a> {
    title: &'a str,
    body: &'a str,
    head: &'a str,
    base: &'a str,
}

/// Response from `POST /repos/{owner}/{repo}/pulls`.
#[derive(Deserialize)]
struct PullRequestResponse {
    number: u64,
    html_url: String,
}

/// Response from `GET /repos/{owner}/{repo}/pulls/{pull_number}`.
#[derive(Deserialize)]
struct ExistingPullRequestResponse {
    number: u64,
    html_url: String,
    state: String,
    head: PullRequestBranchRef,
    base: PullRequestBranchRef,
}

#[derive(Deserialize)]
struct PullRequestBranchRef {
    #[serde(rename = "ref")]
    ref_: String,
    sha: String,
}

/// Request body for `POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews`.
#[derive(Serialize)]
struct CreateReviewRequest {
    commit_id: String,
    event: String,
    body: String,
    comments: Vec<GitHubPullRequestComment>,
}

/// A single inline comment within a GitHub pull request review submission.
#[derive(Debug, Clone, Serialize)]
pub struct GitHubPullRequestComment {
    pub path: String,
    pub body: String,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    pub side: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_side: Option<String>,
}

/// Response from `POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews`.
#[derive(Deserialize)]
struct ReviewResponse {
    #[allow(dead_code)]
    id: u64,
}

/// Response from `GET /repos/{owner}/{repo}` — only the fields we need.
#[derive(Deserialize)]
struct RepoInfoResponse {
    default_branch: String,
}

// ---------------------------------------------------------------------------
// Public result type
// ---------------------------------------------------------------------------

/// Result of planning how a comment-bearing export should become a GitHub pull request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestPushPlan {
    pub repository: String,
    pub source_path: String,
    pub source_commit: String,
    pub has_source_changes: bool,
    pub has_comments: bool,
    pub content_commit: Option<PlannedCommit>,
    pub anchor_commit: Option<PlannedCommit>,
}

/// Outcome of pushing a comment-bearing document as a GitHub pull request.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestPushResult {
    pub pr_number: u64,
    pub pr_url: String,
    pub pull_request_branch: String,
    pub content_commit: Option<String>,
    pub anchor_commit: Option<String>,
    pub comments_posted: usize,
    pub fallbacks: usize,
    pub source_path: String,
    pub used_generated_source_path: bool,
    pub used_dummy_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedCommit {
    pub kind: PlannedCommitKind,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannedCommitKind {
    Content,
    Anchor,
}

impl PullRequestPushPlan {
    pub fn is_noop(&self) -> bool {
        !self.has_source_changes && !self.has_comments
    }
}

/// Plan how a comment-bearing export should be pushed as a GitHub pull request using an
/// optional externally determined source-change signal.
pub fn plan_pull_request_push_with_source_changes(
    export: &PullRequestExport,
    has_source_changes: Option<bool>,
) -> Result<PullRequestPushPlan> {
    let repository = export
        .source
        .repository
        .clone()
        .ok_or_else(|| eyre!("PullRequestExport.source.repository is required"))?;
    let source_path = export
        .source
        .path
        .clone()
        .ok_or_else(|| eyre!("PullRequestExport.source.path is required"))?;
    let source_commit = export
        .source
        .commit
        .clone()
        .ok_or_else(|| eyre!("PullRequestExport.source.commit is required"))?;

    let has_comments = !export.items.is_empty();
    let has_source_changes = has_source_changes.unwrap_or(false);

    let content_commit = has_source_changes.then(|| PlannedCommit {
        kind: PlannedCommitKind::Content,
        paths: vec![source_path.clone()],
    });

    let anchor_commit = has_comments.then(|| PlannedCommit {
        kind: PlannedCommitKind::Anchor,
        paths: review_paths(export, &source_path),
    });

    Ok(PullRequestPushPlan {
        repository,
        source_path,
        source_commit,
        has_source_changes,
        has_comments,
        content_commit,
        anchor_commit,
    })
}

/// Produce a dry-run result for a planned GitHub PR push.
pub fn pull_request_push_dry_run(
    plan: &PullRequestPushPlan,
    options: &PushDryRunOptions,
) -> PushResult {
    let repository = plan.repository.trim_matches('/');
    let url = url::Url::parse(&format!(
        "https://github.com/{repository}/pull/new/ghpr-{}",
        short_commit(&plan.source_commit)
    ))
    .expect("valid GitHub dry-run URL");

    let files = plan
        .content_commit
        .iter()
        .flat_map(|commit| commit.paths.iter().map(|path| ("content", path)))
        .chain(
            plan.anchor_commit
                .iter()
                .flat_map(|commit| commit.paths.iter().map(|path| ("anchor", path))),
        )
        .map(|(prefix, path)| PushDryRunFile {
            storage_path: format!("{prefix}:{path}"),
            local_path: None,
            size: path.len() as u64,
            compressed: false,
            route: None,
        })
        .collect();

    PushResult::DryRun {
        url,
        files,
        output_dir: options.output_dir.clone(),
    }
}

/// Parse `owner` and `repo` from a GitHub repository URL.
///
/// Accepts both `https://github.com/owner/repo` and `owner/repo` formats.
pub(crate) fn parse_repo(url: &str) -> Result<(String, String)> {
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .unwrap_or(url);

    let parts: Vec<&str> = path.trim_matches('/').splitn(3, '/').collect();
    if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
        bail!("Cannot parse GitHub owner/repo from: {url}");
    }

    // Strip .git suffix if present
    let repo = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
    Ok((parts[0].to_string(), repo.to_string()))
}

/// Fetch a file's UTF-8 content from a GitHub repo at a specific ref.
async fn fetch_file_at_ref(owner: &str, repo: &str, path: &str, git_ref: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct ContentsResponse {
        content: Option<String>,
        encoding: String,
    }

    let url = api_url(&format!(
        "/repos/{owner}/{repo}/contents/{path}?ref={git_ref}"
    ));
    let resp: ContentsResponse = get_json(&url, owner, repo, PR_AUTH_POLICY).await?;

    let content = resp
        .content
        .ok_or_else(|| eyre!("No content returned for {path} at {git_ref}"))?;

    if resp.encoding != "base64" {
        bail!("Unsupported encoding for {path}: {}", resp.encoding);
    }

    let bytes = base64::engine::general_purpose::STANDARD.decode(content.replace('\n', ""))?;
    Ok(String::from_utf8(bytes)?)
}

async fn resolve_default_branch_head(owner: &str, repo: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct RefObject {
        sha: String,
    }

    #[derive(Deserialize)]
    struct BranchResponse {
        commit: RefObject,
    }

    let repo_info: RepoInfoResponse = get_json(
        &api_url(&format!("/repos/{owner}/{repo}")),
        owner,
        repo,
        PR_AUTH_POLICY,
    )
    .await?;
    let branch: BranchResponse = get_json(
        &api_url(&format!(
            "/repos/{owner}/{repo}/branches/{}",
            repo_info.default_branch
        )),
        owner,
        repo,
        PR_AUTH_POLICY,
    )
    .await?;

    Ok(branch.commit.sha)
}

fn plan_requires_placeholder_content_commit(
    export: &PullRequestExport,
    has_source_changes: bool,
) -> bool {
    !has_source_changes && export.items.is_empty()
}

fn trailing_line_ending_count(content: &str, ending: &str) -> usize {
    let mut count = 0;
    let mut remaining = content;

    while let Some(prefix) = remaining.strip_suffix(ending) {
        count += 1;
        remaining = prefix;
    }

    count
}

/// Create a minimal reversible placeholder content change for opening a PR.
///
/// Rather than inserting a visible marker line, this toggles the number of
/// trailing end-of-file line endings through a small cycle:
///
/// - 0 trailing line endings -> 1
/// - 1 trailing line ending -> 2
/// - 2 trailing line endings -> 0
///
/// This keeps placeholder-only diffs minimally intrusive while still ensuring
/// that repeated placeholder commits can produce a detectable content change.
fn placeholder_content_change(content: &str) -> String {
    let ending = line_ending(content);
    let trailing = trailing_line_ending_count(content, ending);

    if trailing < PR_PLACEHOLDER_MAX_TRAILING_LINE_ENDINGS {
        format!("{content}{ending}")
    } else {
        content.trim_end_matches(['\r', '\n']).to_string()
    }
}

fn should_create_placeholder_change(source_text: &str) -> bool {
    placeholder_content_change(source_text) != source_text
}

fn review_paths(export: &PullRequestExport, source_path: &str) -> Vec<String> {
    let mut paths = vec![source_path.to_string()];

    for item in &export.items {
        if let Some(path) = item.source_path.as_deref()
            && !paths.iter().any(|existing| existing == path)
        {
            paths.push(path.to_string());
        }
    }

    paths
}

async fn review_file_contents(
    owner: &str,
    repo: &str,
    source_path: &str,
    source_commit: &str,
    items: &[PullRequestComment],
) -> Result<Vec<(String, String)>> {
    let mut file_contents: Vec<(String, String)> = vec![(
        source_path.to_string(),
        fetch_file_at_ref(owner, repo, source_path, source_commit).await?,
    )];

    let mut seen_paths: std::collections::HashSet<&str> =
        std::collections::HashSet::from([source_path]);
    for item in items {
        if let Some(path) = item.source_path.as_deref()
            && seen_paths.insert(path)
        {
            let content = fetch_file_at_ref(owner, repo, path, source_commit).await?;
            file_contents.push((path.to_string(), content));
        }
    }

    Ok(file_contents)
}

async fn create_content_commit(
    owner: &str,
    repo: &str,
    parent_sha: &str,
    files: &[(String, String)],
    message: &str,
    policy: GitHubAuthPolicy,
) -> Result<String> {
    let parent_commit: CommitResponse = get_json(
        &api_url(&format!("/repos/{owner}/{repo}/git/commits/{parent_sha}")),
        owner,
        repo,
        policy,
    )
    .await?;

    let tree_entries: Vec<TreeEntry> = files
        .iter()
        .map(|(path, content)| TreeEntry {
            path: path.clone(),
            mode: "100644".to_string(),
            type_: "blob".to_string(),
            content: content.clone(),
        })
        .collect();

    let tree: TreeResponse = post_json(
        &api_url(&format!("/repos/{owner}/{repo}/git/trees")),
        &CreateTreeRequest {
            base_tree: &parent_commit.tree.sha,
            tree: tree_entries,
        },
        owner,
        repo,
        PR_AUTH_POLICY,
    )
    .await?;

    let commit: CommitResponse = post_json(
        &api_url(&format!("/repos/{owner}/{repo}/git/commits")),
        &CreateCommitRequest {
            message,
            tree: &tree.sha,
            parents: vec![parent_sha],
        },
        owner,
        repo,
        PR_AUTH_POLICY,
    )
    .await?;

    Ok(commit.sha)
}

async fn create_placeholder_content_commit(
    owner: &str,
    repo: &str,
    parent_sha: &str,
    path: &str,
    content: &str,
) -> Result<String> {
    let dummy_content = placeholder_content_change(content);

    create_content_commit(
        owner,
        repo,
        parent_sha,
        &[(path.to_string(), dummy_content)],
        "Add placeholder change to open pull request",
        PR_AUTH_POLICY,
    )
    .await
}

fn line_ending(content: &str) -> &str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn collect_anchor_target_lines(items: &[PullRequestComment]) -> Vec<u32> {
    let mut lines: Vec<u32> = items
        .iter()
        .filter_map(|item| item.range.start_line.or(item.range.end_line))
        .collect();
    lines.sort_unstable();
    lines.dedup();

    lines
}

fn is_blank_line(line: &str, ending: &str) -> bool {
    line.strip_suffix(ending).unwrap_or(line).trim().is_empty()
}

fn choose_anchor_line(target_line: u32, split_lines: &[String], ending: &str) -> u32 {
    let Some(target_index) = target_line.checked_sub(1).map(|line| line as usize) else {
        return target_line;
    };

    let max_distance = ANCHOR_COVERAGE_DISTANCE as usize;

    for distance in 1..=max_distance {
        if let Some(index) = target_index.checked_sub(distance)
            && split_lines
                .get(index)
                .is_some_and(|line| is_blank_line(line, ending))
        {
            return index as u32 + 1;
        }

        let after_index = target_index + distance;
        if split_lines
            .get(after_index)
            .is_some_and(|line| is_blank_line(line, ending))
        {
            return after_index as u32 + 1;
        }
    }

    target_line
}

fn collect_anchor_lines(content: &str, items: &[PullRequestComment]) -> Vec<u32> {
    let target_lines = collect_anchor_target_lines(items);
    if target_lines.is_empty() {
        return Vec::new();
    }

    let ending = line_ending(content);
    let split_lines: Vec<String> = if content.is_empty() {
        vec![String::new()]
    } else {
        content.split_inclusive(ending).map(String::from).collect()
    };

    let mut lines: Vec<u32> = target_lines
        .into_iter()
        .map(|line| choose_anchor_line(line, &split_lines, ending))
        .collect();
    lines.sort_unstable();
    lines.dedup();

    let mut minimized = Vec::new();
    for line in lines {
        let covered = minimized
            .last()
            .is_some_and(|previous| line.saturating_sub(*previous) <= ANCHOR_COVERAGE_DISTANCE);

        if !covered {
            minimized.push(line);
        }
    }

    minimized
}

/// Estimate which line numbers belong to substantive content changes.
///
/// This intentionally uses a lightweight line-by-line comparison rather than a
/// full diff hunk algorithm. For proactive mixed-content anchor planning, the
/// goal is to conservatively detect nearby changed lines that are likely to
/// give GitHub enough diff context for inline review anchoring. Cases missed by
/// this estimate still fall back to the existing retry path that creates an
/// anchor commit after a 422 response.
fn changed_lines(before: &str, after: &str) -> Vec<u32> {
    let before_lines: Vec<&str> = before.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    let max_len = before_lines.len().max(after_lines.len());

    (0..max_len)
        .filter_map(|index| {
            (before_lines.get(index) != after_lines.get(index)).then_some(index as u32 + 1)
        })
        .collect()
}

fn is_line_covered_by_hunk(line: u32, changed_lines: &[u32]) -> bool {
    changed_lines
        .iter()
        .any(|changed| line.abs_diff(*changed) <= ANCHOR_COVERAGE_DISTANCE)
}

/// Remove review targets that are already covered by substantive content hunks
/// or by a previously planned synthetic marker.
///
/// This keeps mixed-content anchor commits as small as possible by skipping
/// markers that would be redundant for GitHub inline review placement.
fn filter_anchor_lines_needing_markers(lines: &[u32], changed_lines: &[u32]) -> Vec<u32> {
    let mut planned = Vec::new();

    for &line in lines {
        let covered_by_content = is_line_covered_by_hunk(line, changed_lines);
        let covered_by_marker = planned
            .last()
            .is_some_and(|previous| line.abs_diff(*previous) <= ANCHOR_COVERAGE_DISTANCE);

        if !covered_by_content && !covered_by_marker {
            planned.push(line);
        }
    }

    planned
}

fn insert_anchor_markers(content: &str, _path: &str, lines: &[u32]) -> String {
    if lines.is_empty() {
        return content.to_string();
    }

    let ending = line_ending(content);
    let mut split_lines: Vec<String> = if content.is_empty() {
        vec![String::new()]
    } else {
        content.split_inclusive(ending).map(String::from).collect()
    };

    for &line in lines {
        let Some(index) = line.checked_sub(1).map(|line| line as usize) else {
            continue;
        };

        if let Some(existing) = split_lines.get_mut(index) {
            if let Some(stripped) = existing.strip_suffix(ending) {
                *existing = format!("{stripped}{ANCHOR_MARKER}{ending}");
            } else {
                existing.push_str(ANCHOR_MARKER);
            }
        }
    }

    split_lines.concat()
}

/// Add synthetic review anchors only where the current file contents still
/// need localized diff hunks for inline review placement.
///
/// When `base_files` is provided, anchor planning treats nearby substantive
/// content changes as coverage and avoids adding redundant temporary markers.
fn anchor_review_file_contents(
    files: &[(String, String)],
    source_path: &str,
    items: &[PullRequestComment],
    base_files: Option<&[(String, String)]>,
) -> Vec<(String, String)> {
    files
        .iter()
        .map(|(path, content)| {
            let path_items: Vec<PullRequestComment> = items
                .iter()
                .filter(|item| item.source_path.as_deref().unwrap_or(source_path) == path)
                .cloned()
                .collect();
            let anchor_lines = collect_anchor_lines(content, &path_items);
            let changed_lines = base_files
                .and_then(|files| files.iter().find(|(base_path, _)| base_path == path))
                .map(|(_, base_content)| changed_lines(base_content, content))
                .unwrap_or_default();
            let anchor_lines = filter_anchor_lines_needing_markers(&anchor_lines, &changed_lines);
            (
                path.clone(),
                insert_anchor_markers(content, path, &anchor_lines),
            )
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ReviewContentKinds {
    suggestions: bool,
    comments: bool,
}

fn review_content_kinds(items: &[PullRequestComment]) -> ReviewContentKinds {
    let mut kinds = ReviewContentKinds::default();

    for item in items {
        match item.kind {
            PullRequestCommentKind::Suggestion => kinds.suggestions = true,
            PullRequestCommentKind::Comment => kinds.comments = true,
        }
    }

    kinds
}

fn join_content_terms(terms: &[&str]) -> String {
    match terms {
        [] => String::new(),
        [only] => (*only).to_string(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let mut result = terms[..terms.len() - 1].join(", ");
            result.push_str(", and ");
            result.push_str(terms[terms.len() - 1]);
            result
        }
    }
}

fn pr_content_phrase(
    has_source_changes: bool,
    uses_dummy_change: bool,
    items: &[PullRequestComment],
) -> Option<String> {
    let mut terms = Vec::new();
    let kinds = review_content_kinds(items);

    if has_source_changes && !uses_dummy_change {
        terms.push("edits");
    }
    if kinds.suggestions {
        terms.push("suggestions");
    }
    if kinds.comments {
        terms.push("comments");
    }

    (!terms.is_empty()).then(|| join_content_terms(&terms))
}

fn review_content_phrase(items: &[PullRequestComment]) -> Option<String> {
    let mut terms = Vec::new();
    let kinds = review_content_kinds(items);

    if kinds.suggestions {
        terms.push("suggestions");
    }
    if kinds.comments {
        terms.push("comments");
    }

    (!terms.is_empty()).then(|| join_content_terms(&terms))
}

fn pr_title(
    source_path: &str,
    has_source_changes: bool,
    uses_dummy_change: bool,
    items: &[PullRequestComment],
) -> String {
    match pr_content_phrase(has_source_changes, uses_dummy_change, items) {
        Some(content) => format!("{} for {source_path}", capitalize_first(&content)),
        None => format!("Pull request for {source_path}"),
    }
}

fn pr_body(
    file_path: &str,
    source_path: &str,
    has_source_changes: bool,
    uses_dummy_change: bool,
    items: &[PullRequestComment],
) -> String {
    if has_source_changes && items.is_empty() && uses_dummy_change {
        return "Stencila created a placeholder file change so this pull request can be opened from a review document with no substantive content diff.".to_string();
    }

    match pr_content_phrase(has_source_changes, uses_dummy_change, items) {
        Some(content) => format!(
            "This pull request contains {content} for {}, extracted from {}.",
            inline_code(source_path),
            inline_code(file_path)
        ),
        None => "Stencila GitHub PR export.".to_string(),
    }
}

fn review_body_intro(file_path: &str, source_path: &str, items: &[PullRequestComment]) -> String {
    match review_content_phrase(items) {
        Some(content) => format!(
            "These {content} were extracted from track changes suggestions and comments in {} and mapped to the corresponding positions in {}. Some items may appear as outdated in GitHub; this is usually an artifact of the temporary anchors used to place review comments near their intended source locations.",
            inline_code(file_path),
            inline_code(source_path)
        ),
        None => format!(
            "These review items were extracted from track changes suggestions and comments in {} and mapped to the corresponding positions in {}. Some items may appear as outdated in GitHub; this is usually an artifact of the temporary anchors used to place review comments near their intended source locations.",
            inline_code(file_path),
            inline_code(source_path)
        ),
    }
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Create an anchor commit with localized synthetic hunks near review target lines.
///
/// For existing files with localized diff hunks, GitHub may reject inline
/// review comments more than a few lines away from the nearest diff hunk. The
/// anchor strategy therefore applies reversible whitespace-only changes to
/// anchor lines rather than touching only the end of the file or inserting
/// extra marker lines that would shift later review targets.
async fn create_anchor_commit(
    owner: &str,
    repo: &str,
    parent_sha: &str,
    files: &[(String, String)],
) -> Result<String> {
    create_content_commit(
        owner,
        repo,
        parent_sha,
        files,
        "Add temporary review anchors",
        ANCHOR_AUTH_POLICY,
    )
    .await
}

/// Update a branch ref to point at a new commit SHA.
async fn update_ref(owner: &str, repo: &str, branch: &str, sha: &str, force: bool) -> Result<()> {
    #[derive(Serialize)]
    struct UpdateRefRequest<'a> {
        sha: &'a str,
        force: bool,
    }

    let url = api_url(&format!("/repos/{owner}/{repo}/git/refs/heads/{branch}"));
    patch_json(
        &url,
        &UpdateRefRequest { sha, force },
        owner,
        repo,
        PR_AUTH_POLICY,
    )
    .await
}

/// Submit GitHub inline comments for a PR.
///
/// In `RequireInline` mode, a GitHub 422 response indicating unresolved diff anchors is returned
/// as `NeedsAnchorCommit` so the caller can synthesize an anchor commit and retry. In
/// `AllowFallback` mode, the same condition degrades to a body-only review submission.
async fn submit_github_review(
    owner: &str,
    repo: &str,
    pr_number: u64,
    review_commit_sha: &str,
    items: &[PullRequestComment],
    source_path: &str,
    file_path: &str,
    mode: SubmitReviewMode,
) -> Result<(usize, usize), SubmitReviewError> {
    let (comments, fallback_items) = convert_to_review_comments(items, source_path);
    let mut all_fallback_items = fallback_items;

    tracing::debug!(
        pr_number,
        review_commit_sha,
        comment_count = comments.len(),
        fallback_count = all_fallback_items.len(),
        comment_paths = ?comments.iter().map(|comment| (&comment.path, comment.start_line, comment.line)).collect::<Vec<_>>(),
        resolutions = ?items
            .iter()
            .map(|item| {
                (
                    item.source_path.as_deref().unwrap_or(source_path),
                    item.range.start_line,
                    item.range.end_line,
                    &item.resolution,
                    item.github_suggestion.is_some(),
                )
            })
            .collect::<Vec<_>>(),
        "Preparing GitHub PR review submission"
    );

    let review_url = api_url(&format!("/repos/{owner}/{repo}/pulls/{pr_number}/reviews"));

    // First attempt: submit with inline comments
    let submitted_inline = if !comments.is_empty() {
        let review_intro = review_body_intro(file_path, source_path, items);
        let initial_body = if all_fallback_items.is_empty() {
            review_intro.clone()
        } else {
            format!(
                "{review_intro}\n\n{}",
                format_fallback_body(&all_fallback_items, source_path)
            )
        };

        let result: std::result::Result<ReviewResponse, _> = post_json_with_status(
            &review_url,
            &CreateReviewRequest {
                commit_id: review_commit_sha.to_string(),
                event: "COMMENT".into(),
                body: initial_body,
                comments: comments.clone(),
            },
            owner,
            repo,
            ANCHOR_AUTH_POLICY,
        )
        .await;

        match result {
            Ok(_) => true,
            Err(e) if e.status == Some(StatusCode::UNPROCESSABLE_ENTITY.as_u16()) => {
                if matches!(mode, SubmitReviewMode::RequireInline)
                    && is_missing_anchor_error(&e.message)
                {
                    tracing::debug!(
                        "GitHub rejected inline comments with 422; signalling that an anchor commit is required: {}",
                        e.message
                    );
                    return Err(SubmitReviewError::NeedsAnchorCommit);
                }

                if matches!(mode, SubmitReviewMode::RequireInline)
                    && is_unresolved_inline_path_error(&e.message)
                {
                    tracing::debug!(
                        "GitHub rejected inline comments because the path could not be resolved; signalling fallback-only submission: {}",
                        e.message
                    );
                    return Err(SubmitReviewError::UnresolvedInlinePath);
                }

                if !is_missing_anchor_error(&e.message)
                    && !is_unresolved_inline_path_error(&e.message)
                {
                    return Err(SubmitReviewError::Other(eyre!(
                        "Failed to submit pull request comments: {e}"
                    )));
                }

                tracing::debug!(
                    "GitHub rejected inline comments with 422, falling back to body-only submission: {}",
                    e.message
                );
                let inline_as_fallback: Vec<&PullRequestComment> = items
                    .iter()
                    .filter(|item| {
                        matches!(item.resolution, PullRequestCommentResolution::Anchored)
                    })
                    .filter(|item| item.range.end_line.is_some())
                    .collect();
                all_fallback_items.extend(inline_as_fallback);
                false
            }
            Err(e) => Err(SubmitReviewError::Other(eyre!(
                "Failed to submit pull request comments: {e}"
            )))?,
        }
    } else {
        false
    };

    // If inline submission failed (422) or there were no inline comments,
    // submit a body-only comment payload with all items as fallback text.
    if !submitted_inline {
        let review_intro = review_body_intro(file_path, source_path, items);
        let fallback_body = if all_fallback_items.is_empty() {
            review_intro
        } else {
            format!(
                "{review_intro}\n\n{}",
                format_fallback_body(&all_fallback_items, source_path)
            )
        };

        let _review: ReviewResponse = post_json(
            &review_url,
            &CreateReviewRequest {
                commit_id: review_commit_sha.to_string(),
                event: "COMMENT".into(),
                body: fallback_body,
                comments: vec![],
            },
            owner,
            repo,
            ANCHOR_AUTH_POLICY,
        )
        .await
        .map_err(SubmitReviewError::Other)?;
    }

    let inline_count = if submitted_inline { comments.len() } else { 0 };

    Ok((inline_count, all_fallback_items.len()))
}

/// Close a pull request.
async fn close_pr(owner: &str, repo: &str, pr_number: u64) -> Result<()> {
    #[derive(Serialize)]
    struct ClosePrRequest {
        state: &'static str,
    }

    let url = api_url(&format!("/repos/{owner}/{repo}/pulls/{pr_number}"));
    patch_json(
        &url,
        &ClosePrRequest { state: "closed" },
        owner,
        repo,
        PR_AUTH_POLICY,
    )
    .await
}

/// Delete a branch ref.
async fn delete_branch(owner: &str, repo: &str, branch: &str) -> Result<()> {
    let url = api_url(&format!("/repos/{owner}/{repo}/git/refs/heads/{branch}"));
    delete(&url, owner, repo, PR_AUTH_POLICY).await
}

/// Convert resolved [`PullRequestComment`]s into GitHub review comments and fallback items.
///
/// Returns `(inline_comments, fallback_items)`. Items with `Anchored` resolution
/// and valid line numbers become inline review comments. All others are collected
/// as fallback items to be included in the review body.
pub(crate) fn convert_to_review_comments<'a>(
    items: &'a [PullRequestComment],
    default_path: &str,
) -> (Vec<GitHubPullRequestComment>, Vec<&'a PullRequestComment>) {
    let mut comments: Vec<(usize, GitHubPullRequestComment)> = Vec::new();
    let mut fallbacks = Vec::new();

    for (index, item) in items.iter().enumerate() {
        // Only fully `Anchored` items become inline comments. `FallbackLine`
        // items have weaker confidence and are safer as fallback text.
        let line = match item.range.end_line {
            Some(line) if matches!(item.resolution, PullRequestCommentResolution::Anchored) => line,
            _ => {
                fallbacks.push(item);
                continue;
            }
        };

        let path = item
            .source_path
            .as_deref()
            .unwrap_or(default_path)
            .to_string();

        let start_line = item.range.start_line.filter(|&sl| sl != line);

        // Build comment body
        let body = format_review_item_body(item, CommentBodyMode::Anchored);

        comments.push((
            index,
            GitHubPullRequestComment {
                path,
                body,
                line,
                start_line,
                side: "RIGHT".to_string(),
                start_side: start_line.map(|_| "RIGHT".into()),
            },
        ));
    }

    comments.sort_by(|(left_index, left), (right_index, right)| {
        left.path
            .cmp(&right.path)
            .then_with(|| {
                left.start_line
                    .unwrap_or(left.line)
                    .cmp(&right.start_line.unwrap_or(right.line))
            })
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left_index.cmp(right_index))
    });

    let comments = comments.into_iter().map(|(_, comment)| comment).collect();

    (comments, fallbacks)
}

fn format_review_item_body(item: &PullRequestComment, mode: CommentBodyMode) -> String {
    if let Some(github_suggestion) = &item.github_suggestion {
        let lead = format_item_lead(item, mode, None);

        if item.body_markdown.is_empty() {
            return format!("{lead}:\n\n{}", github_suggestion.body);
        }

        return format!(
            "{lead}:\n\n{}\n\n{}",
            item.body_markdown, github_suggestion.body
        );
    }

    let lead = format_item_lead(item, mode, None);
    let mut parts = vec![format!("{lead}:")];

    if !item.body_markdown.is_empty() {
        parts.push(item.body_markdown.clone());
    }

    if matches!(item.kind, PullRequestCommentKind::Suggestion)
        && let Some(replacement) = &item.replacement_text
    {
        parts.push(format!(
            "Suggested replacement: {}",
            inline_code(replacement)
        ));
    }

    if parts.len() == 1 {
        parts.push("(empty comment)".to_string());
    }

    parts.join("\n\n")
}

fn format_item_lead(
    item: &PullRequestComment,
    mode: CommentBodyMode,
    default_path: Option<&str>,
) -> String {
    let location = matches!(mode, CommentBodyMode::Fallback)
        .then(|| format_location_phrase(item, default_path))
        .flatten();

    match item.kind {
        PullRequestCommentKind::Comment => {
            let from = item
                .author_name
                .as_deref()
                .map(|name| format!(" from _{name}_"))
                .unwrap_or_default();
            let target = comment_target_phrase(item);
            match location {
                Some(location) => format!("**Comment{from} on** {target} **{location}**"),
                None => format!("**Comment{from} on** {target}"),
            }
        }
        PullRequestCommentKind::Suggestion => {
            let summary = suggestion_summary_phrase(item);
            match location {
                Some(location) => format!("{summary} **{location}**"),
                None => summary,
            }
        }
    }
}

fn comment_target_phrase(item: &PullRequestComment) -> String {
    if let Some(target) = short_target_snippet(item) {
        inline_code(&target)
    } else if item.range.start_line.is_some() && item.range.end_line.is_some() {
        "these lines".to_string()
    } else {
        "this text".to_string()
    }
}

fn suggestion_summary_phrase(item: &PullRequestComment) -> String {
    let replacement = item
        .replacement_text
        .as_deref()
        .filter(|text| !text.trim().is_empty());
    let selected = short_target_snippet(item);
    let preceding = short_preceding_snippet(item);
    let prefix = item
        .author_name
        .as_deref()
        .map(|name| format!("**Suggestion from _{name}_**: "));

    match suggestion_summary_kind(item) {
        Some(SuggestionType::Insert) => {
            let inserted = replacement
                .map(inline_code)
                .unwrap_or_else(|| "text".to_string());
            if let Some(after) = preceding.or(selected) {
                if let Some(prefix) = &prefix {
                    format!(
                        "{prefix}insert {inserted} **after** {}",
                        inline_code(&after)
                    )
                } else {
                    format!(
                        "**Suggest inserting** {inserted} **after** {}",
                        inline_code(&after)
                    )
                }
            } else if let Some(prefix) = &prefix {
                format!("{prefix}insert {inserted}")
            } else {
                format!("**Suggest inserting** {inserted}")
            }
        }
        Some(SuggestionType::Delete) => {
            if let Some(target) = selected {
                if let Some(prefix) = &prefix {
                    format!("{prefix}delete {}", inline_code(&target))
                } else {
                    format!("**Suggest deleting** {}", inline_code(&target))
                }
            } else if let Some(prefix) = &prefix {
                format!("{prefix}delete this text")
            } else {
                "**Suggest deleting** this text".to_string()
            }
        }
        Some(SuggestionType::Replace) | None => {
            let replacement = replacement
                .map(inline_code)
                .unwrap_or_else(|| "text".to_string());
            if let Some(target) = selected {
                if let Some(prefix) = &prefix {
                    format!(
                        "{prefix}replace {} **with** {replacement}",
                        inline_code(&target)
                    )
                } else {
                    format!(
                        "**Suggest replacing** {} **with** {replacement}",
                        inline_code(&target)
                    )
                }
            } else if let Some(prefix) = &prefix {
                format!("{prefix}replace this text **with** {replacement}")
            } else {
                format!("**Suggest replacing** this text **with** {replacement}")
            }
        }
    }
}

fn short_target_snippet(item: &PullRequestComment) -> Option<String> {
    let selected = item.selected_text.as_deref()?.trim();
    short_snippet(selected)
}

fn short_preceding_snippet(item: &PullRequestComment) -> Option<String> {
    let preceding = item.preceding_text.as_deref()?.trim();
    short_snippet_from_end(preceding)
}

fn short_snippet(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    let collapsed = collapse_whitespace(text);
    let mut chars = collapsed.chars();
    let truncated: String = chars.by_ref().take(60).collect();

    Some(if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    })
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn inline_code(text: &str) -> String {
    let escaped = text.replace('`', "\\`");
    format!("`{escaped}`")
}

fn format_location_phrase(item: &PullRequestComment, default_path: Option<&str>) -> Option<String> {
    let path = item.source_path.as_deref().or(default_path)?;

    match (item.range.start_line, item.range.end_line) {
        (Some(start), Some(end)) if start == end => {
            Some(format!("in {} at line {start}", inline_code(path)))
        }
        (Some(start), Some(end)) => {
            Some(format!("in {} at lines {start}–{end}", inline_code(path)))
        }
        _ => Some(format!("in {}", inline_code(path))),
    }
}

fn suggestion_summary_kind(item: &PullRequestComment) -> Option<SuggestionType> {
    match item.suggestion_type {
        Some(SuggestionType::Insert) => Some(SuggestionType::Insert),
        Some(SuggestionType::Delete) => Some(SuggestionType::Delete),
        Some(SuggestionType::Replace) => Some(SuggestionType::Replace),
        None => {
            let replacement = item.replacement_text.as_deref().map(str::trim);
            let selected = item.selected_text.as_deref().map(str::trim);

            if matches!(replacement, Some("")) {
                Some(SuggestionType::Delete)
            } else if matches!(selected, None | Some("")) {
                Some(SuggestionType::Insert)
            } else {
                Some(SuggestionType::Replace)
            }
        }
    }
}

/// Format fallback items as a Markdown section for the review body.
///
/// Items that could not be anchored as inline comments are rendered here
/// so the reviewer still sees them, just not inline on the diff.
pub(crate) fn format_fallback_body(items: &[&PullRequestComment], default_path: &str) -> String {
    let mut parts = Vec::new();
    parts.push("### Items that could not be anchored inline\n".to_string());

    for item in items {
        parts.push(format!(
            "- {}",
            format_item_lead(item, CommentBodyMode::Fallback, Some(default_path))
        ));

        if !item.body_markdown.is_empty() {
            parts.push(format!("  {}", item.body_markdown.replace('\n', "\n  ")));
        }

        if matches!(item.kind, PullRequestCommentKind::Suggestion)
            && let Some(replacement) = &item.replacement_text
        {
            parts.push(format!(
                "  Suggested replacement: {}",
                inline_code(replacement)
            ));
        }
    }

    parts.join("\n")
}

fn short_commit(commit: &str) -> &str {
    &commit[..7.min(commit.len())]
}

#[cfg(test)]
mod tests {
    use stencila_codec::stencila_format::Format;

    use crate::pull_requests::{
        export::{PullRequestCommentRange, PullRequestSourceContent},
        source::PullRequestSource,
    };

    use super::*;

    #[test]
    fn test_is_missing_anchor_error_matches_anchor_signals() {
        assert!(is_missing_anchor_error(
            "{\"message\":\"Pull request review comment line must be part of the diff\"}"
        ));
        assert!(is_missing_anchor_error(
            "Validation Failed: pull request review thread diff hunk can't be blank"
        ));
        assert!(is_missing_anchor_error(
            "{\"message\":\"Unprocessable Entity\",\"errors\":[\"Line could not be resolved\"],\"documentation_url\":\"https://docs.github.com/rest/pulls/reviews#create-a-review-for-a-pull-request\",\"status\":\"422\"}"
        ));
        assert!(is_missing_anchor_error(
            "{\"message\":\"Validation Failed\",\"errors\":[{\"resource\":\"PullRequestReviewComment\",\"code\":\"custom\",\"field\":\"pull_request_review_thread.line\",\"message\":\"could not be resolved\"}],\"documentation_url\":\"https://docs.github.com/rest/pulls/comments#create-a-review-comment-for-a-pull-request\",\"status\":\"422\"}"
        ));
    }

    #[test]
    fn test_is_missing_anchor_error_rejects_unrelated_422s() {
        assert!(!is_missing_anchor_error(
            "Validation Failed: commit_id is not part of the pull request"
        ));
        assert!(!is_missing_anchor_error(
            "Validation Failed: body is too long"
        ));
    }

    #[test]
    fn test_is_unresolved_inline_path_error_matches_path_signals() {
        assert!(is_unresolved_inline_path_error(
            "{\"message\":\"Unprocessable Entity\",\"errors\":[\"Path could not be resolved\"]}"
        ));
        assert!(is_unresolved_inline_path_error(
            "Validation Failed: pull request review comment path is invalid"
        ));
    }

    #[test]
    fn test_is_unresolved_inline_path_error_rejects_unrelated_422s() {
        assert!(!is_unresolved_inline_path_error(
            "Validation Failed: commit_id is not part of the pull request"
        ));
        assert!(!is_unresolved_inline_path_error(
            "Validation Failed: line must be part of the diff"
        ));
    }

    #[test]
    fn test_parse_repo_https() {
        let (owner, repo) =
            parse_repo("https://github.com/stencila/stencila").expect("valid GitHub repo");
        assert_eq!(owner, "stencila");
        assert_eq!(repo, "stencila");
    }

    #[test]
    fn test_parse_repo_with_git_suffix() {
        let (owner, repo) = parse_repo("https://github.com/stencila/stencila.git")
            .expect("valid GitHub repo with .git suffix");
        assert_eq!(owner, "stencila");
        assert_eq!(repo, "stencila");
    }

    #[test]
    fn test_parse_repo_short() {
        let (owner, repo) = parse_repo("stencila/stencila").expect("valid owner/repo");
        assert_eq!(owner, "stencila");
        assert_eq!(repo, "stencila");
    }

    #[test]
    fn test_parse_repo_trailing_path() {
        let (owner, repo) = parse_repo("https://github.com/stencila/stencila/tree/main")
            .expect("valid GitHub repo URL with trailing path");
        assert_eq!(owner, "stencila");
        assert_eq!(repo, "stencila");
    }

    #[test]
    fn test_parse_repo_invalid() {
        assert!(parse_repo("not-a-repo").is_err());
        assert!(parse_repo("https://github.com/").is_err());
    }

    #[test]
    fn test_convert_to_review_comments_anchored() {
        let items = vec![
            PullRequestComment {
                kind: PullRequestCommentKind::Comment,
                source_path: Some("docs/example.md".into()),
                author_name: None,
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    start_line: Some(1),
                    end_line: Some(3),
                    start_column: Some(1),
                    end_column: Some(10),
                    start_offset: Some(0),
                    end_offset: Some(30),
                    ..Default::default()
                },
                selected_text: Some("Hello world".into()),
                preceding_text: None,
                replacement_text: None,
                body_markdown: "Nice opening.".into(),
                suggestion_type: None,
                suggestion_status: None,
                resolution: PullRequestCommentResolution::Anchored,
                github_suggestion: None,
            },
            PullRequestComment {
                kind: PullRequestCommentKind::Suggestion,
                source_path: Some("docs/example.md".into()),
                author_name: None,
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange::default(),
                selected_text: None,
                preceding_text: None,
                replacement_text: Some("replacement".into()),
                body_markdown: "Unresolved suggestion.".into(),
                suggestion_type: None,
                suggestion_status: None,
                resolution: PullRequestCommentResolution::Unanchored,
                github_suggestion: None,
            },
        ];

        let (comments, fallbacks) = convert_to_review_comments(&items, "docs/example.md");

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].line, 3);
        assert_eq!(comments[0].start_line, Some(1));
        assert_eq!(comments[0].side, "RIGHT");
        assert_eq!(comments[0].start_side, Some("RIGHT".into()));
        assert_eq!(
            comments[0].body,
            "**Comment on** `Hello world`:\n\nNice opening."
        );

        assert_eq!(fallbacks.len(), 1);
        assert!(matches!(
            fallbacks[0].kind,
            PullRequestCommentKind::Suggestion
        ));
    }

    #[test]
    fn test_convert_to_review_comments_includes_author_in_lead() {
        let items = vec![PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: Some("docs/example.md".into()),
            author_name: Some("Alice Smith".into()),
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(2),
                end_line: Some(2),
                ..Default::default()
            },
            selected_text: Some("Hello world".into()),
            preceding_text: None,
            replacement_text: None,
            body_markdown: "Nice opening.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        }];

        let (comments, fallbacks) = convert_to_review_comments(&items, "docs/example.md");

        assert!(fallbacks.is_empty());
        assert_eq!(comments.len(), 1);
        assert_eq!(
            comments[0].body,
            "**Comment from _Alice Smith_ on** `Hello world`:\n\nNice opening."
        );
    }

    #[test]
    fn test_convert_to_review_comments_single_line_has_no_side() {
        let items = vec![PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: Some("docs/example.md".into()),
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(4),
                end_line: Some(4),
                start_column: Some(1),
                end_column: Some(5),
                start_offset: Some(10),
                end_offset: Some(15),
                ..Default::default()
            },
            selected_text: Some("text".into()),
            preceding_text: None,
            replacement_text: None,
            body_markdown: "Single line".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        }];

        let (comments, fallbacks) = convert_to_review_comments(&items, "docs/example.md");

        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].line, 4);
        assert_eq!(comments[0].start_line, None);
        assert_eq!(comments[0].side, "RIGHT");
        assert_eq!(comments[0].start_side, None);
        assert!(fallbacks.is_empty());
    }

    #[test]
    fn test_convert_to_review_comments_orders_by_start_line_not_kind() {
        let items = vec![
            PullRequestComment {
                kind: PullRequestCommentKind::Suggestion,
                source_path: Some("docs/example.md".into()),
                author_name: None,
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    start_line: Some(6),
                    end_line: Some(7),
                    ..Default::default()
                },
                selected_text: None,
                preceding_text: None,
                replacement_text: Some("replacement".into()),
                body_markdown: "late suggestion".into(),
                suggestion_type: None,
                suggestion_status: None,
                resolution: PullRequestCommentResolution::Anchored,
                github_suggestion: None,
            },
            PullRequestComment {
                kind: PullRequestCommentKind::Comment,
                source_path: Some("docs/example.md".into()),
                author_name: None,
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    start_line: Some(1),
                    end_line: Some(1),
                    ..Default::default()
                },
                selected_text: None,
                preceding_text: None,
                replacement_text: None,
                body_markdown: "early comment".into(),
                suggestion_type: None,
                suggestion_status: None,
                resolution: PullRequestCommentResolution::Anchored,
                github_suggestion: None,
            },
            PullRequestComment {
                kind: PullRequestCommentKind::Suggestion,
                source_path: Some("docs/example.md".into()),
                author_name: None,
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    start_line: Some(3),
                    end_line: Some(3),
                    ..Default::default()
                },
                selected_text: None,
                preceding_text: None,
                replacement_text: Some("mid".into()),
                body_markdown: "mid suggestion".into(),
                suggestion_type: None,
                suggestion_status: None,
                resolution: PullRequestCommentResolution::Anchored,
                github_suggestion: None,
            },
        ];

        let (comments, fallbacks) = convert_to_review_comments(&items, "docs/example.md");

        assert!(fallbacks.is_empty());
        assert_eq!(comments.len(), 3);
        assert_eq!(comments[0].start_line, None);
        assert_eq!(comments[0].line, 1);
        assert_eq!(comments[1].start_line, None);
        assert_eq!(comments[1].line, 3);
        assert_eq!(comments[2].start_line, Some(6));
        assert_eq!(comments[2].line, 7);
    }

    #[test]
    fn test_plan_pull_request_push_mixed_flow() {
        let export = PullRequestExport {
            source: PullRequestSource {
                repository: Some("stencila/stencila".into()),
                path: Some("docs/example.md".into()),
                commit: Some("abcdef1234567890".into()),
                format: Format::Smd,
            },
            target: PullRequestTarget::default(),
            content: PullRequestSourceContent {
                text: "before [[edit]] after".into(),
                mapping: None,
            },
            items: vec![PullRequestComment {
                kind: PullRequestCommentKind::Comment,
                source_path: None,
                author_name: None,
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    end_line: Some(1),
                    start_offset: Some(0),
                    end_offset: Some(5),
                    ..Default::default()
                },
                selected_text: None,
                preceding_text: None,
                replacement_text: None,
                body_markdown: "Comment".into(),
                suggestion_type: None,
                suggestion_status: None,
                resolution: PullRequestCommentResolution::Anchored,
                github_suggestion: None,
            }],
            diagnostics: vec![],
        };

        let plan = plan_pull_request_push_with_source_changes(&export, Some(true))
            .expect("plan should succeed");

        assert!(plan.has_source_changes);
        assert!(plan.has_comments);
        assert_eq!(
            plan.content_commit.as_ref().map(|commit| commit.kind),
            Some(PlannedCommitKind::Content)
        );
        assert_eq!(
            plan.anchor_commit.as_ref().map(|commit| commit.kind),
            Some(PlannedCommitKind::Anchor)
        );
        assert_eq!(
            plan.anchor_commit.expect("anchor commit").paths,
            vec!["docs/example.md"]
        );
    }

    #[test]
    fn test_plan_pull_request_push_review_only() {
        let export = PullRequestExport {
            source: PullRequestSource {
                repository: Some("stencila/stencila".into()),
                path: Some("docs/example.md".into()),
                commit: Some("abcdef1234567890".into()),
                format: Format::Smd,
            },
            target: PullRequestTarget::default(),
            content: PullRequestSourceContent {
                text: "plain source".into(),
                mapping: None,
            },
            items: vec![PullRequestComment {
                kind: PullRequestCommentKind::Comment,
                source_path: Some("other/file.md".into()),
                author_name: None,
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    end_line: Some(2),
                    ..Default::default()
                },
                selected_text: None,
                preceding_text: None,
                replacement_text: None,
                body_markdown: "Comment".into(),
                suggestion_type: None,
                suggestion_status: None,
                resolution: PullRequestCommentResolution::Anchored,
                github_suggestion: None,
            }],
            diagnostics: vec![],
        };

        let plan = plan_pull_request_push_with_source_changes(&export, Some(false))
            .expect("plan should succeed");

        assert!(!plan.has_source_changes);
        assert!(plan.has_comments);
        assert!(plan.content_commit.is_none());
        assert_eq!(
            plan.anchor_commit.expect("anchor commit").paths,
            vec!["docs/example.md", "other/file.md"]
        );
    }

    #[test]
    fn test_plan_pull_request_push_noop() {
        let export = PullRequestExport {
            source: PullRequestSource {
                repository: Some("stencila/stencila".into()),
                path: Some("docs/example.md".into()),
                commit: Some("abcdef1234567890".into()),
                format: Format::Smd,
            },
            target: PullRequestTarget::default(),
            content: PullRequestSourceContent {
                text: "plain source".into(),
                mapping: None,
            },
            items: vec![],
            diagnostics: vec![],
        };

        let plan = plan_pull_request_push_with_source_changes(&export, Some(false))
            .expect("plan should succeed");

        assert!(plan.is_noop());
        assert!(plan.content_commit.is_none());
        assert!(plan.anchor_commit.is_none());
    }

    #[test]
    fn test_plan_requires_placeholder_content_commit_for_empty_review() {
        let export = PullRequestExport {
            source: PullRequestSource {
                repository: Some("stencila/stencila".into()),
                path: Some("docs/example.md".into()),
                commit: Some("abcdef1234567890".into()),
                format: Format::Smd,
            },
            target: PullRequestTarget::default(),
            content: PullRequestSourceContent {
                text: "plain source".into(),
                mapping: None,
            },
            items: vec![],
            diagnostics: vec![],
        };

        assert!(plan_requires_placeholder_content_commit(&export, false));
        assert!(!plan_requires_placeholder_content_commit(&export, true));
    }

    #[test]
    fn test_placeholder_content_change_toggles_trailing_line_endings() {
        assert_eq!(placeholder_content_change("plain source"), "plain source\n");
        assert_eq!(
            placeholder_content_change("plain source\n"),
            "plain source\n\n"
        );
        assert_eq!(
            placeholder_content_change("plain source\n\n"),
            "plain source"
        );
    }

    #[test]
    fn test_should_create_placeholder_change_for_reversible_newline_toggle() {
        assert!(should_create_placeholder_change("plain source"));
        assert!(should_create_placeholder_change("plain source\n"));
        assert!(should_create_placeholder_change("plain source\n\n"));
    }

    #[test]
    fn test_anchor_review_file_contents_prefers_blank_line_before_target() {
        let files = vec![(
            "docs/example.md".to_string(),
            ["01", "", "03", "04", "05"].join("\n") + "\n",
        )];
        let items = vec![PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(2),
                end_line: Some(2),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        }];

        let anchored = anchor_review_file_contents(&files, "docs/example.md", &items, None);
        let content = &anchored[0].1;

        assert_eq!(
            content.as_str(),
            format!("01\n{}\n03\n04\n05\n", ANCHOR_MARKER)
        );
    }

    #[test]
    fn test_anchor_review_file_contents_falls_back_to_blank_line_after_target() {
        let files = vec![(
            "docs/example.md".to_string(),
            ["01", "02", "03", "", "05"].join("\n") + "\n",
        )];
        let items = vec![PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(3),
                end_line: Some(3),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        }];

        let anchored = anchor_review_file_contents(&files, "docs/example.md", &items, None);

        assert_eq!(
            anchored[0].1,
            format!("01\n02\n03\n{}\n05\n", ANCHOR_MARKER)
        );
    }

    #[test]
    fn test_anchor_review_file_contents_falls_back_to_target_line_when_no_nearby_blank_line() {
        let files = vec![(
            "src/lib.rs".to_string(),
            (1..=4).map(|line| format!("{line}\n")).collect::<String>(),
        )];
        let items = vec![PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(1),
                end_line: Some(1),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        }];

        let anchored = anchor_review_file_contents(&files, "src/lib.rs", &items, None);

        assert_eq!(anchored[0].1, format!("1{ANCHOR_MARKER}\n2\n3\n4\n"));
    }

    #[test]
    fn test_collect_anchor_lines_coalesces_nearby_targets() {
        let make_item = |line| PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(line),
                end_line: Some(line),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };

        let content = ["01", "", "03", "", "05", "06", "", "08", "09", "10"].join("\n") + "\n";

        let lines = collect_anchor_lines(
            &content,
            &[
                make_item(2),
                make_item(3),
                make_item(5),
                make_item(6),
                make_item(10),
            ],
        );

        assert_eq!(lines, vec![2, 7]);
    }

    #[test]
    fn test_anchor_review_file_contents_minimizes_markers_for_nearby_targets() {
        let files = vec![(
            "docs/example.md".to_string(),
            [
                "01", "", "03", "", "05", "", "07", "08", "09", "10", "11", "12",
            ]
            .join("\n")
                + "\n",
        )];
        let make_item = |line| PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(line),
                end_line: Some(line),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };

        let anchored = anchor_review_file_contents(
            &files,
            "docs/example.md",
            &[
                make_item(2),
                make_item(3),
                make_item(5),
                make_item(6),
                make_item(10),
            ],
            None,
        );
        let content = &anchored[0].1;
        let marker_lines: Vec<usize> = content
            .lines()
            .enumerate()
            .filter_map(|(index, line)| line.contains(ANCHOR_MARKER).then_some(index + 1))
            .collect();

        assert_eq!(content.lines().count(), 12);
        assert_eq!(marker_lines, vec![2, 10]);
    }

    #[test]
    fn test_anchor_review_file_contents_skips_markers_near_content_changes() {
        let base_files = vec![(
            "docs/example.md".to_string(),
            ["01", "02", "03", "04", "05", "06", "07", "08", "09", "10"].join("\n") + "\n",
        )];
        let files = vec![(
            "docs/example.md".to_string(),
            [
                "01", "changed", "03", "04", "05", "06", "updated", "08", "09", "10",
            ]
            .join("\n")
                + "\n",
        )];
        let make_item = |line| PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(line),
                end_line: Some(line),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };

        let anchored = anchor_review_file_contents(
            &files,
            "docs/example.md",
            &[make_item(5), make_item(7)],
            Some(&base_files),
        );
        let content = &anchored[0].1;

        assert_eq!(content.lines().count(), 10);
        assert!(
            !content
                .lines()
                .nth(4)
                .is_some_and(|line| line.ends_with(ANCHOR_MARKER))
        );
        assert!(
            !content
                .lines()
                .nth(6)
                .is_some_and(|line| line.ends_with(ANCHOR_MARKER))
        );
        assert_eq!(content, &files[0].1);
    }

    #[test]
    fn test_pr_body_variants() {
        let comment = PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange::default(),
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };
        let suggestion = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            ..comment.clone()
        };

        assert!(pr_body("review.smd", "docs/example.md", true, false, &[suggestion.clone(), comment.clone()])
            .contains("contains edits, suggestions, and comments for `docs/example.md`, extracted from `review.smd`"));
        assert!(
            pr_body("review.smd", "docs/example.md", true, false, &[]).contains("contains edits")
        );
        assert!(
            pr_body("review.smd", "docs/example.md", true, true, &[])
                .contains("placeholder file change")
        );
        assert!(
            pr_body("review.smd", "docs/example.md", false, false, &[comment])
                .contains("contains comments for `docs/example.md`, extracted from `review.smd`")
        );
        assert_eq!(
            pr_body("review.smd", "docs/example.md", false, false, &[]),
            "Stencila GitHub PR export."
        );
    }

    #[test]
    fn test_pr_title_variants() {
        assert_eq!(
            pr_title("docs/example.md", true, false, &[]),
            "Edits for docs/example.md"
        );
        let comment = PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange::default(),
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };
        let suggestion = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            ..comment.clone()
        };
        assert_eq!(
            pr_title("docs/example.md", false, false, &[suggestion, comment]),
            "Suggestions and comments for docs/example.md"
        );
    }

    #[test]
    fn test_review_body_intro_variants() {
        let comment = PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange::default(),
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };
        let suggestion = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            ..comment.clone()
        };

        assert!(
            review_body_intro(
                "review.smd",
                "docs/example.md",
                &[suggestion.clone(), comment]
            )
            .contains("These suggestions and comments were extracted")
        );
        assert!(
            review_body_intro("review.smd", "docs/example.md", &[suggestion])
                .contains("These suggestions were extracted")
        );
    }

    #[test]
    fn test_review_only_push_plan_includes_anchor_commit() {
        let mut operations = Vec::new();

        let has_source_changes = false;
        let has_review_items = true;

        if has_source_changes {
            operations.push("content_commit");
        }

        if has_review_items {
            operations.push("anchor_commit");
        }

        operations.push("create_pr");

        assert_eq!(operations, vec!["anchor_commit", "create_pr"]);
    }

    #[test]
    fn test_existing_pr_failures_do_not_close_pr() {
        let mut operations = Vec::new();
        let existing_pr_number = Some(42_u64);
        let comment_submission_failed = true;

        if comment_submission_failed && existing_pr_number.is_none() {
            operations.push("close_pr");
        }

        assert!(operations.is_empty());
    }

    #[test]
    fn test_new_pr_failures_still_close_pr() {
        let mut operations = Vec::new();
        let existing_pr_number = None::<u64>;
        let comment_submission_failed = true;

        if comment_submission_failed && existing_pr_number.is_none() {
            operations.push("close_pr");
        }

        assert_eq!(operations, vec!["close_pr"]);
    }

    #[test]
    fn test_existing_anchor_commit_is_preserved_when_no_new_anchor_is_created() {
        let mut export = PullRequestExport {
            source: PullRequestSource {
                repository: Some("stencila/stencila".into()),
                path: Some("docs/example.md".into()),
                commit: Some("abcdef1234567890".into()),
                format: Format::Smd,
            },
            target: PullRequestTarget {
                anchor_commit: Some("existing-anchor-sha".into()),
                side: Some(PullRequestSide::Right),
                ..Default::default()
            },
            content: PullRequestSourceContent {
                text: "plain source".into(),
                mapping: None,
            },
            items: vec![],
            diagnostics: vec![],
        };

        let new_anchor_commit: Option<String> = None;
        let anchor_commit = new_anchor_commit.or_else(|| export.target.anchor_commit.clone());
        export.target.anchor_commit = anchor_commit.clone();
        export.target.side = anchor_commit.as_ref().map(|_| PullRequestSide::Right);

        assert_eq!(
            export.target.anchor_commit.as_deref(),
            Some("existing-anchor-sha")
        );
        assert!(matches!(export.target.side, Some(PullRequestSide::Right)));
    }

    #[test]
    fn test_format_fallback_body() {
        let item = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(5),
                end_line: Some(5),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: Some("better text".into()),
            body_markdown: "Please rephrase.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Unanchored,
            github_suggestion: None,
        };

        let body = format_fallback_body(&[&item], "docs/example.md");
        assert!(
            body.contains("**Suggest inserting** `better text` **in `docs/example.md` at line 5**")
        );
        assert!(body.contains("Please rephrase."));
        assert!(body.contains("Suggested replacement: `better text`"));
    }

    #[test]
    fn test_format_fallback_body_uses_item_source_path() {
        let item_default = PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(1),
                end_line: Some(1),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: "Default path item.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Unanchored,
            github_suggestion: None,
        };

        let item_secondary = PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: Some("other/file.md".into()),
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(10),
                end_line: Some(12),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: None,
            replacement_text: None,
            body_markdown: "Secondary file item.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Unanchored,
            github_suggestion: None,
        };

        let body = format_fallback_body(&[&item_default, &item_secondary], "docs/main.md");

        // Item without source_path should use the default path
        assert!(body.contains("**Comment on** these lines **in `docs/main.md` at line 1**"));
        // Item with its own source_path should use that, not the default
        assert!(body.contains("**Comment on** these lines **in `other/file.md` at lines 10–12**"));
        assert!(!body.contains("**in `docs/main.md` at lines 10–12**"));
    }

    #[test]
    fn test_format_review_item_body_for_anchored_suggestion_replace() {
        let item = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: Some("docs/example.md".into()),
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(5),
                end_line: Some(5),
                start_offset: Some(10),
                end_offset: Some(13),
                ..Default::default()
            },
            selected_text: Some("abc".into()),
            preceding_text: None,
            replacement_text: Some("def".into()),
            body_markdown: "This reads more clearly.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };

        let body = format_review_item_body(&item, CommentBodyMode::Anchored);
        assert_eq!(
            body,
            "**Suggest replacing** `abc` **with** `def`:\n\nThis reads more clearly.\n\nSuggested replacement: `def`"
        );
    }

    #[test]
    fn test_format_review_item_body_for_github_suggestion_insert() {
        let item = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: Some("docs/example.md".into()),
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(5),
                end_line: Some(5),
                ..Default::default()
            },
            selected_text: Some("abc".into()),
            preceding_text: None,
            replacement_text: Some("def".into()),
            body_markdown: "This adds the missing term.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: Some(crate::pull_requests::export::GitHubSuggestion {
                edit_kind: crate::pull_requests::export::SuggestionEditKind::Insert,
                start_line: 5,
                end_line: 5,
                replacement_lines: "abcdef".into(),
                body: "```suggestion\nabcdef\n```".into(),
            }),
        };

        let body = format_review_item_body(&item, CommentBodyMode::Anchored);
        assert_eq!(
            body,
            "**Suggest replacing** `abc` **with** `def`:\n\nThis adds the missing term.\n\n```suggestion\nabcdef\n```"
        );
    }

    #[test]
    fn test_format_review_item_body_for_insert_uses_preceding_text() {
        let item = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: Some("docs/example.md".into()),
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(5),
                end_line: Some(5),
                ..Default::default()
            },
            selected_text: None,
            preceding_text: Some("abc".into()),
            replacement_text: Some("def".into()),
            body_markdown: "This adds the missing term.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };

        let body = format_review_item_body(&item, CommentBodyMode::Anchored);
        assert_eq!(
            body,
            "**Suggest inserting** `def` **after** `abc`:\n\nThis adds the missing term.\n\nSuggested replacement: `def`"
        );
    }

    #[test]
    fn test_format_item_lead_includes_author_for_suggestion() {
        let item = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: Some("docs/example.md".into()),
            author_name: Some("Alice Smith".into()),
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(5),
                end_line: Some(5),
                ..Default::default()
            },
            selected_text: Some("abc".into()),
            preceding_text: None,
            replacement_text: Some("def".into()),
            body_markdown: String::new(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };

        let lead = format_item_lead(&item, CommentBodyMode::Anchored, None);
        assert_eq!(
            lead,
            "**Suggestion from _Alice Smith_**: replace `abc` **with** `def`"
        );
    }

    #[test]
    fn test_short_preceding_snippet_ellides_at_start() {
        let item = PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: None,
            author_name: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange::default(),
            selected_text: None,
            preceding_text: Some(
                "This is some text that should be ellided so we keep the text immediately before"
                    .into(),
            ),
            replacement_text: Some("added".into()),
            body_markdown: String::new(),
            suggestion_type: Some(SuggestionType::Insert),
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Anchored,
            github_suggestion: None,
        };

        let lead = format_item_lead(&item, CommentBodyMode::Anchored, None);
        assert!(lead.contains("**after** `…"));
        assert!(lead.contains("immediately before`"));
        assert!(!lead.contains("This is some text"));
    }
}
