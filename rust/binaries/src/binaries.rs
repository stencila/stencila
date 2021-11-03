use super::Binary;

/// A list of all managed binaries
pub(crate) fn all() -> Vec<Binary> {
    vec![
        chrome(),
        node(),
        pandoc(),
        python(),
    ]
}

/// Binary definition for Chrome / Chromium
///
/// Version history at https://en.wikipedia.org/wiki/Google_Chrome_version_history
/// but only use triples ending in `.0` here and make sure there is a mapping in the
/// `install_chromium` function.
fn chrome() -> Binary {
    Binary::new("chrome", &["chromium"], &["91.0.0"])
}

/// Binary definition for Node.js
///
/// Release list at https://nodejs.org/en/download/releases/
#[rustfmt::skip]
fn node() -> Binary {
    Binary::new(
        "node",
        &[],
        &[
            "16.4.0", "16.4.1", "16.4.2",
            "16.5.0",
            "16.6.0", "16.6.1", "16.6.2",
            "16.7.0",
            "16.8.0",
            "16.9.0", "16.9.1",
            "16.10.0",
            "16.11.0", "16.11.1",
            "16.12.0",
            "16.13.0",
            "17.0.0", "17.0.1",
        ],
    )
}

/// Binary definition for Pandoc
///
/// Release list at https://github.com/jgm/pandoc/releases
/// To avoid version parsing issues we map standard semver triples
/// to Pandoc's quads in the `install_pandoc` function and use only triples here.
fn pandoc() -> Binary {
    Binary::new(
        "pandoc",
        &[],
        &["2.14.0", "2.14.1", "2.14.2", "2.15.0", "2.16.0"],
    )
}

/// Binary definition for Python
///
/// Release list at https://www.python.org/downloads/
fn python() -> Binary {
    Binary::new("python", &["python3"], &["3.9.6", "3.9.7", "3.10.0"])
}
