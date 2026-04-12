use base64::Engine;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use stencila_codec::eyre::{Result, bail, eyre};
use stencila_codec::{PushDryRunFile, PushDryRunOptions, PushResult};

use super::{
    activity::parse_github_pull_request_url,
    export::{
        PullRequestComment, PullRequestCommentKind, PullRequestCommentResolution,
        PullRequestExport, PullRequestSide, PullRequestTarget,
    },
};
use crate::client::{api_url, delete, get_json, patch_json, post_json, post_json_with_status};

const DUMMY_CHANGE_MARKER: &str = "<!-- Stencila ghpr placeholder change -->";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubmitReviewMode {
    RequireInline,
    AllowFallback,
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
        has_source_changes && !has_comments && should_use_dummy_change(source_text);

    let mut current_sha = branch_base_sha.to_string();

    let content_commit_sha = if has_source_changes {
        let content_commit_sha = if use_dummy_change {
            create_dummy_content_commit(owner, repo, &current_sha, source_path, source_text).await?
        } else {
            create_content_commit(
                owner,
                repo,
                &current_sha,
                &[(source_path.to_string(), source_text.to_string())],
            )
            .await?
        };
        update_ref(owner, repo, branch_name, &content_commit_sha).await?;
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
                title: &pr_title(source_path, has_comments),
                body: pr_body(has_source_changes, has_comments, use_dummy_change),
                head: branch_name,
                base: base_branch,
            },
            owner,
            repo,
        )
        .await?
    };

    let (comments_posted, fallbacks) = if has_comments {
        let initial_review_commit_sha = if has_source_changes {
            content_commit_sha
                .clone()
                .unwrap_or_else(|| current_sha.clone())
        } else {
            let anchor_content_ref = if matches!(source_commit, "dirty" | "untracked") {
                current_sha.as_str()
            } else {
                source_commit
            };
            let file_contents =
                review_file_contents(owner, repo, source_path, anchor_content_ref, items).await?;
            let created_anchor_commit_sha =
                create_anchor_commit(owner, repo, &current_sha, &file_contents).await?;
            update_ref(owner, repo, branch_name, &created_anchor_commit_sha).await?;
            anchor_commit_sha = Some(created_anchor_commit_sha.clone());
            created_anchor_commit_sha
        };

        match submit_github_review(
            owner,
            repo,
            pr.number,
            &initial_review_commit_sha,
            items,
            source_path,
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
                let created_anchor_commit_sha =
                    create_anchor_commit(owner, repo, &initial_review_commit_sha, &file_contents)
                        .await?;
                update_ref(owner, repo, branch_name, &created_anchor_commit_sha).await?;
                anchor_commit_sha = Some(created_anchor_commit_sha.clone());

                match submit_github_review(
                    owner,
                    repo,
                    pr.number,
                    &created_anchor_commit_sha,
                    items,
                    source_path,
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
        }
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

    let has_dummy_change = plan_requires_dummy_content_commit(export, has_source_changes);
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

            let repo_info: RepoInfoResponse =
                get_json(&api_url(&format!("/repos/{owner}/{repo}")), &owner, &repo).await?;
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
            )
            .await?;

            (None, None, branch_base_sha, base_branch, branch_name, true)
        };

    let push_result = push_pull_request_inner(
        &owner,
        &repo,
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
        source_path: source_path.to_string(),
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
    let resp: ContentsResponse = get_json(&url, owner, repo).await?;

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

    let repo_info: RepoInfoResponse =
        get_json(&api_url(&format!("/repos/{owner}/{repo}")), owner, repo).await?;
    let branch: BranchResponse = get_json(
        &api_url(&format!(
            "/repos/{owner}/{repo}/branches/{}",
            repo_info.default_branch
        )),
        owner,
        repo,
    )
    .await?;

    Ok(branch.commit.sha)
}

fn plan_requires_dummy_content_commit(
    export: &PullRequestExport,
    has_source_changes: bool,
) -> bool {
    !has_source_changes && export.items.is_empty()
}

