// An internal crate for managing fonts
//
// Currently, main use is to parse CSS font stacks from themes to provide font
// information to Python and R kernels for plotting.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};

use eyre::{Context, Result};
use fontdb::{Database, Family, Query};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use regex::Regex;
use reqwest::Client;
use stencila_dirs::{DirType, get_app_dir};

/// Information about a resolved font
pub struct Font {
    family: String,
    path: PathBuf,
}

impl Font {
    /// Get the font family name (e.g., "Source Serif 4")
    pub fn family(&self) -> &str {
        &self.family
    }

    /// Get the path to the font file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the font file as bytes (useful for embedding in documents)
    pub fn bytes(&self) -> Result<Vec<u8>> {
        fs::read(&self.path).context("Failed to read font file")
    }

    /// Resolve the first available font from a CSS font stack
    ///
    /// Searches for fonts in this order:
    ///
    /// 1. System-installed fonts
    /// 2. Previously cached fonts in the Stencila fonts directory
    /// 3. Downloads from Google Fonts (using heuristic URLs)
    ///
    /// Returns `Ok(None)` if no concrete font could be found.
    /// Returns `Err` only for system failures (I/O errors, etc.).
    pub async fn resolve_first(stack: &str) -> Result<Option<Self>> {
        let candidates = parse_css_stack(stack);

        // Build the system font database once
        let mut db = Database::new();
        db.load_system_fonts();

        for family in candidates.iter() {
            if is_css_generic(family) {
                // Skip CSS generic keywords
                continue;
            }

            // Check system fonts
            if let Some(path) = find_system_font(&db, family) {
                return Ok(Some(Font {
                    family: family.clone(),
                    path,
                }));
            }

            // Check cache
            if let Some(path) = find_cached_ttf(family)? {
                return Ok(Some(Font {
                    family: family.clone(),
                    path,
                }));
            }

            // Try downloading from Google Fonts
            if let Some(path) = try_download_google_font(family).await? {
                return Ok(Some(Font {
                    family: family.clone(),
                    path,
                }));
            }
        }

        // No concrete font found
        Ok(None)
    }
}

/// CSS generic keywords that are not concrete font families
fn is_css_generic(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "serif"
            | "sans-serif"
            | "monospace"
            | "cursive"
            | "fantasy"
            | "system-ui"
            | "ui-serif"
            | "ui-sans-serif"
            | "ui-monospace"
            | "emoji"
            | "math"
            | "fangsong"
    )
}

/// Parse a CSS font-family stack and strip quotes
///
/// Example: "'Iowan Old Style', Palatino, serif" -> ["Iowan Old Style", "Palatino", "serif"]
fn parse_css_stack(stack: &str) -> Vec<String> {
    stack
        .split(',')
        .map(|s| s.trim())
        .map(|s| s.trim_matches('"').trim_matches('\''))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Create a slug for cache file names
///
/// Example: "Source Serif 4" -> "source-serif-4"
fn slugify_family(family: &str) -> String {
    let re = Regex::new(r"[^a-z0-9]+").unwrap_or_else(|_| unreachable!());
    let s = family.to_ascii_lowercase();
    let s = re.replace_all(&s, "-");
    s.trim_matches('-').to_string()
}

/// Check if a font family is installed in the system and get its path
fn find_system_font(db: &Database, family: &str) -> Option<PathBuf> {
    use fontdb::Source;

    if is_css_generic(family) {
        return None;
    }

    let face_id = db.query(&Query {
        families: &[Family::Name(family)],
        ..Default::default()
    })?;

    db.face(face_id).and_then(|face| match &face.source {
        Source::File(path) => Some(path.to_path_buf()),
        Source::Binary(_) | Source::SharedFile(..) => None,
    })
}

/// Try to find an existing cached TTF file for a font family
fn find_cached_ttf(family: &str) -> Result<Option<PathBuf>> {
    let fonts_dir = get_app_dir(DirType::Fonts, true)?;
    let path = fonts_dir.join(format!("{}.ttf", slugify_family(family)));

    if path.exists() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

/// Build candidate URLs for Google Fonts families
///
/// Tries variable fonts first, then static regular variants.
/// Uses jsDelivr CDN first (faster), then raw.githubusercontent as fallback.
fn candidate_google_fonts_urls(family: &str) -> Vec<(String, String)> {
    // Google Fonts directory is usually under ofl/, lowercased with spaces removed
    let dir_name = family.to_ascii_lowercase().replace(' ', "");
    let base_jsd = format!(
        "https://cdn.jsdelivr.net/gh/google/fonts@main/ofl/{dir}/",
        dir = dir_name
    );
    let base_raw = format!(
        "https://raw.githubusercontent.com/google/fonts/refs/heads/main/ofl/{dir}/",
        dir = dir_name
    );

    // Try common font file naming patterns
    let candidates = [
        format!("{}[wght].ttf", family.replace(' ', "")),
        format!("{}[opsz,wght].ttf", family.replace(' ', "")),
        format!("{}-Regular.ttf", family.replace(' ', "")),
        format!("{}Regular.ttf", family.replace(' ', "")),
    ];

    let mut urls = Vec::new();
    for file in candidates {
        let encoded = utf8_percent_encode(&file, NON_ALPHANUMERIC).to_string();
        urls.push(("jsdelivr".to_string(), format!("{}{}", base_jsd, encoded)));
        urls.push(("raw".to_string(), format!("{}{}", base_raw, encoded)));
    }
    urls
}

/// Try to download a font from Google Fonts
async fn try_download_google_font(family: &str) -> Result<Option<PathBuf>> {
    let urls = candidate_google_fonts_urls(family);
    if urls.is_empty() {
        return Ok(None);
    }

    let fonts_dir = get_app_dir(DirType::Fonts, true)?;
    let path = fonts_dir.join(format!("{}.ttf", slugify_family(family)));

    let client = Client::builder().timeout(Duration::from_secs(20)).build()?;

    for (backend, url) in urls {
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let bytes = match resp.bytes().await {
                    Ok(b) if !b.is_empty() => b,
                    _ => continue,
                };

                let mut file = fs::File::create(&path)?;
                file.write_all(&bytes)?;

                tracing::debug!(
                    "Downloaded font {} from {} -> {}",
                    family,
                    backend,
                    path.display()
                );
                return Ok(Some(path));
            }
            _ => continue,
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_css_generic() {
        assert!(is_css_generic("serif"));
        assert!(is_css_generic("sans-serif"));
        assert!(is_css_generic("monospace"));
        assert!(is_css_generic("ui-serif"));

        assert!(!is_css_generic("Arial"));
        assert!(!is_css_generic("Source Serif 4"));
    }

    #[test]
    fn test_parse_css_stack() {
        let stack = "'Source Serif 4', \"Iowan Old Style\", Palatino, serif";
        let parsed = parse_css_stack(stack);
        assert_eq!(
            parsed,
            vec!["Source Serif 4", "Iowan Old Style", "Palatino", "serif"]
        );
    }

    #[test]
    fn test_slugify_family() {
        assert_eq!(slugify_family("Source Serif 4"), "source-serif-4");
        assert_eq!(slugify_family("IBM Plex Mono"), "ibm-plex-mono");
        assert_eq!(slugify_family("Times New Roman"), "times-new-roman");
    }
}
