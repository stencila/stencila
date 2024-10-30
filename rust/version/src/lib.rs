use semver::Version;

/// The current version of Stencila as a string
pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The current version of Stencila as a [`semver::Version`]
pub fn stencila_version() -> Version {
    Version::parse(STENCILA_VERSION).expect("version should always be a semver")
}
