//! Shell pattern packs: `PatternRule`, `Pack`, `Confidence`, and per-domain rule sets.
//!
//! Each pack defines safe and destructive patterns for a specific domain
//! (filesystem, git, database, etc.). All patterns are compiled-in constants.

use std::collections::HashMap;
use std::sync::LazyLock;

use regex::RegexSet;

pub mod bioinformatics;
pub mod chemistry;
pub mod cloud;
pub mod containers;
pub mod core;
pub mod database;
pub mod datasets;
pub mod environments;
pub mod geospatial;
pub mod hpc;
pub mod latex;
pub mod ml;
pub mod notebooks;
pub mod packages;
pub mod scientific;
pub mod system;
pub mod workflows;

// ---------------------------------------------------------------------------
// Helper macros
// ---------------------------------------------------------------------------

/// Tokenize a command string, returning a fallback value on parse failure.
///
/// Used by validators to avoid repeating the `match tokenize(cmd) { ... }`
/// boilerplate. In destructive validators the fallback is `true` (fail-closed:
/// assume the rule fires). In safe validators the fallback is `false`
/// (conservative: not safe on parse failure).
macro_rules! tokenize_or_bail {
    ($cmd:expr, $bail:expr) => {
        match $crate::tool_guard::shell::tokenizer::tokenize($cmd) {
            Ok(t) => t,
            Err(_) => return $bail,
        }
    };
}
pub(crate) use tokenize_or_bail;

use crate::tool_guard::shell::tokenizer::Token;

/// Check if any token matches a specific value.
pub(crate) fn has_token(tokens: &[Token], value: &str) -> bool {
    tokens.iter().any(|t| t.value == value)
}

/// Check if any token starts with a given prefix.
pub(crate) fn has_token_prefix(tokens: &[Token], prefix: &str) -> bool {
    tokens.iter().any(|t| t.value.starts_with(prefix))
}

/// Check if `--dry-run` (or a `--dry-run=...` variant) is present.
pub(crate) fn has_dry_run(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| t.value.starts_with("--dry-run"))
}

/// Declare a safe `PatternRule` with just an id and pattern.
///
/// The generated rule uses the standard form `^command\b[^|><]*$` (provided
/// by the caller) with no validator, empty reason/suggestion, and `High`
/// confidence.
macro_rules! safe_pattern {
    ($id:expr, $pattern:expr) => {
        PatternRule {
            id: $id,
            pattern: $pattern,
            validator: None,
            reason: "",
            suggestion: "",
            confidence: Confidence::High,
        }
    };
    ($id:expr, $pattern:expr, $validator:expr) => {
        PatternRule {
            id: $id,
            pattern: $pattern,
            validator: Some($validator),
            reason: "",
            suggestion: "",
            confidence: Confidence::High,
        }
    };
}

/// Declare a destructive `PatternRule` with id, pattern, reason, suggestion,
/// and confidence — and optionally a validator.
macro_rules! destructive_pattern {
    ($id:expr, $pattern:expr, $reason:expr, $suggestion:expr, $confidence:expr) => {
        PatternRule {
            id: $id,
            pattern: $pattern,
            validator: None,
            reason: $reason,
            suggestion: $suggestion,
            confidence: $confidence,
        }
    };
    ($id:expr, $pattern:expr, $validator:expr, $reason:expr, $suggestion:expr, $confidence:expr) => {
        PatternRule {
            id: $id,
            pattern: $pattern,
            validator: Some($validator),
            reason: $reason,
            suggestion: $suggestion,
            confidence: $confidence,
        }
    };
}
pub(crate) use destructive_pattern;

/// Confidence level for a destructive pattern match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    /// Always denied, even at `trustLevel: high`.
    High,
    /// Denied at low/medium trust, downgraded to `Warn` at high trust.
    Medium,
}

