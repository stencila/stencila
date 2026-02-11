//! System prompt helpers: environment context, git context, and full prompt
//! assembly (spec 6.1, 6.3-6.4).
//!
//! These helpers are called by the session layer to produce pre-formatted
//! strings that get passed to [`ProviderProfile::build_system_prompt()`].

use crate::error::AgentResult;
use crate::execution::ExecutionEnvironment;
use crate::profile::ProviderProfile;

// ---------------------------------------------------------------------------
// GitContext (spec 6.4)
// ---------------------------------------------------------------------------

/// Snapshot of git state at session start (spec 6.4).
///
/// The model can always run `git status`, `git diff`, etc. via the shell
/// tool for the latest state. This snapshot provides initial orientation.
#[derive(Debug, Clone, Default)]
pub struct GitContext {
    /// Whether the working directory is inside a git repository.
    pub is_repo: bool,
    /// Current branch name (empty if not a repo or detached HEAD).
    pub branch: String,
    /// Number of modified/staged files.
    pub modified_count: usize,
    /// Number of untracked files.
    pub untracked_count: usize,
    /// Recent commit subject lines (most recent first).
    pub recent_commits: Vec<String>,
}

/// Gather git context from the execution environment (spec 6.4).
///
/// Runs short git commands via `exec_command` with a 5-second timeout.
/// Returns a default (non-repo) context on any failure — git context
/// is best-effort, never blocks session startup.
pub async fn gather_git_context(env: &dyn ExecutionEnvironment) -> GitContext {
    let mut ctx = GitContext::default();

    // Check if we're in a git repo by running `git rev-parse`
    let branch_result = env
        .exec_command("git rev-parse --abbrev-ref HEAD", 5_000, None, None)
        .await;

    let branch = match branch_result {
        Ok(ref r) if r.exit_code == 0 => r.stdout.trim().to_string(),
        _ => return ctx,
    };

    ctx.is_repo = true;
    ctx.branch = branch;

    // Short status: count modified and untracked files
    if let Ok(r) = env
        .exec_command("git status --short", 5_000, None, None)
        .await
        && r.exit_code == 0
    {
        for line in r.stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("??") {
                ctx.untracked_count += 1;
            } else if !trimmed.is_empty() {
                ctx.modified_count += 1;
            }
        }
    }

    // Recent commits (last 10)
    if let Ok(r) = env
        .exec_command("git log --oneline -10", 5_000, None, None)
        .await
        && r.exit_code == 0
    {
        ctx.recent_commits = r
            .stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
    }

    ctx
}

