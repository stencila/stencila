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
pub(crate) use safe_pattern;

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
    /// Safe patterns checked in step 2.
    pub safe_patterns: &'static [PatternRule],
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

/// Get the compiled safe patterns (all packs combined).
pub fn safe_patterns() -> &'static CompiledPatterns {
    static COMPILED: LazyLock<CompiledPatterns> = LazyLock::new(|| {
        let mut patterns = Vec::new();
        let mut rules = Vec::new();
        for pack in all_packs() {
            for rule in pack.safe_patterns {
                patterns.push(rule.pattern);
                rules.push(rule);
            }
        }
        let regex_set = RegexSet::new(&patterns).expect("safe patterns should compile");
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
        assert!(!compiled.rules.is_empty());
    }

    #[test]
    fn destructive_patterns_compile() {
        let compiled = destructive_patterns();
        assert!(!compiled.rules.is_empty());
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