/// A single pattern rule for safe or destructive command matching.
#[derive(Debug)]
pub struct PatternRule {
    /// Unique rule identifier within its pack (e.g., `force_push`).
    pub id: &'static str,
    /// Regex pattern for Phase A candidate scanning.
    pub pattern: &'static str,
    /// Optional Phase B post-match validator. Returns `true` if the rule fires.
    pub validator: Option<fn(&str) -> bool>,
    /// Human-readable reason for the match.
    pub reason: &'static str,
    /// Actionable suggestion for the LLM.
    pub suggestion: &'static str,
    /// Confidence level determining verdict at each trust level.
    pub confidence: Confidence,
}

/// A collection of related pattern rules for a specific domain.
#[derive(Debug)]
pub struct Pack {
    /// Dot-separated pack identifier (e.g., `core.filesystem`).
    pub id: &'static str,
    /// Human-readable name.
    pub name: &'static str,
    /// Brief description of what the pack guards.
    pub description: &'static str,
    /// Destructive patterns checked in step 3.
    pub destructive_patterns: &'static [PatternRule],
}

// ---------------------------------------------------------------------------
// Dangerous `find` flags — shared between safe validator and destructive rule.
// ---------------------------------------------------------------------------

/// Flags that make `find` destructive (delete/execute actions).
pub const FIND_DESTRUCTIVE_FLAGS: &[&str] = &["-delete", "-exec", "-execdir", "-ok", "-okdir"];

/// Additional flags that make `find` unsafe at low trust (write to files).
pub const FIND_WRITE_FLAGS: &[&str] = &["-fprint", "-fls", "-fprintf"];

/// All dangerous `find` flags (superset for the safe pattern validator).
fn find_all_dangerous_flags() -> &'static [&'static str] {
    static FLAGS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
        let mut v: Vec<&str> = Vec::new();
        v.extend_from_slice(FIND_DESTRUCTIVE_FLAGS);
        v.extend_from_slice(FIND_WRITE_FLAGS);
        v
    });
    &FLAGS
}

// ---------------------------------------------------------------------------
// Sensitive path validator for safe read commands
// ---------------------------------------------------------------------------

/// Safe-pattern validator for read commands (`cat`, `bat`, `head`, etc.).
///
/// Returns `false` when any argument targets a sensitive path, so the
/// command falls through to the `sensitive_read` destructive pattern.
///
/// Re-uses the canonical sensitive-path constants from `core.rs` to avoid
/// drift between the safe-pattern exclusion and the destructive check.
fn read_not_sensitive_validator(cmd: &str) -> bool {
    use core::{SENSITIVE_READ_BASENAMES, SENSITIVE_READ_PREFIXES, SENSITIVE_READ_TARGETS};

    let tokens = tokenize_or_bail!(cmd, false);
    let first = tokens.first().map(|t| t.value.as_str()).unwrap_or("");
    !tokens.iter().skip(1).any(|t| {
        let v = t.value.as_str();
        if v.starts_with('-') || v == first {
            return false;
        }
        if SENSITIVE_READ_TARGETS.contains(&v) {
            return true;
        }
        if SENSITIVE_READ_PREFIXES.iter().any(|p| v.starts_with(p)) {
            return true;
        }
        let basename = v.rsplit('/').next().unwrap_or(v);
        SENSITIVE_READ_BASENAMES.contains(&basename)
    })
}

// ---------------------------------------------------------------------------
// Safe pattern catalog (§3.3)
// ---------------------------------------------------------------------------

