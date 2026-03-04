//! Web tool guard: URL/domain-based checks for `web_fetch`.
//!
//! Evaluates URLs against built-in rules for SSRF prevention, metadata
//! endpoint protection, protocol safety, and domain allow/deny lists.

use std::net::{Ipv4Addr, Ipv6Addr};

use url::Url;

use super::{GuardVerdict, TrustLevel};

// ---------------------------------------------------------------------------
// Constants from spec §4.8
// ---------------------------------------------------------------------------

pub const METADATA_HOSTS: &[&str] = &[
    "169.254.169.254",          // AWS, Azure, most cloud providers
    "fd00:ec2::254",            // AWS IMDSv2 IPv6 endpoint
    "metadata.google.internal", // GCP
    "100.100.100.200",          // Alibaba Cloud
];

pub const CREDENTIAL_PATH_PREFIXES: &[&str] = &[
    // AWS IMDSv1/v2
    "/latest/meta-data/iam/security-credentials",
    "/latest/api/token",
    // GCP
    "/computeMetadata/v1/instance/service-accounts",
    // Azure
    "/metadata/identity/oauth2/token",
    // Alibaba Cloud
    "/latest/meta-data/ram/security-credentials",
];

pub const HIGH_RISK_PORTS: &[u16] = &[
    22,    // SSH
    23,    // Telnet
    25,    // SMTP
    135,   // MS RPC
    139,   // NetBIOS
    445,   // SMB
    2375,  // Docker daemon (unencrypted)
    2376,  // Docker daemon (TLS)
    3306,  // MySQL
    5432,  // PostgreSQL
    5900,  // VNC
    6379,  // Redis
    6443,  // Kubernetes API
    8500,  // Consul
    8200,  // Vault
    9200,  // Elasticsearch
    27017, // MongoDB
];

// ---------------------------------------------------------------------------
// Rule reason/suggestion strings (from spec §6.7)
// ---------------------------------------------------------------------------

const CREDENTIAL_URL_REASON: &str = "Metadata credential paths return IAM tokens and secrets that can be used for privilege escalation";
const CREDENTIAL_URL_SUGGESTION: &str =
    "Use the cloud provider's CLI for credential management (e.g., `aws sts get-caller-identity`)";

const METADATA_ENDPOINT_REASON: &str =
    "Cloud metadata endpoints expose instance credentials and configuration";
const METADATA_ENDPOINT_SUGGESTION: &str =
    "Access cloud credentials through the provider's CLI or SDK instead";

const INTERNAL_NETWORK_REASON: &str =
    "Fetching internal network addresses can expose services not meant for external access (SSRF)";
const INTERNAL_NETWORK_SUGGESTION: &str =
    "Use a public URL, or access internal services through an appropriate API";

const NON_HTTPS_REASON: &str = "Unencrypted HTTP requests can expose data in transit";
const NON_HTTPS_SUGGESTION: &str = "Use `https://` instead of `http://`";

const HIGH_RISK_PORT_REASON: &str =
    "Port is associated with an infrastructure service not typically accessed via HTTP";
const HIGH_RISK_PORT_SUGGESTION: &str =
    "Use the service's dedicated CLI or client library instead of HTTP";

const DOMAIN_ALLOWLIST_REASON: &str = "Domain is not in the agent's allowed domain list";
const DOMAIN_ALLOWLIST_SUGGESTION: &str =
    "Add the domain to `allowedDomains` in the agent definition, or use an allowed domain";

const DOMAIN_DENYLIST_REASON: &str = "Domain is in the agent's disallowed domain list";
const DOMAIN_DENYLIST_SUGGESTION: &str =
    "Remove the domain from `disallowedDomains` if access is intended, or use a different source";

const PARSE_FAILURE_REASON: &str = "URL could not be parsed";
const PARSE_FAILURE_SUGGESTION: &str = "Provide a valid URL (e.g., `https://example.com/path`)";

// ---------------------------------------------------------------------------
// WebToolGuard
// ---------------------------------------------------------------------------

pub struct WebToolGuard {
    allowed_domains: Option<Vec<String>>,
    disallowed_domains: Option<Vec<String>>,
}

