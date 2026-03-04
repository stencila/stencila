//! LaTeX pack: `latex.build_tools`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, safe_pattern};

// ---------------------------------------------------------------------------
// latex.build_tools
// ---------------------------------------------------------------------------

pub static BUILD_TOOLS_PACK: Pack = Pack {
    id: "latex.build_tools",
    name: "LaTeX Build Tools",
    description: "Guards against destructive LaTeX build tool operations",
    safe_patterns: &[
        safe_pattern!("latexmk_version", r"^latexmk\s+--version\b[^|><]*$"),
        safe_pattern!("biber_version", r"^biber\s+--version\b[^|><]*$"),
    ],
    destructive_patterns: &[
        destructive_pattern!(
            "latexmk_clean",
            r"\blatexmk\s+-C\b",
            "Removes all generated files including PDFs and auxiliary files",
            "Use `latexmk -c` (lowercase) to clean only auxiliary files, preserving PDFs",
            Confidence::Medium
        ),
        destructive_pattern!(
            "latexmk_clean_all",
            r"\blatexmk\s+-CA\b",
            "Removes all generated files including PDFs, auxiliary files, and extra generated files",
            "Use `latexmk -c` (lowercase) to clean only auxiliary files, preserving PDFs",
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
    fn latexmk_clean_matches() {
        let re = Regex::new(rule_by_id(&BUILD_TOOLS_PACK, "latexmk_clean").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("latexmk -C"));
        assert!(re.is_match("latexmk -C project.tex"));
        assert!(!re.is_match("latexmk -c"));
        assert!(!re.is_match("latexmk -pdf"));
        assert!(!re.is_match("latexmk project.tex"));
    }

    #[test]
    fn latexmk_clean_all_matches() {
        let re = Regex::new(rule_by_id(&BUILD_TOOLS_PACK, "latexmk_clean_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("latexmk -CA"));
        assert!(re.is_match("latexmk -CA project.tex"));
        assert!(!re.is_match("latexmk -c"));
        assert!(!re.is_match("latexmk -C"));
        assert!(!re.is_match("latexmk -pdf"));
        assert!(!re.is_match("latexmk project.tex"));
    }

    #[test]
    fn latexmk_clean_does_not_match_clean_all() {
        let re = Regex::new(rule_by_id(&BUILD_TOOLS_PACK, "latexmk_clean").pattern)
            .expect("pattern should compile");
        // -CA contains -C but the \b anchor should prevent matching
        // because 'A' is a word character, so -C is not at a word boundary in -CA
        assert!(!re.is_match("latexmk -CA"));
    }
}
