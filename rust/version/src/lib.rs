use semver::Version;

/// The current version of Stencila as a string
pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A versioned User-Agent header for making requests
///
/// Includes and email as required by some APIs
pub const STENCILA_USER_AGENT: &str = concat!(
    "Stencila/",
    env!("CARGO_PKG_VERSION"),
    " (mailto:user-agent@stencila.io)"
);

/// The current version of Stencila as a [`semver::Version`]
pub fn stencila_version() -> Version {
    Version::parse(STENCILA_VERSION).expect("version should always be a semver")
}