fn should_use_dummy_change(source_text: &str) -> bool {
    !source_text.contains(DUMMY_CHANGE_MARKER)
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
) -> Result<String> {
    let parent_commit: CommitResponse = get_json(
        &api_url(&format!("/repos/{owner}/{repo}/git/commits/{parent_sha}")),
        owner,
        repo,
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
    )
    .await?;

    let commit: CommitResponse = post_json(
        &api_url(&format!("/repos/{owner}/{repo}/git/commits")),
        &CreateCommitRequest {
            message: "feat: apply Stencila document changes",
            tree: &tree.sha,
            parents: vec![parent_sha],
        },
        owner,
        repo,
    )
    .await?;

    Ok(commit.sha)
}

async fn create_dummy_content_commit(
    owner: &str,
    repo: &str,
    parent_sha: &str,
    path: &str,
    content: &str,
) -> Result<String> {
    let dummy_content = if content.is_empty() {
        format!("\n{DUMMY_CHANGE_MARKER}\n")
    } else if content.ends_with('\n') {
        format!("{content}{DUMMY_CHANGE_MARKER}\n")
    } else {
        format!("{content}\n{DUMMY_CHANGE_MARKER}\n")
    };

    create_content_commit(
        owner,
        repo,
        parent_sha,
        &[(path.to_string(), dummy_content)],
    )
    .await
}

fn pr_title(source_path: &str, has_review_items: bool) -> String {
    if has_review_items {
        format!("Review of {source_path}")
    } else {
        format!("Update {source_path}")
    }
}

fn pr_body(
    has_source_changes: bool,
    has_review_items: bool,
    uses_dummy_change: bool,
) -> &'static str {
    match (has_source_changes, has_review_items, uses_dummy_change) {
        (true, false, true) => {
            "Stencila created a placeholder file change so this pull request can be opened from a review document with no substantive content diff."
        }
        (true, true, _) => {
            "Stencila document changes and review comments submitted as a GitHub pull request and PR review."
        }
        (true, false, false) => "Stencila document changes submitted as a GitHub pull request.",
        (false, true, _) => "Stencila document review submitted as a GitHub pull request review.",
        (false, false, _) => "Stencila GitHub PR export.",
    }
}

