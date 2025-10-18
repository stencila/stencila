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

/// The source from which a font was resolved
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSource {
    /// Font found in system fonts
    System,
    /// Font found in Stencila's cache directory
    Cached,
    /// Font downloaded from Google Fonts
    Downloaded,
}

/// Information about a resolved font
pub struct Font {
    family: String,
    path: PathBuf,
    source: FontSource,
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

    /// Get the source from which this font was resolved
    pub fn source(&self) -> FontSource {
        self.source
    }

    /// Read the font file as bytes (useful for embedding in documents)
    pub fn bytes(&self) -> Result<Vec<u8>> {
        fs::read(&self.path).context("Failed to read font file")
    }

    /// Check if this font is safe to embed in documents
    ///
    /// Returns `true` if:
    /// - The font came from Google Fonts (cached or downloaded), OR
    /// - The font is a system font that's known to be open source
    ///
    /// Returns `false` for system fonts that may be proprietary.
    pub fn is_safe_to_embed(&self) -> bool {
        match self.source {
            // Google Fonts downloads are always safe
            FontSource::Cached | FontSource::Downloaded => true,
            // System fonts: only safe if they're known open source fonts
            FontSource::System => is_open_source_font(&self.family),
        }
    }

    /// Extract the first non-generic font name from a CSS font stack
    ///
    /// This is useful as a fallback when `resolve_first` returns `None`.
    /// It parses the CSS stack and returns the first concrete font name
    /// (skipping CSS generic keywords like "serif", "sans-serif", etc.).
    ///
    /// The returned font name will have quotes removed and be clean for
    /// use in XML attributes or other contexts.
    ///
    /// Returns `None` if the stack contains only generic keywords.
    ///
    /// # Examples
    ///
    /// ```
    /// use stencila_fonts::Font;
    ///
    /// let stack = "'Iowan Old Style', Palatino, serif";
    /// assert_eq!(Font::extract_first(stack), Some("Iowan Old Style".to_string()));
    ///
    /// let generic_only = "serif, sans-serif";
    /// assert_eq!(Font::extract_first(generic_only), None);
    /// ```
    pub fn extract_first(stack: &str) -> Option<String> {
        parse_css_stack(stack)
            .into_iter()
            .find(|name| !is_css_generic(name))
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
                    source: FontSource::System,
                }));
            }

            // Check cache
            if let Some(path) = find_cached_ttf(family)? {
                return Ok(Some(Font {
                    family: family.clone(),
                    path,
                    source: FontSource::Cached,
                }));
            }

            if is_proprietary_font(family) {
                // Skip known proprietary fonts to avoid wasted download attempts
                tracing::trace!("Skipping proprietary font: {family}");
                continue;
            }

            // Try downloading from Google Fonts
            if let Some(path) = try_download_google_font(family).await? {
                return Ok(Some(Font {
                    family: family.clone(),
                    path,
                    source: FontSource::Downloaded,
                }));
            }
        }

        // No concrete font found
        Ok(None)
    }
}

/// Normalize a font family name for consistent matching
///
/// This function performs the following normalization steps:
/// 1. Trims leading and trailing whitespace
/// 2. Removes surrounding single or double quotes
/// 3. Collapses multiple consecutive spaces to a single space
/// 4. Converts to lowercase (using to_lowercase() for Unicode support)
///
/// This ensures that font names like " 'Arial' ", "Arial", and "ARIAL"
/// are all normalized to "arial" for consistent matching.
fn normalize_font_name(name: &str) -> String {
    // Trim whitespace
    let mut normalized = name.trim();

    // Remove surrounding quotes (single or double)
    if (normalized.starts_with('\'') && normalized.ends_with('\''))
        || (normalized.starts_with('"') && normalized.ends_with('"'))
    {
        normalized = &normalized[1..normalized.len() - 1];
        normalized = normalized.trim(); // Trim again after removing quotes
    }

    // Collapse multiple spaces to single space using a simple approach
    let mut result = String::with_capacity(normalized.len());
    let mut prev_was_space = false;

    for ch in normalized.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }

    // Convert to lowercase (supports Unicode better than to_ascii_lowercase)
    result.to_lowercase()
}

