//! Dataset packs: `datasets.versioning`, `datasets.transfer`.

use super::{Confidence, Pack, PatternRule, destructive_pattern};

// ---------------------------------------------------------------------------
// datasets.versioning
// ---------------------------------------------------------------------------

pub static VERSIONING_PACK: Pack = Pack {
    id: "datasets.versioning",
    name: "Data Versioning",
    description: "Guards against destructive operations on scientific data versioning tools",
    destructive_patterns: &[
        destructive_pattern!(
            "dvc_gc",
            r"\bdvc\s+gc\b",
            "Removes cached DVC file versions, potentially losing data not on the current branch",
            "Use `dvc gc --dry` to preview what will be removed; avoid `--all-commits`",
            Confidence::High
        ),
        destructive_pattern!(
            "dvc_remove",
            r"\bdvc\s+remove\b",
            "Stops tracking files with DVC and removes .dvc metadata",
            "Use `dvc status` to review tracked files first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "git_annex_drop_force",
            r"\bgit\s+annex\s+drop\b.*--force\b",
            "Force-drops annexed file content without verifying remote copies exist",
            "Use `git annex drop` without `--force` to ensure remote copies are verified first",
            Confidence::High
        ),
        destructive_pattern!(
            "git_annex_drop",
            r"\bgit\s+annex\s+drop\b",
            "Drops local copies of annexed file content",
            "Use `git annex whereis` to verify remote copies exist before dropping",
            Confidence::Medium
        ),
        destructive_pattern!(
            "datalad_drop",
            r"\bdatalad\s+drop\b",
            "Drops local file content from a DataLad dataset",
            "Use `datalad status` to review before dropping; avoid `--reckless`",
            Confidence::Medium
        ),
    ],
};

// ---------------------------------------------------------------------------
// datasets.transfer
// ---------------------------------------------------------------------------