/// Safe pattern rules for step 2 matching.
///
/// Each entry uses the standard form `^command\b[^|><]*$` to exclude
/// pipe and redirection operators. The `safe_pattern!` macro generates
/// a `PatternRule` with empty reason/suggestion and `High` confidence.
///
/// Read commands (`cat`, `bat`, `head`, `tail`, `less`, `more`) have a
/// validator that rejects sensitive paths, allowing the `sensitive_read`
/// destructive pattern to catch them.
pub static SAFE_PATTERNS: &[PatternRule] = &[
    // Read-only filesystem
    safe_pattern!("ls", r"^ls\b[^|><]*$"),
    safe_pattern!("cat", r"^cat\b[^|><]*$", read_not_sensitive_validator),
    safe_pattern!("bat", r"^bat\b[^|><]*$", read_not_sensitive_validator),
    safe_pattern!("head", r"^head\b[^|><]*$", read_not_sensitive_validator),
    safe_pattern!("tail", r"^tail\b[^|><]*$", read_not_sensitive_validator),
    safe_pattern!("less", r"^less\b[^|><]*$", read_not_sensitive_validator),
    safe_pattern!("wc", r"^wc\b[^|><]*$"),
    safe_pattern!("file", r"^file\b[^|><]*$"),
    safe_pattern!("stat", r"^stat\b[^|><]*$"),
    safe_pattern!("find", r"^find\b[^|><]*$", find_safe_validator),
    safe_pattern!("du", r"^du\b[^|><]*$"),
    safe_pattern!("df", r"^df\b[^|><]*$"),
    safe_pattern!("tree", r"^tree\b[^|><]*$"),
    safe_pattern!("grep", r"^grep\b[^|><]*$"),
    safe_pattern!("rg", r"^rg\b[^|><]*$"),
    safe_pattern!("diff", r"^diff\b[^|><]*$"),
    safe_pattern!("sort", r"^sort\b[^|><]*$"),
    safe_pattern!("md5sum", r"^md5sum\b[^|><]*$"),
    safe_pattern!("sha256sum", r"^sha256sum\b[^|><]*$"),
    safe_pattern!("realpath", r"^realpath\b[^|><]*$"),
    safe_pattern!("dirname", r"^dirname\b[^|><]*$"),
    safe_pattern!("basename", r"^basename\b[^|><]*$"),
    safe_pattern!("readlink", r"^readlink\b[^|><]*$"),
    safe_pattern!("test", r"^test\b[^|><]*$"),
    safe_pattern!("bracket", r"^\[[^|><]*$"),
    safe_pattern!("double_bracket", r"^\[\[[^|><]*$"),
    // Read-only git
    safe_pattern!("git_status", r"^git\s+status\b[^|><]*$"),
    safe_pattern!("git_log", r"^git\s+log\b[^|><]*$"),
    safe_pattern!("git_diff", r"^git\s+diff\b[^|><]*$"),
    safe_pattern!("git_show", r"^git\s+show\b[^|><]*$"),
    safe_pattern!(
        "git_branch",
        r"^git\s+branch\b[^|><]*$",
        git_branch_safe_validator
    ),
    safe_pattern!("git_tag", r"^git\s+tag\b[^|><]*$", git_tag_safe_validator),
    safe_pattern!("git_remote_v", r"^git\s+remote\s+-v\b[^|><]*$"),
    safe_pattern!("git_rev_parse", r"^git\s+rev-parse\b[^|><]*$"),
    // Read-only build inspection
    safe_pattern!("cargo_check", r"^cargo\s+check\b[^|><]*$"),
    safe_pattern!(
        "cargo_clippy",
        r"^cargo\s+clippy\b[^|><]*$",
        cargo_clippy_safe_validator
    ),
    safe_pattern!("go_vet", r"^go\s+vet\b[^|><]*$"),
    // Environment inspection
    safe_pattern!("env", r"^env\b[^|><]*$"),
    safe_pattern!("printenv", r"^printenv\b[^|><]*$"),
    safe_pattern!("which", r"^which\b[^|><]*$"),
    safe_pattern!("whoami", r"^whoami\b[^|><]*$"),
    safe_pattern!("uname", r"^uname\b[^|><]*$"),
    safe_pattern!("pwd", r"^pwd\b[^|><]*$"),
    safe_pattern!("echo", r"^echo\b[^|><]*$"),
    safe_pattern!("date", r"^date\b[^|><]*$"),
    safe_pattern!("hostname", r"^hostname\b[^|><]*$"),
    safe_pattern!("id", r"^id\b[^|><]*$"),
    safe_pattern!("groups", r"^groups\b[^|><]*$"),
    // Safe filesystem mutation
    safe_pattern!("mkdir", r"^mkdir\b[^|><]*$"),
    safe_pattern!("touch", r"^touch\b[^|><]*$"),
    // Read-only bioinformatics (note: fastqc/multiqc create output report files
    // but are non-destructive — they never modify input data)
    safe_pattern!("samtools_view", r"^samtools\s+view\b[^|><]*$", samtools_view_safe_validator),
    safe_pattern!("samtools_flagstat", r"^samtools\s+flagstat\b[^|><]*$"),
    safe_pattern!("samtools_stats", r"^samtools\s+stats\b[^|><]*$"),
    safe_pattern!("samtools_idxstats", r"^samtools\s+idxstats\b[^|><]*$"),
    safe_pattern!("samtools_depth", r"^samtools\s+depth\b[^|><]*$"),
    safe_pattern!("bcftools_view", r"^bcftools\s+view\b[^|><]*$"),
    safe_pattern!("bcftools_query", r"^bcftools\s+query\b[^|><]*$"),
    safe_pattern!("bcftools_stats", r"^bcftools\s+stats\b[^|><]*$"),
    safe_pattern!("bedtools_intersect", r"^bedtools\s+intersect\b[^|><]*$"),
    safe_pattern!("bedtools_coverage", r"^bedtools\s+coverage\b[^|><]*$"),
    safe_pattern!("fastqc", r"^fastqc\b[^|><]*$"),
    safe_pattern!("multiqc", r"^multiqc\b[^|><]*$"),
    safe_pattern!("blastn", r"^blastn\b[^|><]*$"),
    safe_pattern!("blastp", r"^blastp\b[^|><]*$"),
    safe_pattern!("blastx", r"^blastx\b[^|><]*$"),
    safe_pattern!("feature_counts", r"^featureCounts\b[^|><]*$"),
    safe_pattern!("htseq_count", r"^htseq-count\b[^|><]*$"),
    // Read-only environments / Python
    safe_pattern!("python_version", r"^python[23]?\s+--version\b[^|><]*$"),
    safe_pattern!("pip_list", r"^pip[3]?\s+list\b[^|><]*$"),
    safe_pattern!("pip_show", r"^pip[3]?\s+show\b[^|><]*$"),
    safe_pattern!("pip_freeze", r"^pip[3]?\s+freeze\b[^|><]*$"),
    safe_pattern!("conda_list", r"^conda\s+list\b[^|><]*$"),
    safe_pattern!("conda_env_list", r"^conda\s+env\s+list\b[^|><]*$"),
    safe_pattern!("conda_env_export", r"^conda\s+env\s+export\b[^|><]*$"),
    safe_pattern!("conda_info", r"^conda\s+info\b[^|><]*$"),
    safe_pattern!("mamba_list", r"^mamba\s+list\b[^|><]*$"),
    safe_pattern!("mamba_env_list", r"^mamba\s+env\s+list\b[^|><]*$"),
    safe_pattern!("uv_pip_list", r"^uv\s+pip\s+list\b[^|><]*$"),
    safe_pattern!("uv_pip_show", r"^uv\s+pip\s+show\b[^|><]*$"),
    safe_pattern!("jupyter_kernelspec_list", r"^jupyter\s+kernelspec\s+list\b[^|><]*$"),
    // Read-only R
    safe_pattern!("r_version", r"^R\s+--version\b[^|><]*$"),
    safe_pattern!("rscript_version", r"^Rscript\s+--version\b[^|><]*$"),
    // Read-only HPC schedulers
    safe_pattern!("squeue", r"^squeue\b[^|><]*$"),
    safe_pattern!("sinfo", r"^sinfo\b[^|><]*$"),
    safe_pattern!("sacct", r"^sacct\b[^|><]*$"),
    safe_pattern!("qstat", r"^qstat\b[^|><]*$"),
    safe_pattern!("bjobs", r"^bjobs\b[^|><]*$"),
    safe_pattern!("bhist", r"^bhist\b[^|><]*$"),
    safe_pattern!("bqueues", r"^bqueues\b[^|><]*$"),
    // Read-only HPC environment modules
    safe_pattern!("module_list", r"^module\s+list\b[^|><]*$"),
    safe_pattern!("module_avail", r"^module\s+avail\b[^|><]*$"),
    safe_pattern!("module_show", r"^module\s+show\b[^|><]*$"),
    safe_pattern!("module_spider", r"^module\s+spider\b[^|><]*$"),
    safe_pattern!("module_whatis", r"^module\s+whatis\b[^|><]*$"),
    // Read-only workflow engines
    safe_pattern!("nextflow_log", r"^nextflow\s+log\b[^|><]*$"),
    safe_pattern!("nextflow_list", r"^nextflow\s+list\b[^|><]*$"),
    safe_pattern!("snakemake_summary", r"^snakemake\s+--summary\b[^|><]*$"),
    safe_pattern!("snakemake_dryrun", r"^snakemake\s+(?:--dryrun|-n|--dry-run)\b[^|><]*$"),
    // Read-only data versioning
    safe_pattern!("dvc_status", r"^dvc\s+status\b[^|><]*$"),
    safe_pattern!("dvc_diff", r"^dvc\s+diff\b[^|><]*$"),
    safe_pattern!("dvc_params_diff", r"^dvc\s+params\s+diff\b[^|><]*$"),
    safe_pattern!("dvc_metrics_show", r"^dvc\s+metrics\s+show\b[^|><]*$"),
    safe_pattern!("dvc_plots_show", r"^dvc\s+plots\s+show\b[^|><]*$"),
    safe_pattern!("git_annex_whereis", r"^git\s+annex\s+whereis\b[^|><]*$"),
    safe_pattern!("git_annex_info", r"^git\s+annex\s+info\b[^|><]*$"),
    safe_pattern!("datalad_status", r"^datalad\s+status\b[^|><]*$"),
    // Read-only Spack
    safe_pattern!("spack_find", r"^spack\s+find\b[^|><]*$"),
    safe_pattern!("spack_info", r"^spack\s+info\b[^|><]*$"),
    safe_pattern!("spack_list", r"^spack\s+list\b[^|><]*$"),
    // Read-only geospatial
    safe_pattern!("gdalinfo", r"^gdalinfo\b[^|><]*$"),
    safe_pattern!("ogrinfo", r"^ogrinfo\b[^|><]*$"),
    safe_pattern!("ncdump", r"^ncdump\b[^|><]*$"),
    safe_pattern!("cdo_info", r"^cdo\s+(?:info|sinfo|showname|showvar)\b[^|><]*$"),
    // Read-only ML / GPU
    safe_pattern!("nvidia_smi", r"^nvidia-smi\b[^|><]*$"),
    safe_pattern!("gpustat", r"^gpustat\b[^|><]*$"),
    safe_pattern!("wandb_status", r"^wandb\s+status\b[^|><]*$"),
    safe_pattern!("mlflow_models_list", r"^mlflow\s+models\s+list\b[^|><]*$"),
    // Read-only scientific computing
    safe_pattern!("julia_version", r"^julia\s+--version\b[^|><]*$"),
    safe_pattern!("matlab_ver", r"^matlab\b.*\bver\b[^|><]*$"),
    // Read-only data transfer
    safe_pattern!("globus_ls", r"^globus\s+ls\b[^|><]*$"),
    safe_pattern!("globus_task_list", r"^globus\s+task\s+list\b[^|><]*$"),
    safe_pattern!("rclone_ls", r"^rclone\s+(?:ls|lsd|lsl|lsf|size)\b[^|><]*$"),
    safe_pattern!("ils", r"^ils\b[^|><]*$"),
    safe_pattern!("iquest", r"^iquest\b[^|><]*$"),
    // Read-only chemistry
    safe_pattern!("obabel_list", r"^obabel\s+-L\b[^|><]*$"),
    safe_pattern!("gmx_check", r"^gmx\s+check\b[^|><]*$"),
    safe_pattern!("gmx_dump", r"^gmx\s+dump\b[^|><]*$"),
    // Read-only LaTeX
    safe_pattern!("latexmk_version", r"^latexmk\s+--version\b[^|><]*$"),
    safe_pattern!("biber_version", r"^biber\s+--version\b[^|><]*$"),
    // Stencila read-only
    safe_pattern!(
        "stencila_secrets_list",
        r"^stencila\s+secrets\s+list\b[^|><]*$"
    ),
    safe_pattern!(
        "stencila_auth_status",
        r"^stencila\s+auth\s+status\b[^|><]*$"
    ),
    safe_pattern!(
        "stencila_cloud_status",
        r"^stencila\s+cloud\s+status\b[^|><]*$"
    ),
    safe_pattern!("stencila_db_status", r"^stencila\s+db\s+status\b[^|><]*$"),
    safe_pattern!("stencila_db_log", r"^stencila\s+db\s+log\b[^|><]*$"),
    safe_pattern!("stencila_db_verify", r"^stencila\s+db\s+verify\b[^|><]*$"),
    safe_pattern!("stencila_status", r"^stencila\s+status\b[^|><]*$"),
    safe_pattern!(
        "stencila_formats_list",
        r"^stencila\s+formats\s+list\b[^|><]*$"
    ),
    safe_pattern!(
        "stencila_models_list",
        r"^stencila\s+models\s+list\b[^|><]*$"
    ),
];

