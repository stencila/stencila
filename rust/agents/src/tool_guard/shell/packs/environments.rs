//! Environment packs: `environments.managers`, `environments.r`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, tokenize_or_bail};

// ---------------------------------------------------------------------------
// environments.managers
// ---------------------------------------------------------------------------

/// Validator for `conda_env_remove`: returns `false` (does not fire) if
/// `--dry-run` or `-d` is present.
fn conda_env_remove_no_dryrun_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    !tokens
        .iter()
        .any(|t| t.value == "--dry-run" || t.value == "-d")
}

pub static MANAGERS_PACK: Pack = Pack {
    id: "environments.managers",
    name: "Environment Managers",
    description: "Guards against destructive operations on environment and package managers",
    destructive_patterns: &[
        destructive_pattern!(
            "conda_env_remove",
            r"\bconda\s+(?:env\s+remove|remove\s+--all)\b",
            conda_env_remove_no_dryrun_validator,
            "Permanently removes a Conda environment and all installed packages",
            "Use `conda env export` to save the environment specification first",
            Confidence::High
        ),
        destructive_pattern!(
            "conda_clean_all",
            r"\bconda\s+clean\s+.*(?:--all|-a)\b",
            "Removes all cached packages, tarballs, and index caches",
            "Use `conda clean --dry-run --all` to preview what will be removed",
            Confidence::Medium
        ),
        destructive_pattern!(
            "pip_uninstall",
            r"\bpip[3]?\s+uninstall\b",
            "Uninstalls Python packages which may break dependent packages",
            "Use `pip show <package>` to review dependents before uninstalling",
            Confidence::Medium
        ),
        destructive_pattern!(
            "virtualenv_remove",
            r"\brm\b.*\b(?:\.?venv|virtualenv)\b",
            "Removes a Python virtual environment directory",
            "Backup with `pip freeze > requirements.txt` before deleting the environment",
            Confidence::Medium
        ),
        destructive_pattern!(
            "mamba_env_remove",
            r"\bmamba\s+(?:env\s+remove|remove\s+--all)\b",
            "Permanently removes a Mamba/Conda environment and all installed packages",
            "Use `mamba env export` to save the environment specification first",
            Confidence::High
        ),
        destructive_pattern!(
            "uv_cache_clean",
            r"\buv\s+cache\s+clean\b",
            "Removes the entire uv package cache",
            "Use `uv cache dir` to inspect the cache location first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "micromamba_env_remove",
            r"\bmicromamba\s+(?:env\s+remove|remove\s+--all)\b",
            "Permanently removes a Micromamba environment and all installed packages",
            "Use `micromamba env export` to save the environment specification first",
            Confidence::High
        ),
        destructive_pattern!(
            "poetry_env_remove",
            r"\bpoetry\s+env\s+remove\b",
            "Removes a Poetry virtual environment",
            "Use `poetry env info` to review the environment first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "pipenv_rm",
            r"\bpipenv\s+--rm\b",
            "Removes the Pipenv virtual environment",
            "Ensure `Pipfile.lock` is committed so the environment can be recreated",
            Confidence::Medium
        ),
        destructive_pattern!(
            "spack_uninstall_all",
            r"\bspack\s+uninstall\b.*(?:--all|-a)\b",
            "Uninstalls all Spack packages, potentially removing shared dependencies",
            "Uninstall specific packages by name; review with `spack find` first",
            Confidence::High
        ),
    ],
};

// ---------------------------------------------------------------------------
// environments.r
// ---------------------------------------------------------------------------

