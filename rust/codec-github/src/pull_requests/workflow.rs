use std::path::{Path, PathBuf};

use stencila_codec::{
    Codec, EncodeInfo, EncodeOptions, PushDryRunOptions, PushResult,
    eyre::{Result, bail, eyre},
    stencila_format::Format,
    stencila_schema::Node,
};
use stencila_codec_csv::CsvCodec;
use stencila_codec_ipynb::IpynbCodec;
use stencila_codec_latex::LatexCodec;
use stencila_codec_markdown::MarkdownCodec;

use super::{
    export::{PullRequestExport, export_pull_request},
    push::{
        plan_pull_request_push_with_source_changes, pull_request_push_dry_run,
        push_pull_request_export,
    },
    source::pull_request_source,
};

pub(crate) fn infer_source_changes(
    local_path: &Path,
    git_info: &LocalGitFileInfo,
    export: &PullRequestExport,
    encoded_source_text: &str,
) -> Result<bool> {
    let working_tree_text = std::fs::read_to_string(local_path).unwrap_or_default();

    if matches!(
        git_info.commit.as_deref(),
        Some("dirty") | Some("untracked")
    ) {
        return Ok(true);
    }

    if working_tree_text != encoded_source_text {
        return Ok(true);
    }

    let Some(source_commit) = export.source.commit.as_deref() else {
        return Ok(true);
    };

    let Some(source_path) = export.source.path.as_deref() else {
        return Ok(true);
    };

    let repo_root = closest_git_repo_local(local_path)?;
    let committed_source = git_show_file(&repo_root, source_commit, source_path)?;
    Ok(committed_source != encoded_source_text)
}

fn default_ghpr_source_path(local_path: &Path) -> String {
    let stem = local_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("document");

    format!("{stem}.md")
}

pub(crate) fn prefers_default_ghpr_source_path(git_info: &LocalGitFileInfo) -> bool {
    matches!(git_info.commit.as_deref(), Some("untracked"))
}

