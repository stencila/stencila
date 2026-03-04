//! Package registry pack: `packages.registries`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, has_token, tokenize_or_bail};

/// Validator for publish commands: returns `false` (does not fire) if `--dry-run`
/// is present, since dry runs are safe.
fn publish_no_dryrun_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    !has_token(&tokens, "--dry-run")
}

pub static REGISTRIES_PACK: Pack = Pack {
    id: "packages.registries",
    name: "Package Registries",
    description: "Guards against destructive package manager operations affecting registries",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!(
            "npm_unpublish",
            r"\b(?:npm|yarn\s+npm|pnpm)\s+unpublish\b",
            "Unpublishing removes a package version from the public registry, potentially breaking dependents",
            "Use `npm deprecate` to mark versions as deprecated instead",
            Confidence::High
        ),
        destructive_pattern!(
            "npm_deprecate",
            r"\b(?:npm|pnpm)\s+deprecate\b",
            "Deprecating a package version affects all consumers",
            "Verify the package name and version before deprecating",
            Confidence::Medium
        ),
        destructive_pattern!(
            "npm_cache_clean",
            r"\b(?:npm\s+cache\s+clean\s+--force|yarn\s+cache\s+clean|pnpm\s+store\s+prune)\b",
            "Removes the local package cache, requiring full re-download",
            "Use `npm cache verify` to check cache integrity instead",
            Confidence::Medium
        ),
        destructive_pattern!(
            "cargo_publish",
            r"\bcargo\s+publish\b",
            publish_no_dryrun_validator,
            "Publishing a crate to crates.io is a public, irreversible action",
            "Verify package metadata with `cargo package --list` first; ensure version and contents are correct",
            Confidence::High
        ),
        destructive_pattern!(
            "npm_publish",
            r"\b(?:npm|yarn|pnpm)\s+publish\b",
            publish_no_dryrun_validator,
            "Publishing a package to a registry is a public, irreversible action",
            "Verify package contents with `npm pack --dry-run` first; ensure version and contents are correct",
            Confidence::High
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    #[test]
    fn npm_unpublish_matches() {
        let re = Regex::new(rule_by_id(&REGISTRIES_PACK, "npm_unpublish").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("npm unpublish my-package"));
        assert!(re.is_match("pnpm unpublish my-package"));
        assert!(!re.is_match("npm install my-package"));
    }

    #[test]
    fn npm_deprecate_matches() {
        let re = Regex::new(rule_by_id(&REGISTRIES_PACK, "npm_deprecate").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("npm deprecate my-package@1.0.0 \"use v2\""));
        assert!(re.is_match("pnpm deprecate my-package@1.0.0 \"use v2\""));
    }

    #[test]
    fn npm_cache_clean_matches() {
        let re = Regex::new(rule_by_id(&REGISTRIES_PACK, "npm_cache_clean").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("npm cache clean --force"));
        assert!(re.is_match("yarn cache clean"));
        assert!(re.is_match("pnpm store prune"));
        assert!(!re.is_match("npm cache verify"));
    }

    #[test]
    fn cargo_publish_matches() {
        let re = Regex::new(rule_by_id(&REGISTRIES_PACK, "cargo_publish").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("cargo publish"));
        assert!(re.is_match("cargo publish --dry-run"));
        assert!(!re.is_match("cargo build"));

        // Validator: dry-run is safe, so validator returns false
        assert!(publish_no_dryrun_validator("cargo publish"));
        assert!(!publish_no_dryrun_validator("cargo publish --dry-run"));
    }

    #[test]
    fn npm_publish_matches() {
        let re = Regex::new(rule_by_id(&REGISTRIES_PACK, "npm_publish").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("npm publish"));
        assert!(re.is_match("yarn publish"));
        assert!(re.is_match("pnpm publish"));
        assert!(!re.is_match("npm install"));

        // Validator: dry-run is safe, so validator returns false
        assert!(!publish_no_dryrun_validator("npm publish --dry-run"));
    }
}
