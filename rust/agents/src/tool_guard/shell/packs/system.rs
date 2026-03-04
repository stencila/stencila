//! System packs: `system.disk`, `system.network`, `system.services`.

use super::tokenize_or_bail;
use super::{Confidence, Pack, PatternRule, destructive_pattern, has_token_prefix};

/// Validator for `dd_to_device`: checks for `of=/dev/...`.
fn dd_to_device_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    has_token_prefix(&tokens, "of=/dev/")
}

/// Validator for `fdisk_parted`: returns `false` if in print/list mode.
fn fdisk_parted_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    // Exclude print/list modes
    !tokens
        .iter()
        .any(|t| t.value == "-l" || t.value == "print" || t.value == "--list")
}

pub static DISK_PACK: Pack = Pack {
    id: "system.disk",
    name: "Disk",
    description: "Guards against destructive disk operations",
    destructive_patterns: &[
        destructive_pattern!(
            "dd_to_device",
            r"\bdd\b",
            dd_to_device_validator,
            "Writing directly to devices can destroy partitions and data",
            "Specify a file path instead of a device",
            Confidence::High
        ),
        destructive_pattern!(
            "mkfs",
            r"\bmkfs\b",
            "Formatting a device destroys all data on it",
            "Verify the target device with `lsblk` first",
            Confidence::High
        ),
        destructive_pattern!(
            "fdisk_parted",
            r"\b(?:fdisk|parted|gdisk)\b",
            fdisk_parted_validator,
            "Partition changes can cause data loss",
            "Use `fdisk -l` or `parted print` to inspect first",
            Confidence::Medium
        ),
    ],
};

pub static NETWORK_PACK: Pack = Pack {
    id: "system.network",
    name: "Network",
    description: "Guards against destructive network operations",
    destructive_patterns: &[
        destructive_pattern!(
            "iptables_flush",
            r"\b(?:iptables|ip6tables)\s+(?:-F|--flush)\b",
            "Flushing firewall rules removes all network security policies",
            "Use `iptables -L` to list rules first; save with `iptables-save`",
            Confidence::High
        ),
        destructive_pattern!(
            "route_delete",
            r"\b(?:ip\s+route\s+(?:del|flush)|route\s+del)\b",
            "Deleting routes can cause network connectivity loss",
            "Use `ip route show` to review routes before modification",
            Confidence::Medium
        ),
        destructive_pattern!(
            "interface_down",
            r"\b(?:ifconfig\s+\w+\s+down|ip\s+link\s+set\s+\w+\s+down)\b",
            "Bringing down a network interface disrupts connectivity",
            "Ensure you have alternative access before modifying interfaces",
            Confidence::Medium
        ),
    ],
};

/// Validator for `systemctl_destructive`: fires only when the target service
/// looks critical (ssh, network, firewall, docker, kubelet, systemd).
fn systemctl_critical_validator(cmd: &str) -> bool {
    let tokens = tokenize_or_bail!(cmd, true);
    const CRITICAL: &[&str] = &["ssh", "network", "firewall", "docker", "kubelet", "systemd"];
    tokens.iter().any(|t| {
        let v = &t.value;
        // Skip flags (start with '-') and the command/sub-command tokens
        if v.starts_with('-')
            || *v == "systemctl"
            || *v == "stop"
            || *v == "disable"
            || *v == "mask"
        {
            return false;
        }
        let lower = v.to_ascii_lowercase();
        CRITICAL.iter().any(|kw| lower.contains(kw))
    })
}

