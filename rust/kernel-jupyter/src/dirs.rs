use std::{env, path::PathBuf, process::Command};

/// Run the `jupyter` command to get lists of directories
///
/// This can be necessary if Jupyter has been installed using something
/// like `mamba` (and maybe Nix?) in which case the rules described in
/// the Jupyter documentation and implemented below may not apply.
///
/// An optimization could be to store `static` results and throttle
/// calls to `jupyter`.
fn jupyter_paths() -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut data = Vec::new();
    let mut runtime = Vec::new();
    if let Ok(output) = Command::new("jupyter").arg("--path").output() {
        if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
            let mut group = "";
            for line in stdout.lines() {
                let line = line.trim();
                if line.ends_with(':') {
                    group = line;
                } else if group == "data:" {
                    data.push(PathBuf::from(line));
                } else if group == "runtime:" {
                    runtime.push(PathBuf::from(line));
                }
            }
        }
    }
    (data, runtime)
}

/// Get *the* Jupyter data directory.
///
/// See https://jupyter.readthedocs.io/en/latest/use/jupyter-directories.html.
pub fn data_dir() -> PathBuf {
    let (dirs, ..) = jupyter_paths();
    if let Some(first) = dirs.first() {
        return first.clone();
    }

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
    let (mut dirs, ..) = jupyter_paths();

    if let Ok(path) = env::var("JUPYTER_PATH") {
        #[cfg(target_os = "windows")]
        const SEP: char = ';';
        #[cfg(not(target_os = "windows"))]
        const SEP: char = ':';

        let paths = path.split(SEP).map(PathBuf::from).collect();
        append_missing(&mut dirs, paths);
    }

    append_missing(
        &mut dirs,
        vec![
            data_dir(),
            #[cfg(not(target_os = "windows"))]
            PathBuf::from("/usr/local/share/jupyter"),
            #[cfg(not(target_os = "windows"))]
            PathBuf::from("/usr/share/jupyter"),
        ],
    );

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
    let (.., mut dirs) = jupyter_paths();

    if let Ok(path) = env::var("JUPYTER_RUNTIME_DIR") {
        push_missing(&mut dirs, PathBuf::from(path));
    }

    push_missing(&mut dirs, data_dir().join("runtime"));

    if let Some(runtime_dir) = ::dirs::runtime_dir() {
        push_missing(&mut dirs, runtime_dir.join("jupyter"));
    }

    dirs
}

/// Add a path if it is missing from a set of paths
fn push_missing(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

/// Append paths if they are missing from a set of paths
fn append_missing(paths: &mut Vec<PathBuf>, others: Vec<PathBuf>) {
    for path in others {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
}
