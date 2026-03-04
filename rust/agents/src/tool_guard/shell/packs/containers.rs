//! Container packs: `containers.docker`, `containers.kubectl`.

use super::{Confidence, Pack, PatternRule, destructive_pattern, has_dry_run, tokenize_or_bail};

// ---------------------------------------------------------------------------
// containers.docker
// ---------------------------------------------------------------------------

pub static DOCKER_PACK: Pack = Pack {
    id: "containers.docker",
    name: "Docker",
    description: "Guards against destructive Docker operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!(
            "system_prune",
            r"\bdocker\s+system\s+prune\b",
            "Removes all unused containers, networks, images, and optionally volumes",
            "Use `docker container prune` or `docker image prune` for targeted cleanup",
            Confidence::High
        ),
        destructive_pattern!(
            "volume_prune",
            r"\bdocker\s+volume\s+prune\b",
            "Permanently deletes all unused volumes and their data",
            "List volumes with `docker volume ls` and remove specific ones",
            Confidence::High
        ),
        destructive_pattern!(
            "force_remove",
            r"\bdocker\s+(?:rm|rmi)\b.*(?:--force|-f)\b",
            "Force-removes running containers or in-use images",
            "Stop containers first with `docker stop`, then remove",
            Confidence::Medium
        ),
    ],
};

// ---------------------------------------------------------------------------
// containers.kubectl
// ---------------------------------------------------------------------------

/// Validator for `drain_node`: returns `false` if `--dry-run` is present.
fn drain_no_dryrun_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    !has_dry_run(&tokens)
}

pub static KUBECTL_PACK: Pack = Pack {
    id: "containers.kubectl",
    name: "Kubernetes",
    description: "Guards against destructive Kubernetes operations",
    safe_patterns: &[],
    destructive_patterns: &[
        destructive_pattern!(
            "delete_namespace",
            r"\bkubectl\s+delete\s+(?:namespace|ns)\b",
            "Deletes all resources in the namespace",
            "Delete specific resources within the namespace instead",
            Confidence::High
        ),
        destructive_pattern!(
            "delete_all",
            r"\bkubectl\s+delete\b.*(?:--all\b|--all-namespaces\b|-A\b)",
            "Mass-deletes resources across scopes",
            "Delete specific resources by name",
            Confidence::High
        ),
        destructive_pattern!(
            "drain_node",
            r"\bkubectl\s+drain\b",
            drain_no_dryrun_validator,
            "Evicts all pods from a node",
            "Use `kubectl drain --dry-run=client` first to preview",
            Confidence::Medium
        ),
    ],
};

#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::super::tests::rule_by_id;
    use super::*;

    // -- Docker --

    #[test]
    fn system_prune_matches() {
        let re = Regex::new(rule_by_id(&DOCKER_PACK, "system_prune").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("docker system prune"));
        assert!(re.is_match("docker system prune -a"));
        assert!(!re.is_match("docker system info"));
    }

    #[test]
    fn volume_prune_matches() {
        let re = Regex::new(rule_by_id(&DOCKER_PACK, "volume_prune").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("docker volume prune"));
        assert!(!re.is_match("docker volume ls"));
    }

    #[test]
    fn force_remove_matches() {
        let re = Regex::new(rule_by_id(&DOCKER_PACK, "force_remove").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("docker rm -f container1"));
        assert!(re.is_match("docker rmi --force image1"));
        assert!(!re.is_match("docker rm container1"));
    }

    // -- Kubernetes --

    #[test]
    fn delete_namespace_matches() {
        let re = Regex::new(rule_by_id(&KUBECTL_PACK, "delete_namespace").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("kubectl delete namespace prod"));
        assert!(re.is_match("kubectl delete ns staging"));
        assert!(!re.is_match("kubectl delete pod my-pod"));
    }

    #[test]
    fn delete_all_matches() {
        let re = Regex::new(rule_by_id(&KUBECTL_PACK, "delete_all").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("kubectl delete pods --all"));
        assert!(re.is_match("kubectl delete pods --all-namespaces"));
        assert!(re.is_match("kubectl delete pods -A"));
        assert!(!re.is_match("kubectl delete pod my-pod"));
    }

    #[test]
    fn drain_validator_cases() {
        assert!(drain_no_dryrun_validator("kubectl drain node1"));
        assert!(!drain_no_dryrun_validator(
            "kubectl drain node1 --dry-run=client"
        ));
    }
}