fn git_show_file(repo_root: &Path, git_ref: &str, path: &str) -> Result<String> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["show", &format!("{git_ref}:{path}")])
        .output()?;

    if !output.status.success() {
        return Ok(String::new());
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub(crate) async fn encode_source_text(
    node: &Node,
    path: &Path,
    source: &GhprSource,
) -> Result<String> {
    let options = Some(EncodeOptions {
        format: Some(source.format.clone()),
        from_path: Some(path.to_path_buf()),
        ..Default::default()
    });

    let (text, _info): (String, EncodeInfo) = match source.format {
        Format::Markdown | Format::Myst | Format::Qmd | Format::Smd => {
            MarkdownCodec.to_string(node, options).await?
        }
        Format::Ipynb => IpynbCodec.to_string(node, options).await?,
        Format::Latex => LatexCodec.to_string(node, options).await?,
        Format::Csv | Format::Tsv => CsvCodec.to_string(node, options).await?,
        _ => {
            bail!(
                "`ghpr` currently supports text-based source documents for content commits; got {}",
                source.format.name()
            )
        }
    };

    Ok(text)
}

/// Push a document to GitHub as a repository-native pull request.
///
/// This powers the `ghpr` push target. It is intentionally push-only and does
/// not use the normal tracked-remote model because the source of truth is the
/// Git repository provenance embedded in the document and/or local repository.
pub async fn push_pull_request(
    node: &Node,
    path: Option<&Path>,
    _title: Option<&str>,
    url: Option<&url::Url>,
    dry_run: Option<PushDryRunOptions>,
) -> Result<PushResult> {
    let path = path.ok_or_else(|| eyre!("A local document path is required for `ghpr` pushes"))?;

    let git_info = git_file_info_local(path)?;
    let source = ghpr_source(node, path, &git_info)?;
    let source_text = encode_source_text(node, path, &source).await?;

    let mut export = export_pull_request(node, &source_text, source.format.clone(), None)?;

    if export.source.repository.is_none() {
        export.source.repository = source.repository.clone();
    }
    if export.source.path.is_none() {
        export.source.path = source.path.clone();
    }
    if export.source.commit.is_none() {
        export.source.commit = source.commit.clone();
    }

    let has_source_changes = infer_source_changes(path, &git_info, &export, &source_text)?;
    let plan = plan_pull_request_push_with_source_changes(&export, Some(has_source_changes))?;

    if plan.is_noop() {
        bail!(
            "No GitHub PR actions are needed: the document has neither source changes nor review items"
        );
    }

    if let Some(options) = dry_run.as_ref()
        && options.enabled
    {
        return Ok(pull_request_push_dry_run(&plan, options));
    }

    let result = push_pull_request_export(
        &mut export,
        &path.to_string_lossy(),
        url,
        has_source_changes,
    )
    .await?;
    if source.generated_path {
        tracing::info!(
            source_path = %result.source_path,
            "ghpr push used a generated source path"
        );
    }
    if result.used_dummy_change {
        tracing::info!(
            source_path = %result.source_path,
            "ghpr push used a placeholder change to create a pull request"
        );
    }
    Ok(PushResult::GitHubPullRequest {
        url: url::Url::parse(&result.pr_url)?,
        source_path: result.source_path,
        used_generated_source_path: source.generated_path,
        used_dummy_change: result.used_dummy_change,
        pr_number: result.pr_number,
        pull_request_branch: result.pull_request_branch,
        comments_posted: result.comments_posted,
        fallbacks: result.fallbacks,
    })
}

pub(crate) fn ghpr_source(
    node: &Node,
    local_path: &Path,
    git_info: &LocalGitFileInfo,
) -> Result<GhprSource> {
    let local_format = Format::from_path(local_path);
    let pull_request_source = pull_request_source(node, local_format);
    let mut repository = pull_request_source.repository;
    let mut source_path = pull_request_source.path;
    let mut commit = pull_request_source.commit;

    if repository.is_none() {
        repository = git_info.origin.clone();
    }
    if source_path.is_none() && !prefers_default_ghpr_source_path(git_info) {
        source_path = git_info.path.clone();
    }
    if commit.is_none() {
        commit = git_info.commit.clone();
    }

    let generated_path = source_path.is_none();
    let source_path = source_path.unwrap_or_else(|| default_ghpr_source_path(local_path));
    let source_format = Format::from_path(Path::new(&source_path));

    Ok(GhprSource {
        repository,
        path: Some(source_path),
        commit,
        format: source_format,
        generated_path,
    })
}

pub(crate) fn git_file_info_local(path: &Path) -> Result<LocalGitFileInfo> {
    use std::process::{Command, Stdio};

    let path = path.canonicalize()?;
    let repo_root = closest_git_repo_local(&path)?;
    let relative_path = path
        .strip_prefix(&repo_root)?
        .to_str()
        .ok_or_else(|| eyre!("Path is not valid UTF-8"))?
        .to_string();

    let tracked = Command::new("git")
        .arg("-C")
        .arg(&repo_root)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(&relative_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?
        .success();

    let commit = if tracked {
        let clean = Command::new("git")
            .arg("-C")
            .arg(&repo_root)
            .args(["status", "--porcelain", "--"])
            .arg(&relative_path)
            .output()?
            .stdout
            .is_empty();

        if clean {
            let output = Command::new("git")
                .arg("-C")
                .arg(&repo_root)
                .args(["log", "-1", "--format=%H", "--", &relative_path])
                .output()?;
            if !output.status.success() {
                bail!("Unable to get commit SHA for `{}`", path.display());
            }
            Some(String::from_utf8(output.stdout)?.trim().to_string())
        } else {
            Some("dirty".into())
        }
    } else {
        Some("untracked".into())
    };

    Ok(LocalGitFileInfo {
        origin: get_origin_local(&repo_root),
        path: Some(relative_path),
        commit,
    })
}

fn closest_git_repo_local(path: &Path) -> Result<PathBuf> {
    let mut current_dir = if path.is_file() {
        path.parent()
            .ok_or_else(|| eyre!("File has no parent directory"))?
    } else {
        path
    };

    loop {
        if current_dir.join(".git").exists() {
            return Ok(current_dir.to_path_buf());
        }

        let Some(parent_dir) = current_dir.parent() else {
            break;
        };
        current_dir = parent_dir;
    }

    bail!("Path is not within a Git repository: {}", path.display())
}

fn get_origin_local(repo_root: &Path) -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let origin = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!origin.is_empty()).then_some(origin)
}

