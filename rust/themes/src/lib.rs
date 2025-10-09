use std::{
    collections::BTreeMap,
    env::current_dir,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use eyre::{Result, bail, eyre};
use lightningcss::{
    printer::Printer,
    properties::{Property, custom::CustomPropertyName},
    rules::CssRule,
    stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet},
    targets::{Features, Targets},
    traits::ToCss,
};
use notify::{Event, RecursiveMode, Watcher, event::EventKind};
use pathdiff::diff_paths;
use regex::Regex;
use serde_json::{Value, json};
use tokio::{
    fs::{read_to_string, remove_file, write},
    sync::mpsc,
};

use stencila_dirs::{DirType, get_app_dir};
use stencila_web_dist::Web;

pub mod cli;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeType {
    Workspace,
    User,
    Builtin,
}

impl ThemeType {
    /// Get a display string for the theme type
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeType::Workspace => "workspace",
            ThemeType::User => "user",
            ThemeType::Builtin => "builtin",
        }
    }
}

pub struct Theme {
    /// The type of theme
    pub r#type: ThemeType,

    /// The name of the theme
    ///
    /// If a workspace theme (i.e. a `theme.css` file) then may not
    /// have a name. Otherwise should have a theme.
    pub name: Option<String>,

    /// The location of the theme
    ///
    /// The relative path, from the current working directory, to the theme.
    /// For builtin themes, will be `None`.
    pub location: Option<String>,

    /// The CSS content of the theme
    pub content: String,

    /// CSS variables defined in the theme
    ///
    /// Includes variables from base.css merged with theme-specific variables.
    /// Variable names have the `--` prefix stripped.
    pub variables: BTreeMap<String, String>,
}

impl Theme {
    /// Create a new theme from CSS content
    ///
    /// # Arguments
    /// * `r#type` - The type of theme
    /// * `name` - Optional name for the theme
    /// * `location` - Optional file path location
    /// * `css` - Raw CSS content
    /// * `normalize` - Whether to normalize and minify the CSS
    pub fn new(
        r#type: ThemeType,
        name: Option<String>,
        location: Option<String>,
        css: String,
        normalize: bool,
    ) -> Self {
        let content = if normalize {
            Self::normalize_css(&css)
        } else {
            css
        };
        let variables = Self::merge_css_variables(&content);

        Self {
            r#type,
            name,
            location,
            content,
            variables,
        }
    }

    /// Normalize and minify CSS
    ///
    /// Parses CSS and outputs a minified version. Returns the original CSS
    /// if parsing or printing fails.
    pub fn normalize_css(css: &str) -> String {
        StyleSheet::parse(css, ParserOptions::default())
            .map(|stylesheet| {
                stylesheet
                    .to_css(PrinterOptions {
                        minify: true,
                        ..Default::default()
                    })
                    .map(|result| result.code)
                    .unwrap_or_else(|_| css.to_string())
            })
            .unwrap_or_else(|_| css.to_string())
    }

    /// Get a CSS variable value by name
    ///
    /// The name should not include the `--` prefix.
    pub fn variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    /// Compute variables with resolved references and typed JSON values
    ///
    /// Resolves var() references, evaluates calc() and color-mix() using lightningcss,
    /// normalizes colors to hex and lengths to points, and returns typed JSON values
    /// suitable for kernels (Python matplotlib, R ggplot2, etc.)
    pub fn compute_variables(&self) -> BTreeMap<String, Value> {
        let mut computed = BTreeMap::new();

        for (name, value) in &self.variables {
            let resolved = self.resolve_var_references(value, 0);
            let evaluated = Self::evaluate_css_value(&resolved);

            let normalized = if Self::should_normalize_length_to_points(name) {
                if let Some(points) = Self::normalize_length_to_points(&evaluated) {
                    computed.insert(name.clone(), json!(points));
                    continue;
                }
                evaluated
            } else {
                evaluated
            };

            let typed = Self::to_json_value(&normalized);
            computed.insert(name.clone(), typed);
        }

        computed
    }

