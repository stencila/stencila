use std::{env, path::PathBuf};

/// Get *the* Jupyter data directory.
///
/// See https://jupyter.readthedocs.io/en/latest/use/jupyter-directories.html.
pub fn data_dir() -> PathBuf {
    if let Ok(path) = env::var("JUPYTER_DATA_DIR") {
        PathBuf::from(path)
    } else if let Some(data_dir) = ::dirs::data_dir() {
        #[cfg(target_os = "macos")]
        return data_dir
            .parent()
            .expect("Should have a parent dir")
            .join("Jupyter");

        #[cfg(not(target_os = "macos"))]
        return data_dir.join("jupyter");
    } else {
        PathBuf::from(".")
    }
}

/// Get all the directories where Jupyter stores data files.
///
/// See https://jupyter.readthedocs.io/en/latest/use/jupyter-directories.html
/// and `jupyter --paths`.
pub fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = if let Ok(path) = env::var("JUPYTER_PATH") {
        #[cfg(target_os = "windows")]
        const SEP: char = ';';
        #[cfg(not(target_os = "windows"))]
        const SEP: char = ':';
        path.split(SEP).map(PathBuf::from).collect()
    } else {
        vec![]
    };

    dirs.push(data_dir());
    dirs.push(PathBuf::from("/usr/local/share/jupyter"));
    dirs.push(PathBuf::from("/usr/share/jupyter"));

    dirs
}

/// Get the directories where Jupyter may store kernel specs.
pub fn kernel_dirs() -> Vec<PathBuf> {
    data_dirs()
        .into_iter()
        .map(|path| path.join("kernels"))
        .collect()
}

/// Get the directories where Jupyter may store runtime files e.g. connection files.
///
/// See https://jupyter.readthedocs.io/en/latest/use/jupyter-directories.html
/// and `jupyter --runtime-dir`. To avoid brittleness this returns multiple options.
pub fn runtime_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(path) = env::var("JUPYTER_RUNTIME_DIR") {
        dirs.push(PathBuf::from(path))
    }

    dirs.push(data_dir().join("runtime"));

    if let Some(runtime_dir) = ::dirs::runtime_dir() {
        dirs.push(runtime_dir.join("jupyter"));
    }

    dirs
}