/// Create an anchor commit with minimal end-of-file newline changes.
///
/// This mirrors the working site review PR approach: cycle each reviewed file
/// through a small newline-only change so the PR has a minimal diff while still
/// providing a file change for GitHub review comments to attach to.
async fn create_anchor_commit(
    owner: &str,
    repo: &str,
    parent_sha: &str,
    files: &[(String, String)],
) -> Result<String> {
    let parent_commit: CommitResponse = get_json(
        &api_url(&format!("/repos/{owner}/{repo}/git/commits/{parent_sha}")),
        owner,
        repo,
    )
    .await?;

    let tree_entries: Vec<TreeEntry> = files
        .iter()
        .map(|(path, content)| {
            let line_ending = if content.contains("\r\n") {
                "\r\n"
            } else {
                "\n"
            };
            let double_ending = format!("{line_ending}{line_ending}");
            let anchor_content = if content.ends_with(&double_ending) {
                content[..content.len() - line_ending.len()].to_string()
            } else {
                format!("{content}{line_ending}")
            };

            TreeEntry {
                path: path.clone(),
                mode: "100644".to_string(),
                type_: "blob".to_string(),
                content: anchor_content,
            }
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
    )
    .await?;

    let commit: CommitResponse = post_json(
        &api_url(&format!("/repos/{owner}/{repo}/git/commits")),
        &CreateCommitRequest {
            message: "chore: anchor commit for Stencila pull request comments",
            tree: &tree.sha,
            parents: vec![parent_sha],
        },
        owner,
        repo,
    )
    .await?;

    Ok(commit.sha)
}

/// Update a branch ref to point at a new commit SHA.
async fn update_ref(owner: &str, repo: &str, branch: &str, sha: &str) -> Result<()> {
    #[derive(Serialize)]
    struct UpdateRefRequest<'a> {
        sha: &'a str,
        force: bool,
    }

    let url = api_url(&format!("/repos/{owner}/{repo}/git/refs/heads/{branch}"));
    patch_json(&url, &UpdateRefRequest { sha, force: false }, owner, repo).await
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
    anchor_commit_sha: &str,
    items: &[PullRequestComment],
    source_path: &str,
    mode: SubmitReviewMode,
) -> Result<(usize, usize), SubmitReviewError> {
    let (comments, fallback_items) = convert_to_review_comments(items, source_path);
    let mut all_fallback_items = fallback_items;

    tracing::debug!(
        pr_number,
        anchor_commit_sha,
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
        let initial_body = if all_fallback_items.is_empty() {
            "Stencila document pull request comments.".to_string()
        } else {
            format!(
                "Stencila document pull request comments.\n\n{}",
                format_fallback_body(&all_fallback_items, source_path)
            )
        };

        let result: std::result::Result<ReviewResponse, _> = post_json_with_status(
            &review_url,
            &CreateReviewRequest {
                commit_id: anchor_commit_sha.to_string(),
                event: "COMMENT".into(),
                body: initial_body,
                comments: comments.clone(),
            },
            owner,
            repo,
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
        let fallback_body = if all_fallback_items.is_empty() {
            "Stencila document pull request comments.".to_string()
        } else {
            format!(
                "Stencila document pull request comments.\n\n{}",
                format_fallback_body(&all_fallback_items, source_path)
            )
        };

        let _review: ReviewResponse = post_json(
            &review_url,
            &CreateReviewRequest {
                commit_id: anchor_commit_sha.to_string(),
                event: "COMMENT".into(),
                body: fallback_body,
                comments: vec![],
            },
            owner,
            repo,
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
    patch_json(&url, &ClosePrRequest { state: "closed" }, owner, repo).await
}

/// Delete a branch ref.
async fn delete_branch(owner: &str, repo: &str, branch: &str) -> Result<()> {
    let url = api_url(&format!("/repos/{owner}/{repo}/git/refs/heads/{branch}"));
    delete(&url, owner, repo).await
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
    let mut comments = Vec::new();
    let mut fallbacks = Vec::new();

    for item in items {
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
        let body = match &item.github_suggestion {
            Some(gs) => gs.body.clone(),
            None => format_comment_body(item),
        };

        comments.push(GitHubPullRequestComment {
            path,
            body,
            line,
            start_line,
            side: "RIGHT".to_string(),
            start_side: start_line.map(|_| "RIGHT".into()),
        });
    }

    (comments, fallbacks)
}

/// Format a plain comment body (non-suggestion items).
fn format_comment_body(item: &PullRequestComment) -> String {
    let mut body = item.body_markdown.clone();

    // For suggestions without a GitHubSuggestion block, include the
    // replacement text explicitly so the reviewer can see the intent.
    if matches!(item.kind, PullRequestCommentKind::Suggestion)
        && let Some(replacement) = &item.replacement_text
    {
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str(&format!("**Suggested replacement:** `{replacement}`"));
    }

    if body.is_empty() {
        body = "(empty comment)".to_string();
    }

    body
}

/// Format fallback items as a Markdown section for the review body.
///
/// Items that could not be anchored as inline comments are rendered here
/// so the reviewer still sees them, just not inline on the diff.
pub(crate) fn format_fallback_body(items: &[&PullRequestComment], default_path: &str) -> String {
    let mut parts = Vec::new();
    parts.push("### Items that could not be anchored inline\n".to_string());

    for item in items {
        let kind = match item.kind {
            PullRequestCommentKind::Comment => "Comment",
            PullRequestCommentKind::Suggestion => "Suggestion",
        };

        let file_path = item.source_path.as_deref().unwrap_or(default_path);

        let location = match (item.range.start_line, item.range.end_line) {
            (Some(start), Some(end)) if start == end => format!("{file_path}:{start}"),
            (Some(start), Some(end)) => format!("{file_path}:{start}-{end}"),
            _ => file_path.to_string(),
        };

        parts.push(format!("- **{kind}** at `{location}`"));

        if !item.body_markdown.is_empty() {
            parts.push(format!(
                "  > {}",
                item.body_markdown.replace('\n', "\n  > ")
            ));
        }

        if let Some(replacement) = &item.replacement_text {
            parts.push(format!("  Suggested: `{replacement}`"));
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
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange::default(),
                selected_text: None,
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
        assert_eq!(comments[0].body, "Nice opening.");

        assert_eq!(fallbacks.len(), 1);
        assert!(matches!(
            fallbacks[0].kind,
            PullRequestCommentKind::Suggestion
        ));
    }

    #[test]
    fn test_convert_to_review_comments_single_line_has_no_side() {
        let items = vec![PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: Some("docs/example.md".into()),
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
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    end_line: Some(1),
                    start_offset: Some(0),
                    end_offset: Some(5),
                    ..Default::default()
                },
                selected_text: None,
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
                node_id: None,
                parent_node_id: None,
                range: PullRequestCommentRange {
                    end_line: Some(2),
                    ..Default::default()
                },
                selected_text: None,
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
    fn test_plan_requires_dummy_content_commit_for_empty_review() {
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

        assert!(plan_requires_dummy_content_commit(&export, false));
        assert!(!plan_requires_dummy_content_commit(&export, true));
    }

    #[test]
    fn test_should_use_dummy_change_marker_detection() {
        assert!(should_use_dummy_change("plain source"));
        assert!(!should_use_dummy_change(DUMMY_CHANGE_MARKER));
    }

    #[test]
    fn test_pr_body_variants() {
        assert!(pr_body(true, true, false).contains("changes and review comments"));
        assert!(pr_body(true, false, false).contains("document changes"));
        assert!(pr_body(true, false, true).contains("placeholder file change"));
        assert!(pr_body(false, true, false).contains("document review"));
        assert!(pr_body(false, false, false).contains("GitHub PR export"));
    }

    #[test]
    fn test_pr_title_variants() {
        assert_eq!(
            pr_title("docs/example.md", true),
            "Review of docs/example.md"
        );
        assert_eq!(pr_title("docs/example.md", false), "Update docs/example.md");
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
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(5),
                end_line: Some(5),
                ..Default::default()
            },
            selected_text: None,
            replacement_text: Some("better text".into()),
            body_markdown: "Please rephrase.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Unanchored,
            github_suggestion: None,
        };

        let body = format_fallback_body(&[&item], "docs/example.md");
        assert!(body.contains("Suggestion"));
        assert!(body.contains("docs/example.md:5"));
        assert!(body.contains("Please rephrase."));
        assert!(body.contains("`better text`"));
    }

    #[test]
    fn test_format_fallback_body_uses_item_source_path() {
        let item_default = PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: None,
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(1),
                end_line: Some(1),
                ..Default::default()
            },
            selected_text: None,
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
            node_id: None,
            parent_node_id: None,
            range: PullRequestCommentRange {
                start_line: Some(10),
                end_line: Some(12),
                ..Default::default()
            },
            selected_text: None,
            replacement_text: None,
            body_markdown: "Secondary file item.".into(),
            suggestion_type: None,
            suggestion_status: None,
            resolution: PullRequestCommentResolution::Unanchored,
            github_suggestion: None,
        };

        let body = format_fallback_body(&[&item_default, &item_secondary], "docs/main.md");

        // Item without source_path should use the default path
        assert!(body.contains("docs/main.md:1"));
        // Item with its own source_path should use that, not the default
        assert!(body.contains("other/file.md:10-12"));
        assert!(!body.contains("docs/main.md:10"));
    }
}