    /// Recursively resolve var() references in a CSS value
    ///
    /// Depth limit prevents infinite recursion from circular references
    fn resolve_var_references(&self, value: &str, depth: u8) -> String {
        const MAX_DEPTH: u8 = 10;

        if depth >= MAX_DEPTH {
            return value.to_string();
        }

        // Regex pattern: var(--name) or var(--name, fallback)
        static REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"var\(\s*--([\w-]+)\s*(?:,\s*([^)]+))?\s*\)").expect("var() regex is valid")
        });

        REGEX
            .replace_all(value, |caps: &regex::Captures| {
                let var_name = &caps[1];
                if let Some(var_value) = self.variables.get(var_name) {
                    // Recurse immediately to resolve nested references
                    self.resolve_var_references(var_value, depth + 1)
                } else if let Some(fallback) = caps.get(2) {
                    fallback.as_str().to_string()
                } else {
                    caps[0].to_string() // keep original if not found
                }
            })
            .to_string()
    }

    /// Evaluate calc() and color-mix() using lightningcss minification
    ///
    /// For color values (detected by variable name or color functions in value),
    /// uses the `color:` property with browser targets to ensure transpilation to hex.
    /// For other values, uses the `width:` property for calc() evaluation.
    fn evaluate_css_value(value: &str) -> String {
        let property = if value.contains("color-mix(")
            || value.contains("oklch(")
            || value.contains("oklab(")
            || value.contains("lch(")
            || value.contains("lab(")
            || value.contains("hsl(")
            || value.contains("hsla(")
            || value.contains("rgb(")
            || value.contains("rgba(")
            || value.starts_with('#')
        {
            "color"
        } else {
            "width"
        };

        let prefix = [".temp{", property, ":"].concat();
        let css = [&prefix, value, "}"].concat();
        let Ok(mut sheet) = StyleSheet::parse(&css, ParserOptions::default()) else {
            return value.to_string();
        };

        // Minify (i.e. compile) all features
        let targets = Targets {
            // Features that should always be compiled
            include: Features::all(),
            ..Default::default()
        };
        sheet
            .minify(MinifyOptions {
                targets,
                ..Default::default()
            })
            .ok();

        // Print to minified CSS
        let Ok(result) = sheet.to_css(PrinterOptions {
            minify: true,
            ..Default::default()
        }) else {
            return value.to_string();
        };

        // Extract value
        let value = result.code.trim_start_matches(&prefix);
        if let Some(end) = value.find(";").or_else(|| value.find("}")) {
            Self::hexify_color_name(&value[..end]).to_string()
        } else {
            result.code.to_string()
        }
    }

    /// Convert a color name to its hex value
    ///
    /// LightningCss has a `short_color_name` which is always calls as part of
    /// its minification (ie it can not be turned off). This reverses that so
    /// that we always get hex values for colors.
    fn hexify_color_name(value: &str) -> &str {
        match value {
            "navy" => "#000080",
            "green" => "#008000",
            "teal" => "#008080",
            "indigo" => "#4b0082",
            "maroon" => "#800000",
            "purple" => "#800080",
            "olive" => "#808000",
            "gray" => "#808080",
            "sienna" => "#a0522d",
            "brown" => "#a52a2a",
            "silver" => "#c0c0c0",
            "peru" => "#cd853f",
            "tan" => "#d2b48c",
            "orchid" => "#da70d6",
            "plum" => "#dda0dd",
            "violet" => "#ee82ee",
            "khaki" => "#f0e68c",
            "azure" => "#f0ffff",
            "wheat" => "#f5deb3",
            "beige" => "#f5f5dc",
            "salmon" => "#fa8072",
            "linen" => "#faf0e6",
            "red" => "#ff0000",
            "tomato" => "#ff6347",
            "coral" => "#ff7f50",
            "orange" => "#ffa500",
            "pink" => "#ffc0cb",
            "gold" => "#ffd700",
            "bisque" => "#ffe4c4",
            "snow" => "#fffafa",
            "ivory" => "#fffff0",
            _ => value,
        }
    }

    /// Check if a variable should be normalized as a length (to points) based on its name
    ///
    /// Only normalizes plot-* variables to avoid affecting other theme variables
    fn should_normalize_length_to_points(name: &str) -> bool {
        name.contains("plot")
            && (name.contains("size")
                || name.contains("width")
                || name.contains("height")
                || name.contains("spacing")
                || name.contains("padding")
                || name.contains("margin")
                || name.contains("gap")
                || name.contains("radius")
                || name.contains("tick"))
    }

    /// Normalize a CSS length value to points (pt)
    ///
    /// Conversions:
    /// - rem → pt: value × 16 × 0.75 (assuming 16px root font size, 0.75 for px→pt)
    /// - px → pt: value × 0.75
    /// - pt → pt: unchanged
    /// - Unitless values are returned as-is
    fn normalize_length_to_points(value: &str) -> Option<f64> {
        let normalized = value.trim();

        // Try pt (already in points)
        if let Some(num_str) = normalized.strip_suffix("pt") {
            return num_str.trim().parse::<f64>().ok();
        }

        // Try rem (convert to points: rem × 16 × 0.75)
        if let Some(num_str) = normalized.strip_suffix("rem") {
            return num_str.trim().parse::<f64>().ok().map(|n| n * 16.0 * 0.75);
        }

        // Try px (convert to points: px × 0.75)
        if let Some(num_str) = normalized.strip_suffix("px") {
            return num_str.trim().parse::<f64>().ok().map(|n| n * 0.75);
        }

        // Try em (treat as rem for now)
        if let Some(num_str) = normalized.strip_suffix("em") {
            return num_str.trim().parse::<f64>().ok().map(|n| n * 16.0 * 0.75);
        }

        // Try as unitless number
        normalized.parse::<f64>().ok()
    }

    /// Convert CSS value to typed JSON value
    fn to_json_value(value: &str) -> Value {
        let normalized = value.trim();

        // Strip units: ".25s" -> 0.25, "10px" -> 10
        if let Some(num_str) = normalized
            .strip_suffix('s')
            .or_else(|| normalized.strip_suffix("px"))
            .or_else(|| normalized.strip_suffix("em"))
            .or_else(|| normalized.strip_suffix("rem"))
            && let Ok(num) = num_str.parse::<f64>()
        {
            return json!(num);
        }

        // Unquote: '.25' -> 0.25, "'string'" -> "string"
        if let Some(unquoted) = normalized
            .strip_prefix('\'')
            .and_then(|s| s.strip_suffix('\''))
            .or_else(|| {
                normalized
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
            })
        {
            // Try as number first
            if let Ok(num) = unquoted.parse::<f64>() {
                return json!(num);
            }
            return json!(unquoted);
        }

        // Try as number
        if let Ok(num) = normalized.parse::<f64>() {
            return json!(num);
        }

        // Try as JSON (for arrays/objects if any)
        if let Ok(json) = serde_json::from_str(normalized) {
            return json;
        }

        // Fallback to string
        json!(normalized)
    }

    /// Parse CSS and extract all :root custom properties
    ///
    /// Only collects top-level :root declarations, not those nested
    /// in @media, @supports, or other at-rules.
    /// Variable names have the `--` prefix stripped.
    fn parse_css_variables(css: &str) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();

        let parser_opts = ParserOptions::default();

        let Ok(sheet) = StyleSheet::parse(css, parser_opts) else {
            return map;
        };

        for rule in &sheet.rules.0 {
            if let CssRule::Style(style) = rule {
                // Only consider top-level style rules that include a bare `:root`
                let Ok(selector_list) = style.selectors.to_css_string(PrinterOptions::default())
                else {
                    continue;
                };

                // Check if any comma-separated selector is exactly ":root"
                if !selector_list.split(',').any(|s| s.trim() == ":root") {
                    continue;
                }

                // Iterate over all declarations in the rule
                for (property, _) in style.declarations.iter() {
                    if let Property::Custom(custom_prop) = property {
                        // Only handle custom properties (starting with --), not unknown properties
                        if let CustomPropertyName::Custom(dashed_ident) = &custom_prop.name {
                            // Get the name and strip the -- prefix
                            let name = dashed_ident.0.as_ref().trim_start_matches("--").to_string();

                            // Convert the entire property to CSS, then extract the value
                            let mut prop_string = String::new();
                            if property
                                .to_css(
                                    &mut Printer::new(&mut prop_string, PrinterOptions::default()),
                                    false,
                                )
                                .is_ok()
                            {
                                // The property will be in format "--name: value"
                                // Extract just the value after the colon
                                if let Some(colon_pos) = prop_string.find(':') {
                                    let value = prop_string[colon_pos + 1..].trim().to_string();
                                    // "last one wins" if :root declares the same var multiple times
                                    map.insert(name, value);
                                }
                            }
                        }
                    }
                }
            }
            // All other rule kinds (including @media/@supports/@layer blocks) are skipped
        }

        map
    }

    /// Merge base theme variables with theme-specific CSS variables
    ///
    /// Parses the CSS and extracts custom properties, then merges them with
    /// the base theme variables (theme variables override base).
    fn merge_css_variables(css: &str) -> BTreeMap<String, String> {
        let mut variables = BASE_THEME_VARS.clone();
        variables.extend(Self::parse_css_variables(css));
        variables
    }
}

