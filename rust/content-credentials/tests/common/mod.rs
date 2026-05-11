//! Shared test helpers.

#![allow(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use tempfile::TempDir;

static CONFIG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Per-test guard that points the directories crate at a hermetic temp dir,
/// so tests don't write to the real `~/.config/stencila`.
pub struct ConfigGuard {
    _tmp: TempDir,
    _lock: MutexGuard<'static, ()>,
    prev_xdg: Option<String>,
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        match self.prev_xdg.take() {
            Some(prev) => unsafe { env::set_var("XDG_CONFIG_HOME", prev) },
            None => unsafe { env::remove_var("XDG_CONFIG_HOME") },
        }
    }
}

pub fn set_isolated_config_dir() -> ConfigGuard {
    let lock = CONFIG_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    let tmp = TempDir::new().expect("tmp");
    let path: PathBuf = tmp.path().to_path_buf();
    let prev_xdg = env::var("XDG_CONFIG_HOME").ok();
    unsafe { env::set_var("XDG_CONFIG_HOME", &path) };

    ConfigGuard {
        _tmp: tmp,
        _lock: lock,
        prev_xdg,
    }
}
