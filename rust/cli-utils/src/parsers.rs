use eyre::Result;
use url::Host;

/// Parse an input from the command line as a Ghost host
pub fn parse_host(arg: &str) -> Result<Host> {
    // Question mark converts between error types
    Ok(Host::parse(arg)?)
}
