//! Notebook pack: `notebooks.jupyter`.

use super::{
    Confidence, Pack, PatternRule, destructive_pattern, has_token, safe_pattern, tokenize_or_bail,
};

// ---------------------------------------------------------------------------
// notebooks.jupyter
// ---------------------------------------------------------------------------

/// Validator for `nbconvert_inplace`: returns `true` (fires) only when
/// `--inplace` is present, since non-inplace conversions write to a new file.
fn nbconvert_inplace_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    has_token(&tokens, "--inplace")
}

pub static JUPYTER_PACK: Pack = Pack {
    id: "notebooks.jupyter",
    name: "Jupyter Notebooks",
    description: "Guards against destructive Jupyter notebook operations",
    safe_patterns: &[safe_pattern!(
        "jupyter_kernelspec_list",
        r"^jupyter\s+kernelspec\s+list\b[^|><]*$"
    )],
    destructive_patterns: &[
        destructive_pattern!(
            "nbstripout",
            r"\bnbstripout\b",
            "Strips output cells from notebooks, permanently removing computed results",
            "Backup the notebook first; use `nbstripout --dry-run` if available to preview",
            Confidence::Medium
        ),
        destructive_pattern!(
            "nbconvert_inplace",
            r"\bjupyter\s+nbconvert\b",
            nbconvert_inplace_validator,
            "In-place notebook conversion overwrites the original file",
            "Use `--output` to write to a new file instead of `--inplace`",
            Confidence::Medium
        ),
        destructive_pattern!(
            "jupyter_kernelspec_uninstall",
            r"\bjupyter\s+kernelspec\s+(?:uninstall|remove)\b",
            "Removes a Jupyter kernel specification",
            "Use `jupyter kernelspec list` to review installed kernels first",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    #[test]
    fn nbstripout_matches() {
        let re = Regex::new(rule_by_id(&JUPYTER_PACK, "nbstripout").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("nbstripout notebook.ipynb"));
        assert!(re.is_match("nbstripout --install"));
        assert!(!re.is_match("jupyter notebook"));
    }

    #[test]
    fn nbconvert_inplace_matches() {
        let re = Regex::new(rule_by_id(&JUPYTER_PACK, "nbconvert_inplace").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("jupyter nbconvert --inplace notebook.ipynb"));
        assert!(re.is_match("jupyter nbconvert --to markdown notebook.ipynb"));
        // Pattern matches broadly, validator narrows
        assert!(nbconvert_inplace_validator(
            "jupyter nbconvert --inplace notebook.ipynb"
        ));
        assert!(!nbconvert_inplace_validator(
            "jupyter nbconvert --to html notebook.ipynb"
        ));
    }

    #[test]
    fn jupyter_kernelspec_uninstall_matches() {
        let re = Regex::new(rule_by_id(&JUPYTER_PACK, "jupyter_kernelspec_uninstall").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("jupyter kernelspec uninstall python3"));
        assert!(re.is_match("jupyter kernelspec remove mykernel"));
        assert!(!re.is_match("jupyter kernelspec list"));
        assert!(!re.is_match("jupyter kernelspec install mykernel"));
    }
}
