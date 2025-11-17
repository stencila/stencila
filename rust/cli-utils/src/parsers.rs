use eyre::{Result, bail};
use url::Host;

/// Parse an input from the command line as a Ghost host
pub fn parse_host(arg: &str) -> Result<Host> {
    // Question mark converts between error types
    Ok(Host::parse(arg)?)
}

/// Parse and validate a domain name (rejects IP addresses and ports)
pub fn parse_domain(arg: &str) -> Result<String> {
    // Check for port numbers
    if arg.contains(':') {
        bail!("Domain cannot contain a port number");
    }

    // Parse as a host to validate format
    let host = Host::parse(arg)?;

    // Reject IP addresses - only accept domain names
    match host {
        Host::Domain(domain) => Ok(domain.to_string()),
        Host::Ipv4(_) => bail!("IP addresses are not allowed, please provide a domain name"),
        Host::Ipv6(_) => bail!("IP addresses are not allowed, please provide a domain name"),
    }
}