/// Find the git repository root directory.
///
/// Returns `None` if not in a git repo or the command fails.
pub async fn find_git_root(env: &dyn ExecutionEnvironment) -> Option<String> {
    let result = env
        .exec_command("git rev-parse --show-toplevel", 5_000, None, None)
        .await
        .ok()?;

    if result.exit_code == 0 {
        let root = result.stdout.trim().to_string();
        if !root.is_empty() {
            return Some(root);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Environment context block (spec 6.3)
// ---------------------------------------------------------------------------

/// Build the environment context block (spec 6.3).
///
/// Returns a structured `<environment>` XML block containing runtime
/// information. This block is generated at session start and included
/// in every system prompt.
pub fn build_environment_context(
    working_dir: &str,
    platform: &str,
    os_version: &str,
    model: &str,
    knowledge_cutoff: Option<&str>,
    git_context: &GitContext,
) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d");

    let mut lines = vec![
        "<environment>".to_string(),
        format!("Working directory: {working_dir}"),
        format!("Is git repository: {}", git_context.is_repo),
    ];

    if git_context.is_repo {
        lines.push(format!("Git branch: {}", git_context.branch));

        // Include status counts (spec 6.4)
        if git_context.modified_count > 0 || git_context.untracked_count > 0 {
            let mut status_parts = Vec::new();
            if git_context.modified_count > 0 {
                status_parts.push(format!("{} modified/staged", git_context.modified_count));
            }
            if git_context.untracked_count > 0 {
                status_parts.push(format!("{} untracked", git_context.untracked_count));
            }
            lines.push(format!("Git status: {}", status_parts.join(", ")));
        }

        // Include recent commits (spec 6.4)
        if !git_context.recent_commits.is_empty() {
            lines.push("Recent commits:".to_string());
            for commit in &git_context.recent_commits {
                lines.push(format!("  {commit}"));
            }
        }
    }

    lines.push(format!("Platform: {platform}"));
    lines.push(format!("OS version: {os_version}"));
    lines.push(format!("Today's date: {today}"));
    lines.push(format!("Model: {model}"));
    if let Some(cutoff) = knowledge_cutoff {
        lines.push(format!("Knowledge cutoff: {cutoff}"));
    }
    lines.push("</environment>".to_string());

    lines.join("\n")
}

/// Build the full environment context from an execution environment.
///
/// Convenience wrapper that gathers git context and calls
/// [`build_environment_context`]. The `knowledge_cutoff` is model-specific
/// and should be provided by the session layer (e.g. from model catalog).
pub async fn build_environment_context_from_env(
    env: &dyn ExecutionEnvironment,
    model: &str,
    knowledge_cutoff: Option<&str>,
) -> String {
    let git_context = gather_git_context(env).await;
    build_environment_context(
        env.working_directory(),
        env.platform(),
        &env.os_version(),
        model,
        knowledge_cutoff,
        &git_context,
    )
}

/// Format git context as a summary block for inclusion in the system prompt.
///
/// Returns an empty string if not in a git repo.
pub fn format_git_summary(git_context: &GitContext) -> String {
    if !git_context.is_repo {
        return String::new();
    }

    let mut parts = vec![format!("Git branch: {}", git_context.branch)];

    if git_context.modified_count > 0 || git_context.untracked_count > 0 {
        let mut status_parts = Vec::new();
        if git_context.modified_count > 0 {
            status_parts.push(format!(
                "{} modified/staged file{}",
                git_context.modified_count,
                if git_context.modified_count == 1 {
                    ""
                } else {
                    "s"
                }
            ));
        }
        if git_context.untracked_count > 0 {
            status_parts.push(format!(
                "{} untracked file{}",
                git_context.untracked_count,
                if git_context.untracked_count == 1 {
                    ""
                } else {
                    "s"
                }
            ));
        }
        parts.push(format!("Status: {}", status_parts.join(", ")));
    }

    if !git_context.recent_commits.is_empty() {
        parts.push("Recent commits:".to_string());
        for commit in &git_context.recent_commits {
            parts.push(format!("  {commit}"));
        }
    }

    parts.join("\n")
}

// ---------------------------------------------------------------------------
// Full prompt assembly (spec 6.1)
// ---------------------------------------------------------------------------

/// Build the system prompt for a session by gathering environment context,
/// project docs, and optionally workspace skills.
///
/// Convenience async function that combines profile, environment context,
/// project doc discovery, and skill metadata into a complete system prompt
/// string. When `enable_skills` is true and the `skills` feature is active,
/// discovered skills are included as metadata and the `use_skill` tool is
/// registered in the profile.
pub async fn build_system_prompt(
    profile: &mut dyn ProviderProfile,
    env: &dyn ExecutionEnvironment,
    enable_skills: bool,
) -> AgentResult<String> {
    let env_context = build_environment_context_from_env(env, profile.model(), None).await;

    let git_root = find_git_root(env).await;
    let working_dir = env.working_directory();
    let root = git_root.as_deref().unwrap_or(working_dir);

    let project_docs =
        crate::project_docs::discover_project_docs(env, profile.id(), root, working_dir).await?;

    #[allow(unused_mut)]
    let mut prompt = profile.build_system_prompt(&env_context, &project_docs);

    // Skills layer (between project docs and user instructions).
    // Gated at both compile time (feature) and runtime (config flag).
    #[cfg(feature = "skills")]
    if enable_skills {
        let skills_metadata =
            crate::skills::discover_and_register_skills(profile, working_dir).await;
        if !skills_metadata.is_empty() {
            prompt.push_str("\n\n");
            prompt.push_str(&skills_metadata);
        }
    }

    // Suppress unused variable warning when feature is off.
    #[cfg(not(feature = "skills"))]
    let _ = enable_skills;

    Ok(prompt)
}
