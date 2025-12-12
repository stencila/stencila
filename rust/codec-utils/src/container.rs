//! Container environment detection utilities

use std::{env, fs, path::Path};

/// Detect if running inside a container (Docker, Podman, etc.)
///
/// Checks multiple indicators:
/// - `/.dockerenv` file (Docker)
/// - `/run/.containerenv` file (Podman)
/// - `container` environment variable
/// - `/proc/1/cgroup` containing container-related strings
pub fn is_in_container() -> bool {
    // Check for Docker indicator file
    if Path::new("/.dockerenv").exists() {
        return true;
    }

    // Check for Podman indicator file
    if Path::new("/run/.containerenv").exists() {
        return true;
    }

    // Check environment variable (set by some container runtimes)
    if env::var("container").is_ok() {
        return true;
    }

    // Check cgroup for container indicators (Linux only)
    if let Ok(cgroup) = fs::read_to_string("/proc/1/cgroup")
        && (cgroup.contains("docker")
            || cgroup.contains("kubepods")
            || cgroup.contains("lxc")
            || cgroup.contains("containerd"))
    {
        return true;
    }

    false
}