/// Check if a font family name is a CSS generic keyword
///
/// CSS generic font families are fallback keywords that should not be resolved
/// to concrete font files. This includes:
/// - Standard CSS generics: serif, sans-serif, monospace, cursive, fantasy
/// - System UI families: system-ui, ui-serif, ui-sans-serif, ui-monospace
/// - Special purpose: emoji, math, fangsong
/// - Shorthand forms: sans, mono (commonly used by R and other systems)
///
/// Returns `true` if the name is a generic keyword, `false` otherwise.
fn is_css_generic(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "cursive"
            | "emoji"
            | "fangsong"
            | "fantasy"
            | "math"
            | "mono"
            | "monospace"
            | "sans"
            | "sans-serif"
            | "serif"
            | "system-ui"
            | "ui-monospace"
            | "ui-sans-serif"
            | "ui-serif"
    )
}

/// Check if a font family name is a known proprietary font
///
/// This function identifies common proprietary fonts from Windows, macOS, Adobe, and
/// other vendors. These fonts should not be downloaded from Google Fonts (they won't
/// be there) and should not be embedded in documents (license violation risk).
///
/// This is a blocklist approach focusing on the most commonly specified proprietary
/// fonts to improve performance and legal safety.
///
/// Returns `true` if the font is known to be proprietary, `false` otherwise.
fn is_proprietary_font(name: &str) -> bool {
    // CSS generic families are neither proprietary nor concrete fonts
    if is_css_generic(name) {
        return false;
    }

    let name_normalized = normalize_font_name(name);

    // Check for exact matches first
    if matches!(
        name_normalized.as_str(),
        // Microsoft fonts (Windows)
        "arial" | "arial black" | "arial narrow" | "arial rounded mt bold"
            | "calibri" | "calibri light" | "cambria" | "cambria math" | "candara"
            | "comic sans ms" | "consolas" | "constantia" | "corbel" | "courier new"
            | "ebrima" | "franklin gothic medium" | "gabriola" | "gadugi" | "georgia"
            | "impact" | "ink free" | "javanese text" | "leelawadee ui" | "lucida console"
            | "lucida sans unicode" | "malgun gothic" | "marlett" | "microsoft himalaya"
            | "microsoft jhenghei" | "microsoft new tai lue" | "microsoft phagspa"
            | "microsoft sans serif" | "microsoft tai le" | "microsoft yahei"
            | "microsoft yi baiti" | "mingliu-extb" | "mingliu_hkscs-extb" | "mongolian baiti"
            | "ms gothic" | "ms pgothic" | "ms ui gothic" | "mv boli" | "myanmar text"
            | "nirmala ui" | "palatino linotype" | "segoe mdl2 assets" | "segoe print"
            | "segoe script" | "segoe ui" | "segoe ui emoji" | "segoe ui historic"
            | "segoe ui symbol" | "simsun" | "simsun-extb" | "sitka" | "sylfaen"
            | "symbol" | "tahoma" | "times new roman" | "trebuchet ms" | "verdana"
            | "webdings" | "wingdings" | "yu gothic"
            // Apple fonts (macOS/iOS)
            | "american typewriter" | "andale mono" | "apple braille" | "apple color emoji"
            | "apple sd gothic neo" | "apple symbols" | "arial hebrew" | "avenir"
            | "avenir next" | "avenir next condensed" | "bangkok" | "baskerville"
            | "big caslon" | "bodoni 72" | "bodoni 72 oldstyle" | "bodoni 72 smallcaps"
            | "bradley hand" | "brush script mt" | "chalkboard" | "chalkboard se"
            | "chalkduster" | "charter" | "cochin" | "copperplate" | "courier"
            | "didot" | "euphemia ucas" | "futura" | "geneva" | "geeza pro" | "gill sans"
            | "helvetica" | "helvetica neue" | "herculanum" | "hoefler text"
            | "iowan old style" | "kefa" | "khmer sangam mn" | "kohinoor bangla"
            | "kohinoor devanagari" | "kohinoor gujarati" | "kohinoor telugu" | "lao sangam mn"
            | "lucida grande" | "luminari" | "marker felt" | "menlo" | "mishafi"
            | "monaco" | "mshtakan" | "mukta mahee" | "muna" | "myanmar sangam mn"
            | "nadeem" | "new peninim mt" | "noteworthy" | "optima" | "palatino"
            | "papyrus" | "phosphate" | "plantagenet cherokee" | "raanana" | "rockwell"
            | "sana" | "sathu" | "savoye let"
            | "shree devanagari 714" | "signpainter" | "silom" | "sinhala sangam mn"
            | "skia" | "snell roundhand" | "songti sc" | "songti tc" | "sukhumvit set"
            | "superclarendon" | "thonburi" | "times" | "trattatello" | "zapfino"
            // Adobe fonts
            | "adobe arabic" | "adobe caslon" | "adobe caslon pro" | "adobe garamond"
            | "adobe garamond pro" | "adobe hebrew" | "adobe jenson" | "adobe ming std"
            | "adobe myungjo std" | "adobe song std" | "myriad pro" | "minion pro"
            // Other common proprietary fonts
            | "bitstream charter" | "book antiqua" | "bookman old style" | "century"
            | "century gothic" | "century schoolbook" | "garamond" | "goudy old style"
            | "lucida" | "monotype corsiva" | "ms sans serif" | "ms serif"
    ) {
        return true;
    }

    // Check for partial matches (font family names often include variants)
    // These checks are more permissive to catch variants like "Arial Bold", "Times New Roman Italic", etc.
    if name_normalized.starts_with("arial")
        || name_normalized.starts_with("calibri")
        || name_normalized.starts_with("cambria")
        || name_normalized.starts_with("segoe")
        || name_normalized.starts_with("helvetica")
        || name_normalized.starts_with("avenir")
        || name_normalized.starts_with("gill sans")
        || name_normalized.starts_with("yu gothic")
        || name_normalized.starts_with("times new roman")
        || name_normalized.starts_with("bodoni")
    {
        return true;
    }

    false
}