pub(crate) struct LocalGitFileInfo {
    pub(crate) origin: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) commit: Option<String>,
}

pub(crate) struct GhprSource {
    pub(crate) repository: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) commit: Option<String>,
    pub(crate) format: Format,
    pub(crate) generated_path: bool,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use stencila_codec::{
        PushDryRunOptions,
        stencila_format::Format,
        stencila_schema::{Article, Node},
    };

    use super::*;
    use crate::pull_requests::{
        export::{PullRequestExport, PullRequestSourceContent, PullRequestTarget},
        source::PullRequestSource,
    };

    #[tokio::test]
    async fn ghpr_push_requires_local_path() {
        let node = Node::Article(Article::default());
        let result = push_pull_request(&node, None, None, None, None).await;
        assert!(result.is_err());
        assert!(
            result
                .expect_err("expected missing path error")
                .to_string()
                .contains("local document path is required")
        );
    }

    #[tokio::test]
    async fn ghpr_push_allows_noop_documents_via_dummy_change() {
        let node = Node::Article(Article::default());
        let path = Path::new("/tmp/document.docx");
        let result = push_pull_request(
            &node,
            Some(path),
            None,
            None,
            Some(PushDryRunOptions {
                enabled: true,
                output_dir: None,
            }),
        )
        .await;
        assert!(result.is_err());
    }

    #[test]
    fn ghpr_source_defaults_new_review_documents_to_markdown_path() {
        let node = Node::Article(Article::default());
        let git_info = LocalGitFileInfo {
            origin: Some("https://github.com/stencila/stencila".into()),
            path: None,
            commit: Some("untracked".into()),
        };

        let source = ghpr_source(&node, Path::new("/tmp/example.docx"), &git_info)
            .expect("should infer default source path");

        assert_eq!(source.path.as_deref(), Some("example.md"));
    }

    #[test]
    fn infer_source_changes_compares_encoded_docx_source_to_provenance_file() {
        let export = PullRequestExport {
            source: PullRequestSource {
                repository: Some("stencila/stencila".into()),
                path: Some("docs/example.md".into()),
                commit: Some("abcdef1234567890".into()),
                format: Format::Docx,
            },
            target: PullRequestTarget::default(),
            content: PullRequestSourceContent {
                text: String::new(),
                mapping: None,
            },
            items: vec![],
            diagnostics: vec![],
        };

        let git_info = LocalGitFileInfo {
            origin: None,
            path: Some("tmp/example.docx".into()),
            commit: Some("abcdef1234567890".into()),
        };

        let changed = infer_source_changes(
            Path::new("/tmp/example.docx"),
            &git_info,
            &export,
            "same markdown source",
        )
        .expect("should infer source changes");
        assert!(changed);
    }

    #[test]
    fn infer_source_changes_treats_dirty_git_file_as_changed() {
        let export = PullRequestExport {
            source: PullRequestSource {
                repository: Some("stencila/stencila".into()),
                path: Some("docs/example.md".into()),
                commit: Some("dirty".into()),
                format: Format::Markdown,
            },
            target: PullRequestTarget::default(),
            content: PullRequestSourceContent {
                text: String::new(),
                mapping: None,
            },
            items: vec![],
            diagnostics: vec![],
        };

        let git_info = LocalGitFileInfo {
            origin: None,
            path: Some("docs/example.md".into()),
            commit: Some("dirty".into()),
        };

        let changed = infer_source_changes(Path::new("/tmp/example.md"), &git_info, &export, "")
            .expect("should infer source changes");
        assert!(changed);
    }
}