// ---------------------------------------------------------------------------
// Safe pattern validators
// ---------------------------------------------------------------------------

/// Validator for `find`: returns `false` (not safe) if any token is a
/// dangerous flag.
fn find_safe_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, false);
    let dangerous = find_all_dangerous_flags();
    !tokens.iter().any(|t| dangerous.contains(&t.value.as_str()))
}

/// Validator for `git branch`: returns `false` if `-D` or `-d` (delete) flags
/// are present.
fn git_branch_safe_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, false);
    !tokens.iter().any(|t| t.value == "-D" || t.value == "-d")
}

/// Validator for `git tag`: returns `false` if `-d` (delete) flag is present.
fn git_tag_safe_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, false);
    !tokens.iter().any(|t| t.value == "-d")
}

/// Validator for `cargo clippy`: returns `false` if `--fix` is present
/// (clippy --fix modifies files).
fn cargo_clippy_safe_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, false);
    !tokens.iter().any(|t| t.value == "--fix")
}

/// Validator for `samtools view`: returns `false` if `-o` (output file) is
/// present, since that writes to a file which could overwrite existing data.
fn samtools_view_safe_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, false);
    !tokens.iter().any(|t| t.value == "-o")
}

// ---------------------------------------------------------------------------
// Compiled RegexSets
// ---------------------------------------------------------------------------