/// Check if a font family name is a known open source font
///
/// This function identifies fonts with permissive open source licenses (OFL, Apache,
/// MIT, GPL with font exception, etc.) that are safe to embed in documents.
///
/// This is an allowlist approach focusing on popular open source fonts that users
/// commonly install system-wide but are still safe to embed.
///
/// Returns `true` if the font is known to be open source, `false` otherwise.
fn is_open_source_font(name: &str) -> bool {
    // CSS generic families are neither open source nor concrete fonts
    if is_css_generic(name) {
        return false;
    }

    let name_normalized = normalize_font_name(name);

    // Check for exact matches first
    if matches!(
        name_normalized.as_str(),
        // Google Fonts - Open Font License (OFL)
        "inter" | "inter variable"
            | "roboto" | "roboto mono" | "roboto slab" | "roboto condensed"
            | "noto sans" | "noto serif" | "noto sans mono" | "noto color emoji"
            | "source sans pro" | "source sans 3" | "source serif pro" | "source serif 4"
            | "source code pro"
            | "ibm plex sans" | "ibm plex serif" | "ibm plex mono"
            | "open sans" | "open sans condensed"
            | "lato" | "montserrat" | "oswald" | "raleway"
            | "pt sans" | "pt serif" | "pt mono"
            | "merriweather" | "merriweather sans"
            | "work sans" | "fira sans" | "fira code" | "fira mono"
            | "jetbrains mono" | "ubuntu" | "ubuntu mono" | "ubuntu condensed"
            | "inconsolata" | "droid sans" | "droid serif" | "droid sans mono"
            | "nunito" | "nunito sans" | "playfair display" | "poppins"
            | "crimson text" | "crimson pro" | "eb garamond"
            | "libre baskerville" | "libre franklin" | "libre caslon text"
            | "overpass" | "overpass mono"
            | "dm sans" | "dm serif display" | "dm mono"
            | "manrope" | "recursive" | "space grotesk" | "space mono"
            | "atkinson hyperlegible"
        // Liberation fonts - GPL with font exception
        | "liberation sans" | "liberation serif" | "liberation mono"
        // Bitstream Vera fonts - Free license (BSD-style)
        | "bitstream vera sans" | "bitstream vera serif" | "bitstream vera sans mono"
        // DejaVu fonts - Free license (Bitstream Vera derivative)
        | "dejavu sans" | "dejavu serif" | "dejavu sans mono"
        // GNU FreeFont - GPL with font exception
        | "freesans" | "freeserif" | "freemono"
        // Red Hat fonts - OFL
        | "red hat display" | "red hat text" | "red hat mono"
        // Microsoft open source fonts - OFL
        | "cascadia code" | "cascadia mono"
        // Adobe open source - OFL
        | "source han sans" | "source han serif"
        // Other popular open source
        | "public sans" | "commissioner" | "jost"
        | "bitter" | "exo" | "exo 2"
        | "archivo" | "archivo narrow"
    ) {
        return true;
    }

    // Check for partial matches (font families)
    // These checks catch variants like "Noto Sans Display", "Source Sans Pro Italic", etc.
    if name_normalized.starts_with("noto ")
        || name_normalized.starts_with("source ")
        || name_normalized.starts_with("ibm plex ")
        || name_normalized.starts_with("bitstream vera ")
        || name_normalized.starts_with("dejavu ")
        || name_normalized.starts_with("liberation ")
        || name_normalized.starts_with("red hat ")
        || name_normalized.starts_with("cascadia ")
        || name_normalized.starts_with("libre ")
        || name_normalized.starts_with("pt ")
        || name_normalized.starts_with("roboto ")
        || name_normalized.starts_with("fira ")
        || name_normalized.starts_with("ubuntu ")
    {
        return true;
    }

    false
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
    fn test_normalize_font_name() {
        // Basic normalization
        assert_eq!(normalize_font_name("Arial"), "arial");
        assert_eq!(normalize_font_name("Times New Roman"), "times new roman");

        // Whitespace trimming
        assert_eq!(normalize_font_name("  Arial  "), "arial");
        assert_eq!(
            normalize_font_name("  Times New Roman  "),
            "times new roman"
        );

        // Quote removal
        assert_eq!(normalize_font_name("'Arial'"), "arial");
        assert_eq!(normalize_font_name("\"Arial\""), "arial");
        assert_eq!(normalize_font_name("'Times New Roman'"), "times new roman");
        assert_eq!(
            normalize_font_name("\"Times New Roman\""),
            "times new roman"
        );

        // Quote removal with whitespace
        assert_eq!(normalize_font_name("  'Arial'  "), "arial");
        assert_eq!(normalize_font_name("  \"Roboto\"  "), "roboto");

        // Multiple space collapsing
        assert_eq!(normalize_font_name("Times  New  Roman"), "times new roman");
        assert_eq!(normalize_font_name("Noto  Sans   Mono"), "noto sans mono");

        // Case conversion
        assert_eq!(normalize_font_name("ARIAL"), "arial");
        assert_eq!(normalize_font_name("HeLvEtIcA"), "helvetica");
        assert_eq!(normalize_font_name("Times NEW Roman"), "times new roman");

        // Combined normalization
        assert_eq!(
            normalize_font_name("  'Times  New  Roman'  "),
            "times new roman"
        );
        assert_eq!(
            normalize_font_name("  \"INTER  Variable\"  "),
            "inter variable"
        );
    }

    #[test]
    fn test_is_css_generic() {
        // Standard CSS generics
        assert!(is_css_generic("serif"));
        assert!(is_css_generic("sans-serif"));
        assert!(is_css_generic("monospace"));
        assert!(is_css_generic("cursive"));
        assert!(is_css_generic("fantasy"));

        // Shorthand forms (used by R and other systems)
        assert!(is_css_generic("sans"));
        assert!(is_css_generic("mono"));

        // System UI families
        assert!(is_css_generic("system-ui"));
        assert!(is_css_generic("ui-serif"));
        assert!(is_css_generic("ui-sans-serif"));
        assert!(is_css_generic("ui-monospace"));

        // Special purpose
        assert!(is_css_generic("emoji"));
        assert!(is_css_generic("math"));
        assert!(is_css_generic("fangsong"));

        // Case insensitivity
        assert!(is_css_generic("SERIF"));
        assert!(is_css_generic("Sans-Serif"));

        // Concrete font families should return false
        assert!(!is_css_generic("Arial"));
        assert!(!is_css_generic("Source Serif 4"));
        assert!(!is_css_generic("Noto Sans Mono"));
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

    #[test]
    fn test_is_proprietary_font() {
        // Microsoft fonts
        assert!(is_proprietary_font("Arial"));
        assert!(is_proprietary_font("Times New Roman"));
        assert!(is_proprietary_font("Georgia"));
        assert!(is_proprietary_font("Calibri"));
        assert!(is_proprietary_font("Cambria"));
        assert!(is_proprietary_font("Consolas"));
        assert!(is_proprietary_font("Segoe UI"));
        assert!(is_proprietary_font("Verdana"));
        assert!(is_proprietary_font("Tahoma"));

        // Apple fonts
        assert!(is_proprietary_font("Helvetica"));
        assert!(is_proprietary_font("Helvetica Neue"));
        assert!(is_proprietary_font("Iowan Old Style"));
        assert!(is_proprietary_font("Palatino"));
        assert!(is_proprietary_font("Avenir"));
        assert!(is_proprietary_font("Gill Sans"));
        assert!(is_proprietary_font("Menlo"));
        assert!(is_proprietary_font("Monaco"));

        // Adobe fonts
        assert!(is_proprietary_font("Myriad Pro"));
        assert!(is_proprietary_font("Minion Pro"));

        // Other proprietary
        assert!(is_proprietary_font("Bitstream Charter"));
        assert!(is_proprietary_font("Courier New"));

        // Case insensitivity
        assert!(is_proprietary_font("ARIAL"));
        assert!(is_proprietary_font("Times New Roman"));

        // Normalization: quotes
        assert!(is_proprietary_font("'Arial'"));
        assert!(is_proprietary_font("\"Helvetica\""));
        assert!(is_proprietary_font("'Times New Roman'"));

        // Normalization: whitespace
        assert!(is_proprietary_font("  Arial  "));
        assert!(is_proprietary_font("  Calibri  "));

        // Normalization: combined
        assert!(is_proprietary_font("  'Georgia'  "));
        assert!(is_proprietary_font("  \"Segoe UI\"  "));

        // Partial matches: variants
        assert!(is_proprietary_font("Yu Gothic UI"));
        assert!(is_proprietary_font("Times New Roman PS"));
        assert!(is_proprietary_font("Bodoni 72 Oldstyle"));
        assert!(is_proprietary_font("Arial Bold"));
        assert!(is_proprietary_font("Helvetica Now"));

        // CSS generics should return false
        assert!(!is_proprietary_font("serif"));
        assert!(!is_proprietary_font("sans-serif"));
        assert!(!is_proprietary_font("monospace"));

        // Open source fonts should return false
        assert!(!is_proprietary_font("Noto Sans"));
        assert!(!is_proprietary_font("Roboto"));
        assert!(!is_proprietary_font("Source Sans Pro"));
        assert!(!is_proprietary_font("IBM Plex Mono"));
        assert!(!is_proprietary_font("Inter"));
        assert!(!is_proprietary_font("Liberation Sans"));
    }

    #[test]
    fn test_is_open_source_font() {
        // Google Fonts (OFL)
        assert!(is_open_source_font("Inter"));
        assert!(is_open_source_font("Inter Variable"));
        assert!(is_open_source_font("Roboto"));
        assert!(is_open_source_font("Roboto Mono"));
        assert!(is_open_source_font("Noto Sans"));
        assert!(is_open_source_font("Noto Serif"));
        assert!(is_open_source_font("Source Sans Pro"));
        assert!(is_open_source_font("Source Serif 4"));
        assert!(is_open_source_font("Source Code Pro"));
        assert!(is_open_source_font("IBM Plex Sans"));
        assert!(is_open_source_font("IBM Plex Mono"));
        assert!(is_open_source_font("Open Sans"));
        assert!(is_open_source_font("Lato"));
        assert!(is_open_source_font("Montserrat"));
        assert!(is_open_source_font("JetBrains Mono"));
        assert!(is_open_source_font("Fira Code"));

        // Liberation fonts
        assert!(is_open_source_font("Liberation Sans"));
        assert!(is_open_source_font("Liberation Serif"));
        assert!(is_open_source_font("Liberation Mono"));

        // DejaVu fonts
        assert!(is_open_source_font("DejaVu Sans"));
        assert!(is_open_source_font("DejaVu Serif"));
        assert!(is_open_source_font("DejaVu Sans Mono"));

        // Red Hat fonts
        assert!(is_open_source_font("Red Hat Display"));
        assert!(is_open_source_font("Red Hat Text"));
        assert!(is_open_source_font("Red Hat Mono"));

        // Microsoft open source
        assert!(is_open_source_font("Cascadia Code"));
        assert!(is_open_source_font("Cascadia Mono"));

        // Family prefixes (catches variants)
        assert!(is_open_source_font("Noto Sans Display"));
        assert!(is_open_source_font("Source Sans Variable"));
        assert!(is_open_source_font("IBM Plex Sans Condensed"));

        // Case insensitivity
        assert!(is_open_source_font("INTER"));
        assert!(is_open_source_font("roboto mono"));

        // Normalization: quotes
        assert!(is_open_source_font("'Inter'"));
        assert!(is_open_source_font("\"Roboto\""));
        assert!(is_open_source_font("'Source Sans Pro'"));

        // Normalization: whitespace
        assert!(is_open_source_font("  Inter  "));
        assert!(is_open_source_font("  Noto Sans  "));

        // Normalization: combined
        assert!(is_open_source_font("  'Liberation Sans'  "));
        assert!(is_open_source_font("  \"IBM Plex Mono\"  "));

        // Partial matches: new variants
        assert!(is_open_source_font("Libre Caslon Display"));
        assert!(is_open_source_font("PT Sans Caption"));
        assert!(is_open_source_font("Roboto Flex"));
        assert!(is_open_source_font("Fira Sans Extra Condensed"));
        assert!(is_open_source_font("Ubuntu Condensed"));

        // CSS generics should return false
        assert!(!is_open_source_font("serif"));
        assert!(!is_open_source_font("sans-serif"));
        assert!(!is_open_source_font("monospace"));

        // Proprietary fonts should return false
        assert!(!is_open_source_font("Arial"));
        assert!(!is_open_source_font("Helvetica"));
        assert!(!is_open_source_font("Times New Roman"));
        assert!(!is_open_source_font("Georgia"));
        assert!(!is_open_source_font("Calibri"));
    }
}