pub static SERVICES_PACK: Pack = Pack {
    id: "system.services",
    name: "Services",
    description: "Guards against destructive system service operations",
    destructive_patterns: &[
        destructive_pattern!(
            "systemctl_destructive",
            r"\bsystemctl\s+(?:stop|disable|mask)\b",
            systemctl_critical_validator,
            "Stopping or disabling critical services can break the system",
            "Use `systemctl status` to check service state first",
            Confidence::Medium
        ),
        destructive_pattern!(
            "service_stop",
            r"\bservice\s+\w+\s+stop\b",
            "Stopping services can disrupt running applications",
            "Use `service <name> status` to check before stopping",
            Confidence::Medium
        ),
        destructive_pattern!(
            "kill_signal",
            r"\bkill\s+-9\b",
            "SIGKILL terminates processes without cleanup",
            "Use `kill` (SIGTERM) first to allow graceful shutdown",
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
    fn dd_to_device_validator_cases() {
        assert!(dd_to_device_validator("dd if=/dev/zero of=/dev/sda bs=1M"));
        assert!(!dd_to_device_validator(
            "dd if=/dev/zero of=output.img bs=1M"
        ));
    }

    #[test]
    fn mkfs_matches() {
        let re =
            Regex::new(rule_by_id(&DISK_PACK, "mkfs").pattern).expect("pattern should compile");
        assert!(re.is_match("mkfs.ext4 /dev/sda1"));
        assert!(re.is_match("mkfs /dev/sda1"));
        assert!(!re.is_match("check_fs /dev/sda1"));
    }

    #[test]
    fn fdisk_parted_validator_cases() {
        assert!(fdisk_parted_validator("fdisk /dev/sda"));
        assert!(!fdisk_parted_validator("fdisk -l"));
        assert!(fdisk_parted_validator("parted /dev/sda"));
        assert!(!fdisk_parted_validator("parted print"));
        assert!(!fdisk_parted_validator("fdisk --list"));
    }

    // -- Network pack tests --

    #[test]
    fn iptables_flush_matches() {
        let re = Regex::new(rule_by_id(&NETWORK_PACK, "iptables_flush").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("iptables -F"));
        assert!(re.is_match("iptables --flush"));
        assert!(re.is_match("ip6tables -F"));
        assert!(re.is_match("ip6tables --flush"));
        assert!(!re.is_match("iptables -L"));
        assert!(!re.is_match("iptables -A INPUT -j DROP"));
    }

    #[test]
    fn route_delete_matches() {
        let re = Regex::new(rule_by_id(&NETWORK_PACK, "route_delete").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("ip route del default"));
        assert!(re.is_match("ip route flush table main"));
        assert!(re.is_match("route del default"));
        assert!(!re.is_match("ip route show"));
        assert!(!re.is_match("ip route add default via 10.0.0.1"));
    }

    #[test]
    fn interface_down_matches() {
        let re = Regex::new(rule_by_id(&NETWORK_PACK, "interface_down").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("ifconfig eth0 down"));
        assert!(re.is_match("ip link set eth0 down"));
        assert!(!re.is_match("ifconfig eth0 up"));
        assert!(!re.is_match("ip link set eth0 up"));
    }

    // -- Services pack tests --

    #[test]
    fn systemctl_critical_validator_cases() {
        assert!(systemctl_critical_validator("systemctl stop sshd"));
        assert!(systemctl_critical_validator("systemctl disable docker"));
        assert!(systemctl_critical_validator(
            "systemctl mask NetworkManager"
        ));
        assert!(systemctl_critical_validator("systemctl stop firewalld"));
        assert!(systemctl_critical_validator("systemctl disable kubelet"));
        assert!(systemctl_critical_validator(
            "systemctl stop systemd-resolved"
        ));
        // Non-critical services should not fire
        assert!(!systemctl_critical_validator("systemctl stop myapp"));
        assert!(!systemctl_critical_validator("systemctl disable nginx"));
    }

    #[test]
    fn systemctl_destructive_matches() {
        let re = Regex::new(rule_by_id(&SERVICES_PACK, "systemctl_destructive").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("systemctl stop sshd"));
        assert!(re.is_match("systemctl disable docker"));
        assert!(re.is_match("systemctl mask firewalld"));
        assert!(!re.is_match("systemctl start sshd"));
        assert!(!re.is_match("systemctl status docker"));
        assert!(!re.is_match("systemctl restart nginx"));
    }

    #[test]
    fn service_stop_matches() {
        let re = Regex::new(rule_by_id(&SERVICES_PACK, "service_stop").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("service nginx stop"));
        assert!(re.is_match("service sshd stop"));
        assert!(!re.is_match("service nginx start"));
        assert!(!re.is_match("service nginx status"));
    }

    #[test]
    fn kill_signal_matches() {
        let re = Regex::new(rule_by_id(&SERVICES_PACK, "kill_signal").pattern)
            .expect("pattern should compile");
        assert!(re.is_match("kill -9 1234"));
        assert!(!re.is_match("kill 1234"));
        assert!(!re.is_match("kill -15 1234"));
    }
}