/// All packs in registration order (core first, then extended).
pub fn all_packs() -> &'static [&'static Pack] {
    static PACKS: LazyLock<Vec<&'static Pack>> = LazyLock::new(|| {
        vec![
            // Core packs
            &core::FILESYSTEM_PACK,
            &core::GIT_PACK,
            &core::OBFUSCATION_PACK,
            &core::STENCILA_PACK,
            // Extended packs (alphabetical by module)
            &bioinformatics::SEQUENCE_TOOLS_PACK,
            &chemistry::MOLECULAR_DYNAMICS_PACK,
            &cloud::AWS_PACK,
            &cloud::AZURE_PACK,
            &cloud::GCP_PACK,
            &cloud::IAC_PACK,
            &containers::DOCKER_PACK,
            &containers::KUBECTL_PACK,
            &database::MYSQL_PACK,
            &database::POSTGRESQL_PACK,
            &database::SQLITE_PACK,
            &datasets::VERSIONING_PACK,
            &datasets::TRANSFER_PACK,
            &environments::MANAGERS_PACK,
            &environments::R_PACK,
            &geospatial::CLIMATE_DATA_PACK,
            &geospatial::GDAL_PACK,
            &hpc::APPTAINER_PACK,
            &hpc::SCHEDULERS_PACK,
            &latex::BUILD_TOOLS_PACK,
            &ml::EXPERIMENT_TRACKING_PACK,
            &notebooks::JUPYTER_PACK,
            &packages::REGISTRIES_PACK,
            &scientific::SCIENTIFIC_COMPUTING_PACK,
            &system::DISK_PACK,
            &system::NETWORK_PACK,
            &system::SERVICES_PACK,
            &workflows::WORKFLOW_ENGINES_PACK,
        ]
    });
    &PACKS
}

