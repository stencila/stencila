//! Workflow packs: `workflows.engines`.

use super::{Confidence, Pack, PatternRule, destructive_pattern};

// ---------------------------------------------------------------------------
// workflows.engines
// ---------------------------------------------------------------------------

pub static WORKFLOW_ENGINES_PACK: Pack = Pack {
    id: "workflows.engines",
    name: "Workflow Engines",
    description: "Guards against destructive operations in scientific workflow engines",
    destructive_patterns: &[
        destructive_pattern!(
            "nextflow_clean",
            r"\bnextflow\s+clean\b",
            "Removes cached workflow data and intermediate results",
            "Use `nextflow log` to review runs before cleaning",
            Confidence::Medium
        ),
        destructive_pattern!(
            "nextflow_drop",
            r"\bnextflow\s+drop\b",
            "Removes a cached pipeline from local storage",
            "Use `nextflow list` to review cached pipelines first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "snakemake_delete_all",
            r"\bsnakemake\b.*--delete-all-output\b",
            "Deletes all output files produced by the workflow",
            "Review outputs with `snakemake --summary` first",
            Confidence::High
        ),
        destructive_pattern!(
            "snakemake_unlock",
            r"\bsnakemake\b.*--unlock\b",
            "Force-unlocks a workflow directory, risking concurrent execution conflicts",
            "Ensure no other Snakemake instance is running first",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- Workflow Engines --

    #[test]
    fn nextflow_clean_matches() {
        let re = Regex::new(rule_by_id(&WORKFLOW_ENGINES_PACK, "nextflow_clean").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("nextflow clean"));
        assert!(re.is_match("nextflow clean -f"));
        assert!(!re.is_match("nextflow run main.nf"));
        assert!(!re.is_match("nextflow log"));
    }

    #[test]
    fn nextflow_drop_matches() {
        let re = Regex::new(rule_by_id(&WORKFLOW_ENGINES_PACK, "nextflow_drop").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("nextflow drop nextflow-io/hello"));
        assert!(re.is_match("nextflow drop my-pipeline"));
        assert!(!re.is_match("nextflow list"));
        assert!(!re.is_match("nextflow run main.nf"));
    }

    #[test]
    fn snakemake_delete_all_matches() {
        let re = Regex::new(rule_by_id(&WORKFLOW_ENGINES_PACK, "snakemake_delete_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("snakemake --delete-all-output"));
        assert!(re.is_match("snakemake --cores 4 --delete-all-output"));
        assert!(!re.is_match("snakemake --summary"));
        assert!(!re.is_match("snakemake --dryrun"));
    }

    #[test]
    fn snakemake_unlock_matches() {
        let re = Regex::new(rule_by_id(&WORKFLOW_ENGINES_PACK, "snakemake_unlock").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("snakemake --unlock"));
        assert!(re.is_match("snakemake --cores 1 --unlock"));
        assert!(!re.is_match("snakemake --dryrun"));
        assert!(!re.is_match("snakemake --summary"));
    }
}
