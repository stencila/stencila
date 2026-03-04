//! HPC packs: `hpc.schedulers`, `hpc.apptainer`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, safe_pattern};

// ---------------------------------------------------------------------------
// hpc.schedulers
// ---------------------------------------------------------------------------

pub static SCHEDULERS_PACK: Pack = Pack {
    id: "hpc.schedulers",
    name: "HPC Schedulers",
    description: "Guards against destructive HPC job scheduler operations",
    safe_patterns: &[
        safe_pattern!("squeue", r"^squeue\b[^|><]*$"),
        safe_pattern!("sinfo", r"^sinfo\b[^|><]*$"),
        safe_pattern!("sacct", r"^sacct\b[^|><]*$"),
        safe_pattern!("qstat", r"^qstat\b[^|><]*$"),
        safe_pattern!("bjobs", r"^bjobs\b[^|><]*$"),
        safe_pattern!("bhist", r"^bhist\b[^|><]*$"),
        safe_pattern!("bqueues", r"^bqueues\b[^|><]*$"),
        safe_pattern!("module_list", r"^module\s+list\b[^|><]*$"),
        safe_pattern!("module_avail", r"^module\s+avail\b[^|><]*$"),
        safe_pattern!("module_show", r"^module\s+show\b[^|><]*$"),
        safe_pattern!("module_spider", r"^module\s+spider\b[^|><]*$"),
        safe_pattern!("module_whatis", r"^module\s+whatis\b[^|><]*$"),
    ],
    destructive_patterns: &[
        destructive_pattern!(
            "slurm_cancel_all",
            r"\bscancel\b.*(?:--user|-u)\b",
            "Cancels all jobs for a user, including long-running computations",
            "Cancel specific jobs by ID with `scancel <jobid>`; review with `squeue -u $USER` first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "pbs_qdel_all",
            r"\bqdel\s+all\b",
            "Cancels all PBS/Torque queued and running jobs",
            "Delete specific jobs by ID; review with `qstat` first",
            Confidence::High
        ),
        destructive_pattern!(
            "slurm_scontrol_shutdown",
            r"\bscontrol\s+shutdown\b",
            "Shuts down the Slurm controller daemon",
            "This operation should not be performed by an agent",
            Confidence::High
        ),
        destructive_pattern!(
            "lsf_bkill_all",
            r"\bbkill\s+0\b",
            "Cancels all LSF jobs for the current user",
            "Cancel specific jobs by ID with `bkill <jobid>`; review with `bjobs` first",
            Confidence::High
        ),
        destructive_pattern!(
            "lsf_bkill_user",
            r"\bbkill\b.*(?:-u\b|--user\b)",
            "Cancels all LSF jobs for a user, including long-running computations",
            "Cancel specific jobs by ID with `bkill <jobid>`; review with `bjobs -u $USER` first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "sge_qdel_user",
            r"\bqdel\b.*-u\b",
            "Cancels all SGE/UGE jobs for a user",
            "Delete specific jobs by ID; review with `qstat -u $USER` first",
            Confidence::Medium
        ),
    ],
};

// ---------------------------------------------------------------------------
// hpc.apptainer
// ---------------------------------------------------------------------------

pub static APPTAINER_PACK: Pack = Pack {
    id: "hpc.apptainer",
    name: "HPC Containers",
    description: "Guards against destructive Singularity/Apptainer container operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!(
            "singularity_delete",
            r"\b(?:singularity|apptainer)\s+delete\b",
            "Permanently removes a container image from a remote library",
            "Use `singularity inspect` to review the image first",
            Confidence::High
        ),
        destructive_pattern!(
            "singularity_cache_clean_all",
            r"\b(?:singularity|apptainer)\s+cache\s+clean\b.*(?:--all|-a)\b",
            "Removes all cached container images, requiring re-download",
            "Use `singularity cache list` to review cached images first",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- Schedulers --

    #[test]
    fn slurm_cancel_all_matches() {
        let re = Regex::new(rule_by_id(&SCHEDULERS_PACK, "slurm_cancel_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("scancel --user=myuser"));
        assert!(re.is_match("scancel -u myuser"));
        assert!(!re.is_match("scancel 12345"));
    }

    #[test]
    fn pbs_qdel_all_matches() {
        let re = Regex::new(rule_by_id(&SCHEDULERS_PACK, "pbs_qdel_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("qdel all"));
        assert!(!re.is_match("qdel 12345"));
        assert!(!re.is_match("qdel 12345.pbs"));
    }

    #[test]
    fn slurm_scontrol_shutdown_matches() {
        let re = Regex::new(rule_by_id(&SCHEDULERS_PACK, "slurm_scontrol_shutdown").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("scontrol shutdown"));
        assert!(!re.is_match("scontrol show job 12345"));
        assert!(!re.is_match("scontrol show partition"));
    }

    #[test]
    fn lsf_bkill_all_matches() {
        let re = Regex::new(rule_by_id(&SCHEDULERS_PACK, "lsf_bkill_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("bkill 0"));
        assert!(!re.is_match("bkill 12345"));
        assert!(!re.is_match("bjobs"));
    }

    #[test]
    fn lsf_bkill_user_matches() {
        let re = Regex::new(rule_by_id(&SCHEDULERS_PACK, "lsf_bkill_user").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("bkill -u myuser"));
        assert!(!re.is_match("bkill 12345"));
        assert!(!re.is_match("bjobs -u myuser"));
    }

    #[test]
    fn sge_qdel_user_matches() {
        let re = Regex::new(rule_by_id(&SCHEDULERS_PACK, "sge_qdel_user").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("qdel -u myuser"));
        assert!(!re.is_match("qdel 12345"));
        assert!(!re.is_match("qstat -u myuser"));
    }

    // -- HPC Containers --

    #[test]
    fn singularity_delete_matches() {
        let re = Regex::new(rule_by_id(&APPTAINER_PACK, "singularity_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("singularity delete library://user/collection/image:tag"));
        assert!(re.is_match("apptainer delete library://user/collection/image:tag"));
        assert!(!re.is_match("singularity pull library://user/collection/image:tag"));
        assert!(!re.is_match("singularity inspect image.sif"));
        assert!(!re.is_match("apptainer run image.sif"));
    }

    #[test]
    fn singularity_cache_clean_all_matches() {
        let re = Regex::new(rule_by_id(&APPTAINER_PACK, "singularity_cache_clean_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("singularity cache clean --all"));
        assert!(re.is_match("singularity cache clean -a"));
        assert!(re.is_match("apptainer cache clean --all"));
        assert!(re.is_match("apptainer cache clean -a"));
        assert!(!re.is_match("singularity cache list"));
        assert!(!re.is_match("singularity cache clean"));
        assert!(!re.is_match("apptainer cache list"));
    }
}