impl WebToolGuard {
    pub fn new(
        allowed_domains: Option<Vec<String>>,
        disallowed_domains: Option<Vec<String>>,
    ) -> Self {
        Self {
            allowed_domains: normalize_domain_list(allowed_domains),
            disallowed_domains: normalize_domain_list(disallowed_domains),
        }
    }

    pub fn evaluate(&self, url_str: &str, trust_level: TrustLevel) -> GuardVerdict {
        // Step 1: Parse URL. Parse failure → Deny.
        let parsed = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => return parse_failure_deny(),
        };

        let host_raw = match parsed.host_str() {
            Some(h) => h,
            None => return parse_failure_deny(),
        };

        // Step 2: Host normalization — ASCII case-fold + trailing dot strip.
        // url::Url already lowercases ASCII hosts, but we normalize for safety.
        let host = normalize_host(host_raw);

        // Normalize the path: collapse consecutive slashes for credential path matching.
        let path = normalize_path(parsed.path());

        // Step 3: Evaluate rules in most-specific-first order (short-circuiting).

        // Rule 1: web.credential_url — metadata host + credential path
        if is_metadata_host(&host) && has_credential_path(&path) {
            return GuardVerdict::Deny {
                reason: CREDENTIAL_URL_REASON,
                suggestion: CREDENTIAL_URL_SUGGESTION,
                rule_id: "web.credential_url",
            };
        }

        // Rule 2: web.metadata_endpoint — metadata host (any path)
        if is_metadata_host(&host) {
            return GuardVerdict::Deny {
                reason: METADATA_ENDPOINT_REASON,
                suggestion: METADATA_ENDPOINT_SUGGESTION,
                rule_id: "web.metadata_endpoint",
            };
        }

        // Rule 3: web.internal_network — localhost, private/loopback IP, *.local, *.internal
        if is_internal_network(&host) {
            return GuardVerdict::Deny {
                reason: INTERNAL_NETWORK_REASON,
                suggestion: INTERNAL_NETWORK_SUGGESTION,
                rule_id: "web.internal_network",
            };
        }

        // Rule 4: web.non_https — scheme is http
        if parsed.scheme() == "http" {
            let verdict = match trust_level {
                TrustLevel::Low => GuardVerdict::Deny {
                    reason: NON_HTTPS_REASON,
                    suggestion: NON_HTTPS_SUGGESTION,
                    rule_id: "web.non_https",
                },
                TrustLevel::Medium => GuardVerdict::Warn {
                    reason: NON_HTTPS_REASON,
                    suggestion: NON_HTTPS_SUGGESTION,
                    rule_id: "web.non_https",
                },
                TrustLevel::High => GuardVerdict::Allow,
            };
            if !matches!(verdict, GuardVerdict::Allow) {
                return verdict;
            }
        }

        // Rule 5: web.high_risk_port — port in HIGH_RISK_PORTS
        if let Some(port) = parsed.port() {
            if HIGH_RISK_PORTS.contains(&port) {
                let verdict = match trust_level {
                    TrustLevel::Low => GuardVerdict::Deny {
                        reason: HIGH_RISK_PORT_REASON,
                        suggestion: HIGH_RISK_PORT_SUGGESTION,
                        rule_id: "web.high_risk_port",
                    },
                    TrustLevel::Medium => GuardVerdict::Warn {
                        reason: HIGH_RISK_PORT_REASON,
                        suggestion: HIGH_RISK_PORT_SUGGESTION,
                        rule_id: "web.high_risk_port",
                    },
                    TrustLevel::High => GuardVerdict::Allow,
                };
                if !matches!(verdict, GuardVerdict::Allow) {
                    return verdict;
                }
            }
        }

        // Rule 6: Domain allowlist/denylist.
        // Allowlist precedence: if allowedDomains is set, disallowedDomains is ignored.
        if let Some(ref allowed) = self.allowed_domains {
            if !domain_matches_list(&host, allowed) {
                return GuardVerdict::Deny {
                    reason: DOMAIN_ALLOWLIST_REASON,
                    suggestion: DOMAIN_ALLOWLIST_SUGGESTION,
                    rule_id: "web.domain_allowlist",
                };
            }
        } else if let Some(ref disallowed) = self.disallowed_domains {
            if domain_matches_list(&host, disallowed) {
                return GuardVerdict::Deny {
                    reason: DOMAIN_DENYLIST_REASON,
                    suggestion: DOMAIN_DENYLIST_SUGGESTION,
                    rule_id: "web.domain_denylist",
                };
            }
        }

