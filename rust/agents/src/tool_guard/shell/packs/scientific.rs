//! Scientific computing pack: `scientific.computing`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, safe_pattern};

// ---------------------------------------------------------------------------
// scientific.computing
// ---------------------------------------------------------------------------

pub static SCIENTIFIC_COMPUTING_PACK: Pack = Pack {
    id: "scientific.computing",
    name: "Scientific Computing",
    description: "Guards against destructive operations in scientific computing environments",
    safe_patterns: &[
        safe_pattern!("julia_version", r"^julia\s+--version\b[^|><]*$"),
        safe_pattern!("matlab_ver", r"^matlab\b.*\bver\b[^|><]*$"),
    ],
    destructive_patterns: &[
        destructive_pattern!(
            "julia_pkg_gc",
            r"\bjulia\b.*\bPkg\.gc\b",
            "Removes unused Julia package versions from the package depot",
            "Use `Pkg.status()` to review installed packages first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "julia_pkg_rm",
            r"\bjulia\b.*\bPkg\.rm\b",
            "Removes Julia packages, potentially breaking dependent packages",
            "Use `Pkg.status()` to review dependencies before removing",
            Confidence::Medium
        ),
        destructive_pattern!(
            "matlab_delete",
            r"\bmatlab\b.*\bdelete\s*\(",
            "Deletes files from within a MATLAB session",
            "Verify file paths with `dir()` before deleting",
            Confidence::Medium
        ),
        destructive_pattern!(
            "matlab_rmdir",
            r"\bmatlab\b.*\brmdir\s*\(",
            "Removes directories from within a MATLAB session",
            "Verify directory contents with `dir()` before removing",
            Confidence::Medium
        ),
        destructive_pattern!(
            "octave_unlink",
            r"\boctave\b.*\bunlink\s*\(",
            "Deletes files from within an Octave session",
            "Verify file paths before deleting",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- Julia --

    #[test]
    fn julia_pkg_gc_matches() {
        let re = Regex::new(rule_by_id(&SCIENTIFIC_COMPUTING_PACK, "julia_pkg_gc").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("julia -e 'using Pkg; Pkg.gc()'"));
        assert!(re.is_match("julia --project=. -e 'Pkg.gc()'"));
        assert!(!re.is_match("julia -e 'using Pkg; Pkg.status()'"));
        assert!(!re.is_match("julia script.jl"));
        assert!(!re.is_match("Pkg.gc()"));
    }

    #[test]
    fn julia_pkg_rm_matches() {
        let re = Regex::new(rule_by_id(&SCIENTIFIC_COMPUTING_PACK, "julia_pkg_rm").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("julia -e 'using Pkg; Pkg.rm(\"Example\")'"));
        assert!(re.is_match("julia --project=. -e 'Pkg.rm(\"DataFrames\")'"));
        assert!(!re.is_match("julia -e 'using Pkg; Pkg.add(\"Example\")'"));
        assert!(!re.is_match("julia -e 'Pkg.status()'"));
        assert!(!re.is_match("Pkg.rm(\"Example\")"));
    }

    // -- MATLAB --

    #[test]
    fn matlab_delete_matches() {
        let re = Regex::new(rule_by_id(&SCIENTIFIC_COMPUTING_PACK, "matlab_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("matlab -batch \"delete('output.mat')\""));
        assert!(re.is_match("matlab -r \"delete ('temp.csv')\""));
        assert!(!re.is_match("matlab -batch \"save('output.mat')\""));
        assert!(!re.is_match("matlab -r \"dir('.')\""));
        assert!(!re.is_match("delete('file.txt')"));
    }

    #[test]
    fn matlab_rmdir_matches() {
        let re = Regex::new(rule_by_id(&SCIENTIFIC_COMPUTING_PACK, "matlab_rmdir").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("matlab -batch \"rmdir('output_dir')\""));
        assert!(re.is_match("matlab -r \"rmdir ('temp', 's')\""));
        assert!(!re.is_match("matlab -batch \"mkdir('new_dir')\""));
        assert!(!re.is_match("matlab -r \"dir('.')\""));
        assert!(!re.is_match("rmdir('output_dir')"));
    }

    // -- Octave --

    #[test]
    fn octave_unlink_matches() {
        let re = Regex::new(rule_by_id(&SCIENTIFIC_COMPUTING_PACK, "octave_unlink").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("octave --eval \"unlink('temp.dat')\""));
        assert!(re.is_match("octave -q --eval \"unlink ('output.csv')\""));
        assert!(!re.is_match("octave --eval \"save('data.mat')\""));
        assert!(!re.is_match("octave script.m"));
        assert!(!re.is_match("unlink('temp.dat')"));
    }
}