/// Get the relative path from current directory (using .. if needed)
fn relative_path(path: &Path) -> Option<String> {
    let current = current_dir().ok()?;
    diff_paths(path, &current).map(|p| p.display().to_string())
}

/// Lazy-loaded base theme variables parsed from base.css
static BASE_THEME_VARS: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    if let Some(file) = Web::get("themes/base.css") {
        let css = String::from_utf8_lossy(&file.data);
        Theme::parse_css_variables(&css)
    } else {
        BTreeMap::new()
    }
});

/// Get a list of available themes
///
/// # Arguments
/// * `base_path` - Optional base path for searching workspace themes. If None, uses current directory.
pub async fn list(base_path: Option<&Path>) -> Result<Vec<Theme>> {
    let mut themes = Vec::new();

    // Walk up directory tree looking for theme.css and stop at the first found
    let mut current = base_path
        .map(|p| p.to_path_buf())
        .or_else(|| current_dir().ok())
        .ok_or_else(|| eyre!("Failed to determine base path"))?;
    loop {
        let theme_path = current.join("theme.css");
        if theme_path.exists() {
            let location = relative_path(&theme_path);
            let css = read_to_string(&theme_path).await?;

            themes.push(Theme::new(ThemeType::Workspace, None, location, css, true));

            break;
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    // Get the themes in the themes config directory
    if let Ok(themes_dir) = get_app_dir(DirType::Themes, false)
        && themes_dir.exists()
        && let Ok(mut entries) = tokio::fs::read_dir(&themes_dir).await
    {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("css") {
                let name = path.file_stem().and_then(|s| s.to_str()).map(String::from);
                let location = Some(path.display().to_string());
                let css = read_to_string(&path).await.unwrap_or_default();

                themes.push(Theme::new(ThemeType::User, name, location, css, true));
            }
        }
    }

    // Get the builtin themes from web dist
    for filename in Web::iter() {
        if filename.starts_with("themes/")
            && filename.ends_with(".css")
            && !filename.contains("/base/")
            && filename != "themes/base.css"
            && let Some(file) = Web::get(&filename)
        {
            let name = filename
                .trim_start_matches("themes/")
                .trim_end_matches(".css")
                .to_string();
            let css = String::from_utf8_lossy(&file.data).to_string();

            themes.push(Theme::new(ThemeType::Builtin, Some(name), None, css, false));
        }
    }

    Ok(themes)
}

/// Get a theme by name, or the default theme if no name provided
///
/// # Arguments
/// * `name` - Optional theme name to look for
/// * `base_path` - Optional base path for searching workspace themes. If None, uses current directory.
///
/// # Resolution logic
/// If `name` is provided:
/// - Search user themes directory for matching name
/// - If not found, search builtin themes
/// - Return None if not found
///
/// If `name` is None (default resolution):
/// - Walk up directory tree for `theme.css` (workspace theme)
/// - If not found, look for `default.css` in user themes
/// - If not found, return builtin `stencila.css`
pub async fn get(name: Option<&str>, base_path: Option<&Path>) -> Result<Option<Theme>> {
    if let Some(name) = name {
        // Named theme: search user themes first, then builtins

        // Check user themes
        if let Ok(themes_dir) = get_app_dir(DirType::Themes, false) {
            let theme_path = themes_dir.join(format!("{}.css", name));
            if theme_path.exists() {
                let name = Some(name.to_string());
                let location = Some(theme_path.display().to_string());
                let css = read_to_string(&theme_path).await?;

                return Ok(Some(Theme::new(ThemeType::User, name, location, css, true)));
            }
        }

        // Check builtin themes
        let filename = format!("themes/{}.css", name);
        if let Some(file) = Web::get(&filename) {
            let name = Some(name.to_string());
            let css = String::from_utf8_lossy(&file.data).to_string();

            return Ok(Some(Theme::new(ThemeType::Builtin, name, None, css, false)));
        }

        // Not found
        return Ok(None);
    }

    // Default theme resolution: workspace -> default.css -> stencila.css

    // 1. Look for workspace theme.css
    let mut current = base_path
        .map(|p| p.to_path_buf())
        .or_else(|| current_dir().ok())
        .ok_or_else(|| eyre!("Failed to determine base path"))?;
    loop {
        let theme_path = current.join("theme.css");
        if theme_path.exists() {
            let location = relative_path(&theme_path);
            let css = read_to_string(&theme_path).await?;

            return Ok(Some(Theme::new(
                ThemeType::Workspace,
                None,
                location,
                css,
                true,
            )));
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }

    // 2. Look for default.css in user themes
    if let Ok(themes_dir) = get_app_dir(DirType::Themes, false) {
        let default_path = themes_dir.join("default.css");
        if default_path.exists() {
            let name = Some("default".to_string());
            let location = Some(default_path.display().to_string());
            let css = read_to_string(&default_path).await?;

            return Ok(Some(Theme::new(ThemeType::User, name, location, css, true)));
        }
    }

    // 3. Use builtin stencila.css
    if let Some(file) = Web::get("themes/stencila.css") {
        let name = Some("stencila".to_string());
        let css = String::from_utf8_lossy(&file.data).to_string();

        return Ok(Some(Theme::new(ThemeType::Builtin, name, None, css, false)));
    }

    // Shouldn't happen since stencila.css is embedded
    Ok(None)
}

/// Template for new themes
const THEME_TEMPLATE: &str = r#":root {
    /* Add your custom CSS variable overrides here */
    /* Example:
    --text-font-size: 18px;
    --heading-font-family: 'Georgia', serif;
    --color-accent: #3b82f6;
    */
}
"#;

/// Create a new workspace theme or user theme
///
/// # Arguments
/// * `name` - Optional name for the theme. If None creates `theme.css` in cwd.
/// * `force` - If true, overwrite existing files without prompting
pub async fn new(name: Option<String>, force: bool) -> Result<Option<PathBuf>> {
    let theme_path = if let Some(name) = name {
        // Named theme in user theme directory
        let themes_dir = get_app_dir(DirType::Themes, true)?;
        let mut path = themes_dir.join(name);
        if path.extension().is_none() {
            path.set_extension("css");
        }
        path
    } else {
        // Default to theme.css in current directory
        current_dir()?.join("theme.css")
    };

    // Check if file already exists
    if theme_path.exists() && !force {
        let answer = stencila_ask::ask(&format!(
            "Theme file `{}` already exists. Overwrite?",
            theme_path.display()
        ))
        .await?;

        if answer.is_no() {
            return Ok(None);
        }
    }

    // Ensure parent directory exists
    if let Some(parent) = theme_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Write the template
    write(&theme_path, THEME_TEMPLATE).await?;

    Ok(Some(theme_path))
}

/// Remove a user theme
///
/// # Arguments
/// * `name` - The name of the theme to remove (without .css extension)
/// * `force` - If true, skip confirmation prompt
pub async fn remove(name: &str, force: bool) -> Result<()> {
    let themes_dir = get_app_dir(DirType::Themes, false)?;
    let theme_path = themes_dir.join(format!("{}.css", name));

    if !theme_path.exists() {
        bail!("Theme `{}` not found in user themes directory", name);
    }

    // Confirm deletion unless force is set
    if !force {
        let answer = stencila_ask::ask(&format!(
            "Are you sure you want to remove the user theme `{}`?",
            name
        ))
        .await?;

        if answer.is_no() {
            return Ok(());
        }
    }

    remove_file(&theme_path).await?;

    Ok(())
}

/// Watch a theme for changes and receive updates through a channel
///
/// # Arguments
/// * `name` - Optional theme name. If None, watches workspace theme (theme.css)
/// * `base_path` - Optional base path for searching workspace themes. If None, uses current directory.
///
/// # Returns
/// A receiver that will receive theme updates when the file changes.
/// The watcher stops when the receiver is dropped.
///
/// # Errors
/// Returns an error if the theme cannot be found or the file watcher cannot be created.
pub async fn watch(
    name: Option<&str>,
    base_path: Option<&Path>,
) -> Result<mpsc::Receiver<Result<Theme>>> {
    // First, get the theme to find its location
    let theme = get(name, base_path)
        .await?
        .ok_or_else(|| eyre!("Theme not found"))?;

    let Some(location) = theme.location.clone() else {
        bail!("Theme has no file location and cannot be watched");
    };

    // Watch the parent directory to handle atomic writes (remove + rename)
    let watched_path = PathBuf::from(&location);
    let watched_dir = match watched_path.parent() {
        Some(parent) => parent.to_path_buf(),
        None => bail!("Cannot determine parent directory of theme file"),
    };
    let watched_filename = watched_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(String::from);

    let (sender, receiver) = mpsc::channel(100);
    let base_path_owned = base_path.map(|path| path.to_path_buf());
    let name_owned = name.map(String::from);

    // Spawn a background task to watch the file
    tokio::task::spawn(async move {
        let (file_sender, mut file_receiver) = mpsc::channel(100);

        let mut watcher = match notify::recommended_watcher(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Check if event affects our specific file
                    let affects_target = event.paths.iter().any(|path| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .and_then(|name| {
                                watched_filename.as_ref().map(|watched| name == watched)
                            })
                            .unwrap_or(false)
                    });

                    if affects_target
                        && matches!(
                            event.kind,
                            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                        )
                    {
                        let _ = file_sender.blocking_send(());
                    }
                }
            },
        ) {
            Ok(w) => w,
            Err(error) => {
                let _ = sender
                    .send(Err(eyre!("Failed to create file watcher: {error}")))
                    .await;
                return;
            }
        };

        if let Err(error) = watcher.watch(&watched_dir, RecursiveMode::NonRecursive) {
            let _ = sender
                .send(Err(eyre!("Failed to watch theme directory: {error}")))
                .await;
            return;
        }

        // Listen for file changes and send updates
        while let Some(()) = file_receiver.recv().await {
            // Re-read and reload the theme
            match get(name_owned.as_deref(), base_path_owned.as_deref()).await {
                Ok(Some(updated)) => {
                    if sender.send(Ok(updated)).await.is_err() {
                        // Receiver dropped, stop watching
                        break;
                    }
                }
                Ok(None) => {
                    // Theme no longer exists
                    let _ = sender.send(Err(eyre!("Theme no longer found"))).await;
                    break;
                }
                Err(e) => {
                    let _ = sender.send(Err(e)).await;
                    break;
                }
            }
        }
    });

    Ok(receiver)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A test of using the lightningcss API directly so we can see what it generates
    #[test]
    fn test_lightningcss() -> Result<()> {
        let css = r".temp{
            color: color-mix(in srgb, red 50%, blue);
            width: calc(10px + 5px)
        }";
        let mut sheet = StyleSheet::parse(&css, ParserOptions::default())?;

        let targets = Targets {
            include: Features::all(),
            ..Default::default()
        };
        sheet.minify(MinifyOptions {
            targets,
            ..Default::default()
        })?;

        let result = sheet.to_css(PrinterOptions {
            minify: true,
            ..Default::default()
        })?;
        assert_eq!(&result.code, ".temp{color:purple;width:15px}");

        Ok(())
    }

    #[test]
    fn test_compute_variables_simple_number() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --size: 16; }".into(),
            false,
        );
        let computed = theme.compute_variables();
        assert_eq!(computed.get("size"), Some(&json!(16.0)));
    }

    #[test]
    fn test_compute_variables_strip_units() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --width: 10px; --time: .25s; }".into(),
            false,
        );
        let computed = theme.compute_variables();
        assert_eq!(computed.get("width"), Some(&json!(10.0)));
        assert_eq!(computed.get("time"), Some(&json!(0.25)));
    }

    #[test]
    fn test_compute_variables_unquote() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            r#":root { --num: '.25'; --str: 'hello'; }"#.into(),
            false,
        );
        let computed = theme.compute_variables();
        assert_eq!(computed.get("num"), Some(&json!(0.25)));
        assert_eq!(computed.get("str"), Some(&json!("hello")));
    }

    #[test]
    fn test_compute_variables_var_resolution() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --base: 10; --double: var(--base); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        assert_eq!(computed.get("double"), Some(&json!(10.0)));
    }

    #[test]
    fn test_compute_variables_calc_simple() {
        // Note: lightningcss may not evaluate unitless calc(), so we use px
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --result: calc(10px + 5px); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        // lightningcss should evaluate to "15px", then we strip units
        assert_eq!(computed.get("result"), Some(&json!(15.0)));
    }

    #[test]
    fn test_compute_variables_calc_with_var() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --base: 10px; --double: calc(var(--base) * 2); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        // Should resolve var, calc, and strip units
        assert_eq!(computed.get("double"), Some(&json!(20.0)));
    }

    #[test]
    fn test_compute_variables_nested_var() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --a: 5; --b: var(--a); --c: var(--b); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        assert_eq!(computed.get("c"), Some(&json!(5.0)));
    }

    #[test]
    fn test_compute_variables_var_with_fallback() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --value: var(--missing, 42); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        assert_eq!(computed.get("value"), Some(&json!(42.0)));
    }

    #[test]
    fn test_compute_variables_color_mix() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --plot-color-1: color-mix(in srgb, red 50%, blue); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        // color-mix should be normalized to hex
        let color = computed.get("plot-color-1").and_then(|v| v.as_str());
        assert_eq!(color.unwrap_or_default(), "#800080");
    }

    #[test]
    fn test_normalize_color_hsl_to_hex() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --plot-color-1: hsl(217 91% 60%); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        let color = computed.get("plot-color-1").and_then(|v| v.as_str());
        assert_eq!(color.unwrap_or_default(), "#3c83f6");
    }

    #[test]
    fn test_normalize_color_oklch_to_hex() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --plot-background: oklch(55% 0.2 210); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        let color = computed.get("plot-background").and_then(|v| v.as_str());
        assert_eq!(color.unwrap_or_default(), "#008192");
    }

    #[test]
    fn test_normalize_length_rem_to_points() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --plot-font-size: 1rem; }".into(),
            false,
        );
        let computed = theme.compute_variables();
        // 1rem = 16px = 12pt (16 * 0.75)
        assert_eq!(computed.get("plot-font-size"), Some(&json!(12.0)));
    }

    #[test]
    fn test_normalize_length_px_to_points() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --plot-line-width: 2px; }".into(),
            false,
        );
        let computed = theme.compute_variables();
        // 2px = 1.5pt (2 * 0.75)
        assert_eq!(computed.get("plot-line-width"), Some(&json!(1.5)));
    }

    #[test]
    fn test_normalize_length_calc_to_points() {
        let theme = Theme::new(
            ThemeType::Builtin,
            Some("test".into()),
            None,
            ":root { --base: 16px; --plot-font-size: calc(var(--base) * 0.85); }".into(),
            false,
        );
        let computed = theme.compute_variables();
        // calc(16px * 0.85) = 13.6px = 10.2pt (13.6 * 0.75)
        assert_eq!(computed.get("plot-font-size"), Some(&json!(10.2)));
    }

    #[test]
    fn test_resolve_var_references_circular() {
        // Test that circular references don't cause infinite recursion
        let mut vars = BTreeMap::new();
        vars.insert("a".to_string(), "var(--b)".to_string());
        vars.insert("b".to_string(), "var(--a)".to_string());

        let theme = Theme {
            r#type: ThemeType::Builtin,
            name: Some("test".into()),
            location: None,
            content: String::new(),
            variables: vars,
        };
        let computed = theme.compute_variables();
        assert_eq!(computed.get("a"), Some(&json!("var(--b)")));
    }
}
