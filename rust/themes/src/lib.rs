use std::{
    collections::BTreeMap,
    env::current_dir,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use eyre::{Result, bail};
use lightningcss::{
    printer::Printer,
    properties::{Property, custom::CustomPropertyName},
    rules::CssRule,
    stylesheet::{ParserOptions, PrinterOptions, StyleSheet},
    traits::ToCss,
};
use pathdiff::diff_paths;
use tokio::fs::{read_to_string, remove_file, write};

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
        .ok_or_else(|| eyre::eyre!("Failed to determine base path"))?;
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
        .ok_or_else(|| eyre::eyre!("Failed to determine base path"))?;
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