/// Compiled safe-pattern `RegexSet` with parallel rule references.
pub struct CompiledPatterns {
    pub regex_set: RegexSet,
    pub rules: Vec<&'static PatternRule>,
}

/// Get the compiled safe patterns.
pub fn safe_patterns() -> &'static CompiledPatterns {
    static COMPILED: LazyLock<CompiledPatterns> = LazyLock::new(|| {
        let patterns: Vec<&str> = SAFE_PATTERNS.iter().map(|r| r.pattern).collect();
        let regex_set = RegexSet::new(&patterns).expect("safe patterns should compile");
        let rules: Vec<&PatternRule> = SAFE_PATTERNS.iter().collect();
        CompiledPatterns { regex_set, rules }
    });
    &COMPILED
}

/// Get the compiled destructive patterns (all packs combined).
pub fn destructive_patterns() -> &'static CompiledPatterns {
    static COMPILED: LazyLock<CompiledPatterns> = LazyLock::new(|| {
        let mut patterns = Vec::new();
        let mut rules = Vec::new();
        for pack in all_packs() {
            for rule in pack.destructive_patterns {
                patterns.push(rule.pattern);
                rules.push(rule);
            }
        }
        let regex_set = RegexSet::new(&patterns).expect("destructive patterns should compile");
        CompiledPatterns { regex_set, rules }
    });
    &COMPILED
}