        // Default: Allow if no rule fires.
        GuardVerdict::Allow
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_failure_deny() -> GuardVerdict {
    GuardVerdict::Deny {
        reason: PARSE_FAILURE_REASON,
        suggestion: PARSE_FAILURE_SUGGESTION,
        rule_id: "web.parse_failure",
    }
}

fn normalize_domain_list(list: Option<Vec<String>>) -> Option<Vec<String>> {
    list.map(|domains| domains.into_iter().map(|d| normalize_host(&d)).collect())
}

// ---------------------------------------------------------------------------
// Host normalization
// ---------------------------------------------------------------------------

/// Normalize a host string: ASCII lowercase + strip trailing dot.
fn normalize_host(host: &str) -> String {
    let lowered = host.to_ascii_lowercase();
    lowered.strip_suffix('.').unwrap_or(&lowered).to_string()
}

/// Normalize a URL path: collapse consecutive slashes.
fn normalize_path(path: &str) -> String {
    let mut result = String::with_capacity(path.len());
    let mut prev_slash = false;
    for ch in path.chars() {
        if ch == '/' {
            if !prev_slash {
                result.push('/');
            }
            prev_slash = true;
        } else {
            prev_slash = false;
            result.push(ch);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Metadata host detection
// ---------------------------------------------------------------------------

fn is_metadata_host(host: &str) -> bool {
    // METADATA_HOSTS entries are already lowercase; `host` is pre-normalized.
    // IPv6 hosts from url::Url::host_str() are wrapped in brackets — strip them.
    let bare = host
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host);
    METADATA_HOSTS.iter().any(|&h| bare == h)
}

// ---------------------------------------------------------------------------
// Credential path detection
// ---------------------------------------------------------------------------

fn has_credential_path(normalized_path: &str) -> bool {
    CREDENTIAL_PATH_PREFIXES
        .iter()
        .any(|prefix| normalized_path.starts_with(prefix))
}

// ---------------------------------------------------------------------------
// Internal network detection (spec §4.8)
// ---------------------------------------------------------------------------

fn is_internal_network(host: &str) -> bool {
    // Check for "localhost"
    if host == "localhost" {
        return true;
    }

    // Check for *.local or *.internal hostname suffixes
    if host.ends_with(".local") || host.ends_with(".internal") {
        return true;
    }

    // Check for private/loopback IPv4
    if let Ok(ipv4) = host.parse::<Ipv4Addr>() {
        return is_private_ipv4(ipv4);
    }

    // Check for IPv6 — url crate strips brackets from [::1]
    let ipv6_str = host
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(ipv6) = ipv6_str.parse::<Ipv6Addr>() {
        return is_private_ipv6(ipv6);
    }

    false
}

fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    // 127.0.0.0/8 (loopback)
    if octets[0] == 127 {
        return true;
    }
    // 10.0.0.0/8
    if octets[0] == 10 {
        return true;
    }
    // 172.16.0.0/12
    if octets[0] == 172 && (16..=31).contains(&octets[1]) {
        return true;
    }
    // 192.168.0.0/16
    if octets[0] == 192 && octets[1] == 168 {
        return true;
    }
    // 169.254.0.0/16 (link-local)
    if octets[0] == 169 && octets[1] == 254 {
        return true;
    }
    // 100.64.0.0/10 (shared address space / CGNAT — includes 100.100.100.200)
    if octets[0] == 100 && (64..=127).contains(&octets[1]) {
        return true;
    }
    false
}

fn is_private_ipv6(ip: Ipv6Addr) -> bool {
    // ::1 (loopback)
    if ip == Ipv6Addr::LOCALHOST {
        return true;
    }
    let segments = ip.segments();
    // fe80::/10 (link-local)
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }
    // fc00::/7 (unique-local)
    if segments[0] & 0xfe00 == 0xfc00 {
        return true;
    }
    false
}

// ---------------------------------------------------------------------------
// Domain matching
// ---------------------------------------------------------------------------

/// Check whether `host` matches any entry in a domain list.
///
/// Entries starting with `*.` are wildcard patterns that match subdomains
/// only (not the bare domain). For example, `*.example.com` matches
/// `sub.example.com` but NOT `example.com`.
///
/// All other entries are exact matches.
///
/// Both `host` and entries must already be normalized (lowercase, no trailing dot).
fn domain_matches_list(host: &str, domains: &[String]) -> bool {
    for entry in domains {
        if let Some(suffix) = entry.strip_prefix("*.") {
            // Wildcard: host must end with ".suffix" (subdomain only, not bare domain)
            if host.ends_with(suffix) && host.len() > suffix.len() {
                // Ensure there's a dot before the suffix
                let prefix_len = host.len() - suffix.len();
                if host.as_bytes()[prefix_len - 1] == b'.' {
                    return true;
                }
            }
        } else if host == entry.as_str() {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn guard(allowed: Option<Vec<&str>>, disallowed: Option<Vec<&str>>) -> WebToolGuard {
        WebToolGuard::new(
            allowed.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
            disallowed.map(|v| v.into_iter().map(|s| s.to_string()).collect()),
        )
    }

    fn default_guard() -> WebToolGuard {
        guard(None, None)
    }

    fn verdict_rule_id(v: &GuardVerdict) -> &str {
        match v {
            GuardVerdict::Deny { rule_id, .. } | GuardVerdict::Warn { rule_id, .. } => rule_id,
            GuardVerdict::Allow => "allow",
        }
    }

    fn is_deny(v: &GuardVerdict) -> bool {
        matches!(v, GuardVerdict::Deny { .. })
    }

    fn is_warn(v: &GuardVerdict) -> bool {
        matches!(v, GuardVerdict::Warn { .. })
    }

    fn is_allow(v: &GuardVerdict) -> bool {
        matches!(v, GuardVerdict::Allow)
    }

    // -----------------------------------------------------------------------
    // Parse failure
    // -----------------------------------------------------------------------

    #[test]
    fn parse_failure_denies() {
        let g = default_guard();
        let v = g.evaluate("not a url", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.parse_failure");
    }

    // -----------------------------------------------------------------------
    // web.credential_url
    // -----------------------------------------------------------------------

    #[test]
    fn credential_url_aws_iam() {
        let g = default_guard();
        let v = g.evaluate(
            "http://169.254.169.254/latest/meta-data/iam/security-credentials/",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn credential_url_aws_token() {
        let g = default_guard();
        let v = g.evaluate("http://169.254.169.254/latest/api/token", TrustLevel::High);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn credential_url_gcp() {
        let g = default_guard();
        let v = g.evaluate(
            "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token",
            TrustLevel::Low,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn credential_url_azure() {
        let g = default_guard();
        let v = g.evaluate(
            "http://169.254.169.254/metadata/identity/oauth2/token?api-version=2018-02-01",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn credential_url_alibaba() {
        let g = default_guard();
        let v = g.evaluate(
            "http://100.100.100.200/latest/meta-data/ram/security-credentials/role-name",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn credential_url_fires_before_internal_network() {
        // Spec §13: 169.254.169.254 with credential path fires credential_url, not internal_network
        let g = default_guard();
        let v = g.evaluate(
            "http://169.254.169.254/latest/meta-data/iam/security-credentials/role",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn credential_url_fires_before_metadata_endpoint() {
        // credential_url is more specific than metadata_endpoint
        let g = default_guard();
        let v = g.evaluate(
            "http://169.254.169.254/latest/meta-data/iam/security-credentials/",
            TrustLevel::Medium,
        );
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    // -----------------------------------------------------------------------
    // web.metadata_endpoint
    // -----------------------------------------------------------------------

    #[test]
    fn metadata_endpoint_non_credential_path() {
        let g = default_guard();
        let v = g.evaluate(
            "http://169.254.169.254/latest/meta-data/instance-id",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.metadata_endpoint");
    }

    #[test]
    fn metadata_endpoint_gcp_non_credential() {
        // Spec §13: metadata.google.internal (non-credential path) fires metadata_endpoint
        let g = default_guard();
        let v = g.evaluate(
            "http://metadata.google.internal/computeMetadata/v1/instance/hostname",
            TrustLevel::High,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.metadata_endpoint");
    }

    #[test]
    fn metadata_endpoint_gcp_fires_not_internal_network() {
        // Spec §13: metadata.google.internal fires metadata_endpoint, not internal_network
        let g = default_guard();
        let v = g.evaluate(
            "http://metadata.google.internal/some/path",
            TrustLevel::Medium,
        );
        assert_eq!(verdict_rule_id(&v), "web.metadata_endpoint");
    }

    #[test]
    fn credential_url_aws_ipv6() {
        let g = default_guard();
        let v = g.evaluate(
            "http://[fd00:ec2::254]/latest/meta-data/iam/security-credentials/role",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn metadata_endpoint_aws_ipv6_non_credential() {
        let g = default_guard();
        let v = g.evaluate(
            "http://[fd00:ec2::254]/latest/meta-data/instance-id",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.metadata_endpoint");
    }

    #[test]
    fn aws_ipv6_metadata_fires_not_internal_network() {
        // fd00:ec2::254 is in fc00::/7 (unique-local), but should fire
        // metadata_endpoint rather than internal_network for audit precision
        let g = default_guard();
        let v = g.evaluate("http://[fd00:ec2::254]/some/path", TrustLevel::Medium);
        assert_eq!(verdict_rule_id(&v), "web.metadata_endpoint");
    }

    #[test]
    fn metadata_endpoint_alibaba_non_credential() {
        let g = default_guard();
        let v = g.evaluate(
            "http://100.100.100.200/latest/meta-data/instance-id",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.metadata_endpoint");
    }

    #[test]
    fn metadata_denied_at_all_trust_levels() {
        let g = default_guard();
        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = g.evaluate("http://169.254.169.254/latest/meta-data/", trust);
            assert!(is_deny(&v), "expected deny at {trust:?}, got {v:?}");
        }
    }

    // -----------------------------------------------------------------------
    // web.internal_network
    // -----------------------------------------------------------------------

    #[test]
    fn internal_network_localhost() {
        let g = default_guard();
        let v = g.evaluate("http://localhost:8080/api", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_10_range() {
        let g = default_guard();
        let v = g.evaluate("http://10.0.0.1/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_172_16_range() {
        let g = default_guard();
        let v = g.evaluate("http://172.16.0.1/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_192_168_range() {
        let g = default_guard();
        let v = g.evaluate("http://192.168.1.1/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_127_loopback() {
        let g = default_guard();
        let v = g.evaluate("http://127.0.0.1/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_ipv6_loopback() {
        let g = default_guard();
        let v = g.evaluate("http://[::1]/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_ipv6_link_local() {
        let g = default_guard();
        let v = g.evaluate("http://[fe80::1]/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_ipv6_unique_local() {
        let g = default_guard();
        let v = g.evaluate("http://[fd00::1]/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_dot_local_suffix() {
        let g = default_guard();
        let v = g.evaluate("http://myservice.local/api", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_dot_internal_suffix() {
        // Non-metadata .internal host
        let g = default_guard();
        let v = g.evaluate("http://myservice.internal/api", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn internal_network_denied_at_all_trust_levels() {
        let g = default_guard();
        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = g.evaluate("http://localhost/", trust);
            assert!(is_deny(&v), "expected deny at {trust:?}, got {v:?}");
            assert_eq!(verdict_rule_id(&v), "web.internal_network");
        }
    }

    // -----------------------------------------------------------------------
    // web.non_https
    // -----------------------------------------------------------------------

    #[test]
    fn non_https_warn_at_medium() {
        let g = default_guard();
        let v = g.evaluate("http://example.com/api", TrustLevel::Medium);
        assert!(is_warn(&v));
        assert_eq!(verdict_rule_id(&v), "web.non_https");
    }

    #[test]
    fn non_https_deny_at_low() {
        let g = default_guard();
        let v = g.evaluate("http://example.com/", TrustLevel::Low);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.non_https");
    }

    #[test]
    fn non_https_allow_at_high() {
        let g = default_guard();
        let v = g.evaluate("http://example.com/", TrustLevel::High);
        assert!(is_allow(&v));
    }

    #[test]
    fn https_not_flagged() {
        let g = default_guard();
        let v = g.evaluate("https://example.com/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    // -----------------------------------------------------------------------
    // web.high_risk_port
    // -----------------------------------------------------------------------

    #[test]
    fn high_risk_port_6379_warn_at_medium() {
        let g = default_guard();
        let v = g.evaluate("https://example.com:6379/", TrustLevel::Medium);
        assert!(is_warn(&v));
        assert_eq!(verdict_rule_id(&v), "web.high_risk_port");
    }

    #[test]
    fn high_risk_port_deny_at_low() {
        let g = default_guard();
        let v = g.evaluate("https://example.com:22/", TrustLevel::Low);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.high_risk_port");
    }

    #[test]
    fn high_risk_port_allow_at_high() {
        let g = default_guard();
        let v = g.evaluate("https://example.com:22/", TrustLevel::High);
        assert!(is_allow(&v));
    }

    #[test]
    fn all_high_risk_ports_flagged_at_medium() {
        let g = default_guard();
        for &port in HIGH_RISK_PORTS {
            let url = format!("https://example.com:{port}/");
            let v = g.evaluate(&url, TrustLevel::Medium);
            assert!(is_warn(&v), "expected warn for port {port}, got {v:?}");
            assert_eq!(verdict_rule_id(&v), "web.high_risk_port");
        }
    }

    #[test]
    fn normal_port_not_flagged() {
        let g = default_guard();
        let v = g.evaluate("https://example.com:8080/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn no_explicit_port_not_flagged() {
        let g = default_guard();
        let v = g.evaluate("https://example.com/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    // -----------------------------------------------------------------------
    // web.domain_allowlist
    // -----------------------------------------------------------------------

    #[test]
    fn allowlist_blocks_unlisted_domain() {
        let g = guard(Some(vec!["docs.rs"]), None);
        let v = g.evaluate("https://evil.com/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.domain_allowlist");
    }

    #[test]
    fn allowlist_allows_listed_domain() {
        let g = guard(Some(vec!["docs.rs"]), None);
        let v = g.evaluate("https://docs.rs/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn allowlist_denied_at_all_trust_levels() {
        let g = guard(Some(vec!["docs.rs"]), None);
        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = g.evaluate("https://evil.com/", trust);
            assert!(is_deny(&v), "expected deny at {trust:?}, got {v:?}");
        }
    }

    // -----------------------------------------------------------------------
    // web.domain_denylist
    // -----------------------------------------------------------------------

    #[test]
    fn denylist_blocks_listed_domain() {
        let g = guard(None, Some(vec!["evil.com"]));
        let v = g.evaluate("https://evil.com/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.domain_denylist");
    }

    #[test]
    fn denylist_allows_unlisted_domain() {
        let g = guard(None, Some(vec!["evil.com"]));
        let v = g.evaluate("https://good.com/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    // -----------------------------------------------------------------------
    // Allowlist precedence over denylist
    // -----------------------------------------------------------------------

    #[test]
    fn allowlist_takes_precedence_over_denylist() {
        let g = guard(Some(vec!["docs.rs"]), Some(vec!["docs.rs"]));
        // With allowlist set, denylist is ignored. docs.rs is in the allowlist → Allow.
        let v = g.evaluate("https://docs.rs/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn allowlist_takes_precedence_over_denylist_blocking() {
        // With both set, allowlist wins. evil.com is not in allowlist → Deny (allowlist rule).
        let g = guard(Some(vec!["docs.rs"]), Some(vec!["evil.com"]));
        let v = g.evaluate("https://evil.com/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.domain_allowlist");
    }

    // -----------------------------------------------------------------------
    // Wildcard domain matching
    // -----------------------------------------------------------------------

    #[test]
    fn wildcard_matches_subdomain() {
        let g = guard(Some(vec!["*.example.com"]), None);
        let v = g.evaluate("https://sub.example.com/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn wildcard_does_not_match_bare_domain() {
        let g = guard(Some(vec!["*.example.com"]), None);
        let v = g.evaluate("https://example.com/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.domain_allowlist");
    }

    #[test]
    fn wildcard_matches_deep_subdomain() {
        let g = guard(Some(vec!["*.example.com"]), None);
        let v = g.evaluate("https://a.b.example.com/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn wildcard_denylist() {
        let g = guard(None, Some(vec!["*.evil.com"]));
        let v = g.evaluate("https://sub.evil.com/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.domain_denylist");
    }

    #[test]
    fn wildcard_denylist_bare_not_matched() {
        let g = guard(None, Some(vec!["*.evil.com"]));
        let v = g.evaluate("https://evil.com/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    // -----------------------------------------------------------------------
    // Host normalization
    // -----------------------------------------------------------------------

    #[test]
    fn host_case_insensitive() {
        let g = guard(Some(vec!["example.com"]), None);
        let v = g.evaluate("https://EXAMPLE.COM/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn trailing_dot_stripped() {
        let g = guard(Some(vec!["docs.rs"]), None);
        // url crate normalizes trailing dots in hosts
        let v = g.evaluate("https://docs.rs./", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn domain_list_entries_normalized() {
        // Domain list entry with trailing dot and uppercase
        let g = guard(Some(vec!["DOCS.RS."]), None);
        let v = g.evaluate("https://docs.rs/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    // -----------------------------------------------------------------------
    // Default verdict (unmatched → Allow)
    // -----------------------------------------------------------------------

    #[test]
    fn unmatched_url_allows_at_all_trust_levels() {
        let g = default_guard();
        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = g.evaluate("https://example.com/", trust);
            assert!(is_allow(&v), "expected allow at {trust:?}, got {v:?}");
        }
    }

    // -----------------------------------------------------------------------
    // Spec examples W1, W2, W3
    // -----------------------------------------------------------------------

    #[test]
    fn spec_example_w1() {
        // W1: credential_url at any trust level
        let g = default_guard();
        for trust in [TrustLevel::Low, TrustLevel::Medium, TrustLevel::High] {
            let v = g.evaluate(
                "http://169.254.169.254/latest/meta-data/iam/security-credentials/",
                trust,
            );
            assert!(is_deny(&v), "expected deny at {trust:?}, got {v:?}");
            assert_eq!(verdict_rule_id(&v), "web.credential_url");
        }
    }

    #[test]
    fn spec_example_w2() {
        // W2: http://example.com/api at medium → Warn (non_https)
        let g = default_guard();
        let v = g.evaluate("http://example.com/api", TrustLevel::Medium);
        assert!(is_warn(&v));
        assert_eq!(verdict_rule_id(&v), "web.non_https");
    }

    #[test]
    fn spec_example_w3() {
        // W3: https://evil.com with allowedDomains: ["*.docs.rs"] → Deny
        let g = guard(Some(vec!["*.docs.rs"]), None);
        let v = g.evaluate("https://evil.com", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.domain_allowlist");
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn credential_path_with_consecutive_slashes() {
        // Consecutive slashes in path should be normalized
        let g = default_guard();
        let v = g.evaluate(
            "http://169.254.169.254//latest//meta-data//iam//security-credentials/",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn ftp_scheme_not_flagged_as_non_https() {
        // non_https rule only fires for http scheme
        let g = default_guard();
        let v = g.evaluate("ftp://example.com/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }

    #[test]
    fn built_in_rules_take_precedence_over_allowlist() {
        // Even if a metadata host is in the allowlist, built-in rules fire first
        let g = guard(Some(vec!["169.254.169.254"]), None);
        let v = g.evaluate(
            "http://169.254.169.254/latest/meta-data/iam/security-credentials/",
            TrustLevel::Medium,
        );
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.credential_url");
    }

    #[test]
    fn http_with_high_risk_port_fires_non_https_first() {
        // non_https comes before high_risk_port in evaluation order
        let g = default_guard();
        let v = g.evaluate("http://example.com:6379/", TrustLevel::Medium);
        assert!(is_warn(&v));
        assert_eq!(verdict_rule_id(&v), "web.non_https");
    }

    #[test]
    fn ipv4_169_254_not_metadata_but_internal() {
        // 169.254.x.x that isn't 169.254.169.254 should fire internal_network
        let g = default_guard();
        let v = g.evaluate("http://169.254.1.1/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn ipv4_100_64_range_internal() {
        // 100.64.0.0/10 is shared address space
        let g = default_guard();
        let v = g.evaluate("http://100.64.0.1/", TrustLevel::Medium);
        assert!(is_deny(&v));
        assert_eq!(verdict_rule_id(&v), "web.internal_network");
    }

    #[test]
    fn public_ip_not_internal() {
        let g = default_guard();
        let v = g.evaluate("https://8.8.8.8/", TrustLevel::Medium);
        assert!(is_allow(&v));
    }
}
