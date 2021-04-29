use eyre::Result;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Get the directory where configuration data is stored
pub fn config(ensure: bool) -> Result<PathBuf> {
    let config_base = dirs_next::config_dir().unwrap_or_else(|| env::current_dir().unwrap());
    let dir = match env::consts::OS {
        "macos" => config_base.join("Stencila"),
        "windows" => config_base.join("Stencila").join("Config"),
        _ => config_base.join("stencila"),
    };
    if ensure {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Get the directory where logs are stored
pub fn logs(ensure: bool) -> Result<PathBuf> {
    let config = config(false)?;
    let dir = match env::consts::OS {
        "macos" | "windows" => config.join("Logs"),
        _ => config.join("logs"),
    };
    if ensure {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// Get the directory within which plugins and their configurations are installed
pub fn plugins(ensure: bool) -> Result<PathBuf> {
    let config = config(false)?;
    let dir = match env::consts::OS {
        "macos" | "windows" => config.join("Plugins"),
        _ => config.join("plugins"),
    };
    if ensure {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}