/// Map a rule to its full `rule_id` string (e.g., `shell.core.filesystem.force_push`).
///
/// Uses a pointer-keyed lookup table built once on first call for O(1) access.
pub fn full_rule_id(rule: &PatternRule) -> &'static str {
    static LOOKUP: LazyLock<HashMap<usize, &'static str>> = LazyLock::new(|| {
        let mut map = HashMap::new();
        for pack in all_packs() {
            for r in pack.destructive_patterns {
                let key = std::ptr::from_ref(r) as usize;
                let full = Box::leak(format!("shell.{}.{}", pack.id, r.id).into_boxed_str());
                map.insert(key, full as &'static str);
            }
        }
        map
    });

    let key = std::ptr::from_ref(rule) as usize;
    LOOKUP.get(&key).copied().unwrap_or(rule.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Find a destructive rule by ID within a pack.
    pub(super) fn rule_by_id<'a>(pack: &'a Pack, id: &str) -> &'a PatternRule {
        pack.destructive_patterns
            .iter()
            .find(|r| r.id == id)
            .unwrap_or_else(|| panic!("rule '{id}' not found in pack '{}'", pack.id))
    }

    #[test]
    fn find_safe_validator_flags() {
        assert!(find_safe_validator("find . -name '*.txt'"));
        assert!(find_safe_validator("find . -name exec-summary.txt"));
        assert!(!find_safe_validator("find . -delete"));
        assert!(!find_safe_validator("find . -exec rm {} \\;"));
        assert!(!find_safe_validator("find . -execdir rm {} \\;"));
        assert!(!find_safe_validator("find . -ok rm {} \\;"));
        assert!(!find_safe_validator("find . -okdir rm {} \\;"));
        assert!(!find_safe_validator("find . -fprint output.txt"));
        assert!(!find_safe_validator("find . -fls output.txt"));
        assert!(!find_safe_validator("find . -fprintf output.txt '%p'"));
    }

    #[test]
    fn find_dangerous_flags_superset() {
        let all = find_all_dangerous_flags();
        for flag in FIND_DESTRUCTIVE_FLAGS {
            assert!(
                all.contains(flag),
                "{flag} missing from all dangerous flags"
            );
        }
        for flag in FIND_WRITE_FLAGS {
            assert!(
                all.contains(flag),
                "{flag} missing from all dangerous flags"
            );
        }
    }

    #[test]
    fn safe_patterns_compile() {
        let compiled = safe_patterns();
        assert_eq!(compiled.rules.len(), SAFE_PATTERNS.len());
    }

    #[test]
    fn destructive_patterns_compile() {
        let compiled = destructive_patterns();
        assert!(!compiled.rules.is_empty());
    }

    #[test]
    fn git_branch_safe_validator_flags() {
        assert!(git_branch_safe_validator("git branch"));
        assert!(git_branch_safe_validator("git branch feature-x"));
        assert!(!git_branch_safe_validator("git branch -D feature-x"));
        assert!(!git_branch_safe_validator("git branch -d feature-x"));
    }

    #[test]
    fn git_tag_safe_validator_flags() {
        assert!(git_tag_safe_validator("git tag v1.0"));
        assert!(!git_tag_safe_validator("git tag -d v1.0"));
    }

    #[test]
    fn cargo_clippy_safe_validator_flags() {
        assert!(cargo_clippy_safe_validator("cargo clippy"));
        assert!(cargo_clippy_safe_validator("cargo clippy -- -W warnings"));
        assert!(!cargo_clippy_safe_validator("cargo clippy --fix"));
    }

    #[test]
    fn samtools_view_safe_validator_flags() {
        assert!(samtools_view_safe_validator("samtools view input.bam"));
        assert!(samtools_view_safe_validator("samtools view -h input.bam"));
        assert!(samtools_view_safe_validator("samtools view -b input.sam"));
        assert!(!samtools_view_safe_validator(
            "samtools view -o output.bam input.bam"
        ));
    }

    #[test]
    fn full_rule_id_lookup() {
        // Verify that full_rule_id returns the correct qualified name
        // for a known destructive rule.
        let compiled = destructive_patterns();
        assert!(!compiled.rules.is_empty());
        let first_rule = compiled.rules[0];
        let full = full_rule_id(first_rule);
        assert!(
            full.starts_with("shell."),
            "expected shell. prefix, got: {full}"
        );
        assert!(
            full.ends_with(first_rule.id),
            "expected suffix {}, got: {full}",
            first_rule.id,
        );
    }

    #[test]
    fn read_not_sensitive_validator_rejects_sensitive_paths() {
        // Sensitive paths should be rejected (validator returns false = not safe)
        assert!(!read_not_sensitive_validator("cat /etc/shadow"));
        assert!(!read_not_sensitive_validator("bat ~/.ssh/id_rsa"));
        assert!(!read_not_sensitive_validator("head ~/.aws/credentials"));
        assert!(!read_not_sensitive_validator("tail ~/.gnupg/trustdb.gpg"));
        assert!(!read_not_sensitive_validator("cat .env"));
        assert!(!read_not_sensitive_validator("less /path/to/.netrc"));
        assert!(!read_not_sensitive_validator(
            "cat ~/.config/gcloud/creds.json"
        ));

        // Non-sensitive paths should be allowed (validator returns true = safe)
        assert!(read_not_sensitive_validator("cat README.md"));
        assert!(read_not_sensitive_validator("bat main.rs"));
        assert!(read_not_sensitive_validator("head src/lib.rs"));
        assert!(read_not_sensitive_validator("tail -f logfile.txt"));
        assert!(read_not_sensitive_validator("less Cargo.toml"));
        assert!(read_not_sensitive_validator("cat -n main.rs"));
    }

    #[test]
    fn safe_patterns_reject_pipes_and_redirects() {
        let compiled = safe_patterns();
        // `echo` with redirect should NOT match
        assert!(!compiled.regex_set.is_match("echo foo > /etc/passwd"));
        // `cat` with pipe should NOT match
        assert!(!compiled.regex_set.is_match("cat file | bash"));
        // plain `echo` should match
        assert!(compiled.regex_set.is_match("echo hello"));
        // plain `ls` should match
        assert!(compiled.regex_set.is_match("ls -la"));
    }
}
