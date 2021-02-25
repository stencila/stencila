use std::env;
use std::path::PathBuf;

/// Get the directory where configuration data is stored
pub fn config() -> PathBuf {
    let config_base = dirs::config_dir().unwrap_or_else(|| env::current_dir().unwrap());
    match env::consts::OS {
        "macos" => config_base.join("Stencila"),
        "windows" => config_base.join("Stencila").join("Config"),
        _ => config_base.join("stencila"),
    }
}

/// Get the directory within which plugins and their own configurations are installed
pub fn plugins() -> PathBuf {
    let config = config();
    match env::consts::OS {
        "macos" => config.join("Plugins"),
        "windows" => config.join("Plugins"),
        _ => config.join("plugins"),
    }
}