pub static TRANSFER_PACK: Pack = Pack {
    id: "datasets.transfer",
    name: "Data Transfer Tools",
    description: "Guards against destructive operations in scientific data transfer tools",
    destructive_patterns: &[
        destructive_pattern!(
            "globus_delete",
            r"\bglobus\s+delete\b",
            "Permanently deletes files on a Globus endpoint",
            "Use `globus ls` to review endpoint contents before deleting",
            Confidence::High
        ),
        destructive_pattern!(
            "irods_irm",
            r"\birm\b",
            "Removes files or collections from iRODS",
            "Use `ils` to review the collection before removing",
            Confidence::Medium
        ),
        destructive_pattern!(
            "irods_irmtrash",
            r"\birmtrash\b",
            "Permanently purges iRODS trash, making recovery impossible",
            "Review trash contents before purging",
            Confidence::High
        ),
        destructive_pattern!(
            "rclone_delete",
            r"\brclone\s+(?:delete|purge)\b",
            "Permanently removes files from remote storage",
            "Use `rclone ls` to review files first; prefer `rclone move` with a backup",
            Confidence::High
        ),
        destructive_pattern!(
            "rclone_sync_delete",
            r"\brclone\s+sync\b",
            "Syncs source to destination, deleting files at destination that don't exist at source",
            "Use `rclone sync --dry-run` to preview changes; prefer `rclone copy` to avoid deletions",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- DVC --

    #[test]
    fn dvc_gc_matches() {
        let re = Regex::new(rule_by_id(&VERSIONING_PACK, "dvc_gc").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("dvc gc"));
        assert!(re.is_match("dvc gc --workspace"));
        assert!(re.is_match("dvc gc --all-commits"));
        assert!(!re.is_match("dvc status"));
        assert!(!re.is_match("dvc push"));
    }

    #[test]
    fn dvc_remove_matches() {
        let re = Regex::new(rule_by_id(&VERSIONING_PACK, "dvc_remove").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("dvc remove data.dvc"));
        assert!(!re.is_match("dvc status"));
        assert!(!re.is_match("dvc add data.csv"));
    }

    // -- git-annex --

    #[test]
    fn git_annex_drop_force_matches() {
        let re =
            Regex::new(rule_by_id(&VERSIONING_PACK, "git_annex_drop_force").pattern)
                .expect("pattern should compile");
        assert!(re.is_match("git annex drop --force large_file.dat"));
        assert!(!re.is_match("git annex drop large_file.dat"));
        assert!(!re.is_match("git annex whereis large_file.dat"));
    }

    #[test]
    fn git_annex_drop_matches() {
        let re = Regex::new(rule_by_id(&VERSIONING_PACK, "git_annex_drop").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("git annex drop large_file.dat"));
        assert!(re.is_match("git annex drop --force large_file.dat"));
        assert!(!re.is_match("git annex get large_file.dat"));
        assert!(!re.is_match("git annex whereis large_file.dat"));
    }

    // -- DataLad --

    #[test]
    fn datalad_drop_matches() {
        let re = Regex::new(rule_by_id(&VERSIONING_PACK, "datalad_drop").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("datalad drop dataset/"));
        assert!(re.is_match("datalad drop --reckless availability ."));
        assert!(!re.is_match("datalad status"));
        assert!(!re.is_match("datalad get dataset/"));
    }

    // -- Globus --

    #[test]
    fn globus_delete_matches() {
        let re = Regex::new(rule_by_id(&TRANSFER_PACK, "globus_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("globus delete abc123:/path/to/file"));
        assert!(re.is_match("globus delete --recursive abc123:/data/"));
        assert!(!re.is_match("globus ls abc123:/path/"));
        assert!(!re.is_match("globus transfer abc123:/src abc456:/dst"));
        assert!(!re.is_match("globus endpoint show abc123"));
    }

    // -- iRODS --

    #[test]
    fn irods_irm_matches() {
        let re = Regex::new(rule_by_id(&TRANSFER_PACK, "irods_irm").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("irm /tempZone/home/user/file.txt"));
        assert!(re.is_match("irm -r /tempZone/home/user/collection"));
        assert!(re.is_match("irm -f /tempZone/home/user/data"));
        assert!(!re.is_match("ils /tempZone/home/user/"));
        assert!(!re.is_match("iput localfile /tempZone/home/user/"));
        assert!(!re.is_match("iget /tempZone/home/user/file.txt"));
    }

    #[test]
    fn irods_irmtrash_matches() {
        let re = Regex::new(rule_by_id(&TRANSFER_PACK, "irods_irmtrash").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("irmtrash"));
        assert!(re.is_match("irmtrash -M"));
        assert!(re.is_match("irmtrash --admin"));
        assert!(!re.is_match("irm /tempZone/home/user/file.txt"));
        assert!(!re.is_match("ils /tempZone/trash/"));
    }

    // -- rclone --

    #[test]
    fn rclone_delete_matches() {
        let re = Regex::new(rule_by_id(&TRANSFER_PACK, "rclone_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("rclone delete remote:bucket/path"));
        assert!(re.is_match("rclone purge remote:bucket"));
        assert!(re.is_match("rclone delete --min-age 30d remote:data/"));
        assert!(re.is_match("rclone purge --dry-run remote:bucket"));
        assert!(!re.is_match("rclone copy remote:src local-dest"));
        assert!(!re.is_match("rclone ls remote:bucket"));
        assert!(!re.is_match("rclone sync src:path dst:path"));
        assert!(!re.is_match("rclone move remote:old remote:new"));
    }

    #[test]
    fn rclone_sync_delete_matches() {
        let re = Regex::new(rule_by_id(&TRANSFER_PACK, "rclone_sync_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("rclone sync /local/path remote:bucket"));
        assert!(re.is_match("rclone sync remote:src remote:dst"));
        assert!(re.is_match("rclone sync --dry-run src:path dst:path"));
        assert!(!re.is_match("rclone copy remote:src local-dest"));
        assert!(!re.is_match("rclone ls remote:bucket"));
        assert!(!re.is_match("rclone delete remote:bucket"));
        assert!(!re.is_match("rclone move remote:old remote:new"));
    }
}