pub static R_PACK: Pack = Pack {
    id: "environments.r",
    name: "R Language",
    description: "Guards against destructive R package management operations",
    destructive_patterns: &[
        destructive_pattern!(
            "r_remove_packages",
            r"\bRscript\b.*\bremove\.packages\b",
            "Removes installed R packages",
            "Use `installed.packages()` to review before removing",
            Confidence::Medium
        ),
        destructive_pattern!(
            "r_unlink",
            r"\bRscript\b.*\bunlink\b.*recursive\s*=\s*TRUE",
            "Recursively deletes files or directories from an R script",
            "Verify paths with `list.files()` before deleting",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- Managers --

    #[test]
    fn conda_env_remove_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "conda_env_remove").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("conda env remove -n myenv"));
        assert!(re.is_match("conda remove --all -n myenv"));
        assert!(!re.is_match("conda install numpy"));
        assert!(!re.is_match("conda env list"));
        assert!(!re.is_match("conda env export"));
    }

    #[test]
    fn conda_env_remove_dryrun() {
        assert!(conda_env_remove_no_dryrun_validator("conda env remove -n myenv"));
        assert!(!conda_env_remove_no_dryrun_validator(
            "conda env remove -n myenv --dry-run"
        ));
        assert!(!conda_env_remove_no_dryrun_validator(
            "conda env remove -n myenv -d"
        ));
    }

    #[test]
    fn conda_clean_all_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "conda_clean_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("conda clean --all"));
        assert!(re.is_match("conda clean -a"));
        assert!(re.is_match("conda clean --packages --all"));
        assert!(!re.is_match("conda clean --packages"));
        assert!(!re.is_match("conda install numpy"));
    }

    #[test]
    fn pip_uninstall_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "pip_uninstall").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("pip uninstall numpy"));
        assert!(re.is_match("pip3 uninstall pandas"));
        assert!(!re.is_match("pip install numpy"));
        assert!(!re.is_match("pip list"));
        assert!(!re.is_match("pip show numpy"));
    }

    #[test]
    fn mamba_env_remove_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "mamba_env_remove").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("mamba env remove -n myenv"));
        assert!(re.is_match("mamba remove --all -n myenv"));
        assert!(!re.is_match("mamba install numpy"));
        assert!(!re.is_match("mamba env list"));
    }

    #[test]
    fn uv_cache_clean_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "uv_cache_clean").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("uv cache clean"));
        assert!(!re.is_match("uv cache dir"));
        assert!(!re.is_match("uv pip install numpy"));
    }

    #[test]
    fn micromamba_env_remove_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "micromamba_env_remove").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("micromamba env remove -n myenv"));
        assert!(re.is_match("micromamba remove --all -n myenv"));
        assert!(!re.is_match("micromamba install numpy"));
        assert!(!re.is_match("micromamba env list"));
    }

    #[test]
    fn poetry_env_remove_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "poetry_env_remove").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("poetry env remove python3.11"));
        assert!(re.is_match("poetry env remove --all"));
        assert!(!re.is_match("poetry env info"));
        assert!(!re.is_match("poetry install"));
    }

    #[test]
    fn pipenv_rm_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "pipenv_rm").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("pipenv --rm"));
        assert!(!re.is_match("pipenv install"));
        assert!(!re.is_match("pipenv shell"));
    }

    #[test]
    fn spack_uninstall_all_matches() {
        let re = Regex::new(rule_by_id(&MANAGERS_PACK, "spack_uninstall_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("spack uninstall --all"));
        assert!(re.is_match("spack uninstall -a"));
        assert!(!re.is_match("spack uninstall hdf5"));
        assert!(!re.is_match("spack find"));
    }

    // -- R Language --

    #[test]
    fn r_remove_packages_matches() {
        let re = Regex::new(rule_by_id(&R_PACK, "r_remove_packages").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("Rscript -e 'remove.packages(\"ggplot2\")'"));
        assert!(!re.is_match("Rscript -e 'install.packages(\"ggplot2\")'"));
        assert!(!re.is_match("Rscript script.R"));
    }

    #[test]
    fn r_unlink_matches() {
        let re = Regex::new(rule_by_id(&R_PACK, "r_unlink").pattern)
            .expect("pattern should compile");
        assert!(re.is_match(
            "Rscript -e 'unlink(\"output\", recursive = TRUE)'"
        ));
        assert!(!re.is_match("Rscript -e 'unlink(\"file.txt\")'"));
        assert!(!re.is_match("Rscript script.R"));
    }
}
