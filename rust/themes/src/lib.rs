use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use eyre::{Result, bail};
use pathdiff::diff_paths;

use stencila_dirs::{DirType, get_app_dir};
use stencila_web_dist::Web;
use tokio::fs::{read_to_string, remove_file, write};

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
}

/// Get the relative path from current directory (using .. if needed)
fn relative_path(path: &Path) -> Option<String> {
    let current = current_dir().ok()?;
    diff_paths(path, &current).map(|p| p.display().to_string())
}

/// Get a list of available themes
pub async fn list() -> Result<Vec<Theme>> {
    let mut themes = Vec::new();

    // Walk up directory tree looking for theme.css and stop at the first found
    let mut current = current_dir()?;
    loop {
        let theme_path = current.join("theme.css");
        if theme_path.exists() {
            let css = read_to_string(&theme_path).await?;

            themes.push(Theme {
                r#type: ThemeType::Workspace,
                name: None,
                location: relative_path(&theme_path),
                content: css,
            });
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
                let content = read_to_string(&path).await.unwrap_or_default();

                themes.push(Theme {
                    r#type: ThemeType::User,
                    name,
                    location: Some(path.display().to_string()),
                    content,
                });
            }
        }
    }

    // Get the builtin themes from web dist
    for filename in Web::iter() {
        if filename.starts_with("themes/")
            && filename.ends_with(".css")
            && !filename.contains("/base/")
            && let Some(file) = Web::get(&filename)
        {
            let name = filename
                .trim_start_matches("themes/")
                .trim_end_matches(".css")
                .to_string();

            let css = String::from_utf8_lossy(&file.data);

            themes.push(Theme {
                r#type: ThemeType::Builtin,
                name: Some(name),
                location: None,
                content: css.to_string(),
            });
        }
    }

    Ok(themes)
}

/// Get a theme by name, or the default theme if no name provided
///
/// # Arguments
/// * `name` - Optional theme name to look for
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
pub async fn get(name: Option<&str>) -> Result<Option<Theme>> {
    if let Some(name) = name {
        // Named theme: search user themes first, then builtins

        // Check user themes
        if let Ok(themes_dir) = get_app_dir(DirType::Themes, false) {
            let theme_path = themes_dir.join(format!("{}.css", name));
            if theme_path.exists() {
                let css = read_to_string(&theme_path).await?;

                return Ok(Some(Theme {
                    r#type: ThemeType::User,
                    name: Some(name.to_string()),
                    location: Some(theme_path.display().to_string()),
                    content: css,
                }));
            }
        }

        // Check builtin themes
        let filename = format!("themes/{}.css", name);
        if let Some(file) = Web::get(&filename) {
            let css = String::from_utf8_lossy(&file.data);

            return Ok(Some(Theme {
                r#type: ThemeType::Builtin,
                name: Some(name.to_string()),
                location: None,
                content: css.to_string(),
            }));
        }

        // Not found
        return Ok(None);
    }

    // Default theme resolution: workspace -> default.css -> stencila.css

    // 1. Look for workspace theme.css
    let mut current = current_dir()?;
    loop {
        let theme_path = current.join("theme.css");
        if theme_path.exists() {
            let css = read_to_string(&theme_path).await?;

            return Ok(Some(Theme {
                r#type: ThemeType::Workspace,
                name: None,
                location: relative_path(&theme_path),
                content: css,
            }));
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
            let css = read_to_string(&default_path).await?;

            return Ok(Some(Theme {
                r#type: ThemeType::User,
                name: Some("default".to_string()),
                location: Some(default_path.display().to_string()),
                content: css,
            }));
        }
    }

    // 3. Use builtin stencila.css
    if let Some(file) = Web::get("themes/stencila.css") {
        let css = String::from_utf8_lossy(&file.data);

        return Ok(Some(Theme {
            r#type: ThemeType::Builtin,
            name: Some("stencila".to_string()),
            location: None,
            content: css.to_string(),
        }));
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
            "Are you sure you want to remove the theme `{}`?",
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
