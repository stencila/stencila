use super::{Kernel, KernelTrait};
use crate::{errors::incompatible_language, utils::keys};
use defaults::Defaults;
use eyre::{bail, Result};
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, RwLock},
};
use stencila_schema::Node;
use tokio::{process::Command, task::JoinHandle};

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
enum Status {
    Pending,
    Started,
    Unresponsive,
    Finished,
    Failed,
}

/// A Jupyter kernel
///
/// Most of the fields of this `struct` reflect those in a "kernel spec" and are read from a `kernel.json` file.
/// See https://jupyter-client.readthedocs.io/en/stable/kernels.html#kernel-specs.
/// Comments below are copied from there.
#[skip_serializing_none]
#[derive(Debug, Defaults, JsonSchema, Deserialize, Serialize)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct JupyterKernel {
    /// The id of the kernel instance
    id: String,

    /// The name of the kernel e.g. `python3`, `ir`
    name: String,

    /// The path of the kernel specification file
    path: PathBuf,

    /// A list of command line arguments used to start the kernel.
    /// The text `{connection_file}` in any argument will be replaced with the path to the connection file.
    argv: Vec<String>,

    /// The kernel’s name as it should be displayed in the UI.
    /// Unlike the kernel name used in the API, this can contain arbitrary unicode characters.
    display_name: String,

    /// The name of the language of the kernel. When loading notebooks,
    /// if no matching kernelspec key (may differ across machines) is found, a kernel with a matching
    /// language will be used. This allows a notebook written on any Python or Julia kernel to be
    /// properly associated with the user’s Python or Julia kernel, even if they aren’t listed under
    /// the same name as the author’s.
    language: String,

    /// May be either `signal` or `message` and specifies how a client is supposed to interrupt
    /// cell execution on this kernel, either by sending an interrupt `signal` via the operating
    /// system’s signalling facilities (e.g. `SIGINT` on POSIX systems), or by sending an `interrupt_request`
    /// message on the control channel (see Kernel interrupt). If this is not specified the client
    /// will default to `signal` mode.
    interrupt_mode: Option<String>,

    /// A dictionary of environment variables to set for the kernel. These will be added to the current
    /// environment variables before the kernel is started. Existing environment variables can be
    /// referenced using `${<ENV_VAR>}` and will be substituted with the corresponding value.
    env: Option<HashMap<String, String>>,

    /// A dictionary of additional attributes about this kernel; used by clients to aid in kernel selection.
    /// Metadata added here should be namespaced for the tool reading and writing that metadata.
    metadata: Option<HashMap<String, serde_json::Value>>,

    /// The details (e.g. port numbers) of the connection to the kernel
    #[serde(skip_deserializing)]
    connection: Option<Connection>,

    /// The system id of the kernel process
    pid: Option<u32>,

    /// The status of the kernel
    #[def = "Arc::new(tokio::sync::RwLock::new(Status::Pending))"]
    #[serde(skip)]
    status: Arc<tokio::sync::RwLock<Status>>,

    /// The async task that runs the kernel
    #[serde(skip)]
    run_task: Option<JoinHandle<()>>,

    /// The async task that monitors the kernel
    #[serde(skip)]
    monitor_task: Option<JoinHandle<()>>,
}

impl Clone for JupyterKernel {
    /// Clone a `JupyterKernel`
    ///
    /// Needed because tasks can not be cloned as it would using `Clone` macro.
    /// `JupyterKernel` needs to be `Clone`-able for inspection using the `show`
    /// command (at least, given how we currently do that).
    fn clone(&self) -> Self {
        JupyterKernel {
            id: self.id.clone(),
            name: self.name.clone(),
            path: self.path.clone(),
            argv: self.argv.clone(),
            display_name: self.display_name.clone(),
            language: self.language.clone(),
            interrupt_mode: self.interrupt_mode.clone(),
            env: self.env.clone(),
            metadata: self.metadata.clone(),
            connection: self.connection.clone(),
            pid: self.pid,
            status: self.status.clone(),

            ..Default::default()
        }
    }
}

impl JupyterKernel {
    /// Create a `JupyterKernel` variant.
    pub fn create(id: &str, language: &str) -> Result<Kernel> {
        let mut kernel = JupyterKernel::find(language)?;
        kernel.id = id.to_string();

        Ok(Kernel::Jupyter(kernel))
    }

    /// Find a `JupyterKernel` for the given language.
    ///
    /// Searches for an installed kernel with support for the language.
    /// Is optimized to avoid unnecessary disk reads.
    pub fn find(language: &str) -> Result<JupyterKernel> {
        let specs = KERNEL_SPECS.read().unwrap();

        // Is there is a kernelspec already read with the same name?
        if let Some(kernel) = specs.get(language) {
            return Ok(kernel.clone());
        }

        // Is there is a kernelspec already read that supports the language?
        for kernel in specs.values() {
            if kernel.language(Some(language.to_string())).is_ok() {
                return Ok(kernel.clone());
            }
        }

        drop(specs);

        // For each Jupyter data directory..
        for dir in JupyterKernel::data_dirs() {
            let kernels = dir.join("kernels");

            // Is there is a kernelspec with the same name?
            let path = kernels.join(language).join("kernel.json");
            if path.exists() {
                let kernel = JupyterKernel::read(language, &path)?;
                if kernel.language(Some(language.to_string())).is_ok() {
                    return Ok(kernel);
                }
            }

            // Is there is a kernelspec that supports the language?
            for dir in kernels.read_dir()?.flatten() {
                let path = dir.path().join("kernel.json");
                if path.exists() {
                    let name = dir.file_name().to_string_lossy().to_string();
                    let kernel = JupyterKernel::read(&name, &path)?;
                    if kernel.language(Some(language.to_string())).is_ok() {
                        return Ok(kernel);
                    }
                }
            }
        }

        bail!(
            "Unable to find a Jupyter kernel for language `{}`; perhaps you need to install one?",
            language
        )
    }

    /// Get *the* Jupyter data directory.
    ///
    /// See https://jupyter.readthedocs.io/en/latest/use/jupyter-directories.html.
    fn data_dir() -> PathBuf {
        if let Ok(path) = env::var("JUPYTER_DATA_DIR") {
            PathBuf::from(path)
        } else if let Some(data_dir) = dirs_next::data_dir() {
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

    /// Get all the directories where Jupyter stores data files such as kernel specs.
    ///
    /// See https://jupyter.readthedocs.io/en/latest/use/jupyter-directories.html
    /// and `jupyter --paths`.
    fn data_dirs() -> Vec<PathBuf> {
        let mut dirs = if let Ok(path) = env::var("JUPYTER_PATH") {
            #[cfg(target_os = "windows")]
            const SEP: char = ';';
            #[cfg(not(target_os = "windows"))]
            const SEP: char = ':';
            path.split(SEP).map(PathBuf::from).collect()
        } else {
            vec![]
        };

        dirs.push(JupyterKernel::data_dir());
        dirs.push(PathBuf::from("/usr/local/share/jupyter"));
        dirs.push(PathBuf::from("/usr/share/jupyter"));

        dirs
    }

    /// Get the directory where Jupyter stores runtime files e.g. connection files.
    ///
    /// See https://jupyter.readthedocs.io/en/latest/use/jupyter-directories.html
    /// and `jupyter -runtime-dir`.
    fn runtime_dir() -> PathBuf {
        if let Ok(path) = env::var("JUPYTER_RUNTIME_DIR") {
            PathBuf::from(path)
        } else {
            #[cfg(target_os = "linux")]
            return match dirs_next::runtime_dir() {
                Some(runtime_dir) => runtime_dir.join("jupyter"),
                None => JupyterKernel::data_dir().join("runtime"),
            };

            #[cfg(not(target_os = "linux"))]
            return JupyterKernel::data_dir().join("runtime");
        }
    }

    /// Read a `kernel.json` file and store in `KERNEL_SPECS`
    fn read(name: &str, path: &Path) -> Result<JupyterKernel> {
        let json = fs::read_to_string(path)?;
        let mut kernel: JupyterKernel = serde_json::from_str(&json)?;
        kernel.name = name.to_string();
        kernel.path = path.to_path_buf();

        let mut specs = KERNEL_SPECS.write().unwrap();
        specs.insert(name.to_string(), kernel.clone());

        Ok(kernel)
    }
}

impl KernelTrait for JupyterKernel {
    fn language(&self, language: Option<String>) -> Result<String> {
        let canonical = Ok(self.language.clone());
        if let Some(language) = language {
            if self.language.to_lowercase() == language.to_lowercase() {
                canonical
            } else {
                bail!(incompatible_language::<Self>(&language))
            }
        } else {
            canonical
        }
    }

    fn start(&mut self) -> Result<()> {
        if self.run_task.is_some() {
            return Ok(());
        }

        let connection = Connection::new(&self.id);
        connection.write_file()?;

        let args: Vec<String> = self
            .argv
            .iter()
            .map(|arg| {
                arg.replace(
                    "{connection_file}",
                    &connection.path.to_string_lossy().to_string(),
                )
            })
            .collect();

        let child = Command::new(&args[0])
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let pid = child.id();

        let id = self.id.clone();
        let status = self.status.clone();
        let run_task = tokio::spawn(async move {
            tracing::debug!("Running kernel `{}` with args `{:?}`", id, args);

            let output = child
                .wait_with_output()
                .await
                .expect("Kernel could not be executed");

            if output.status.success() {
                tracing::debug!("Kernel `{}` exited successfully", id);
                *(status.write().await) = Status::Finished;
            } else {
                tracing::error!(
                    "Kernel `{}` had non-zero exit status: {}",
                    id,
                    output.status
                );
                *(status.write().await) = Status::Failed;
            }

            if !output.stderr.is_empty() {
                tracing::error!(
                    "Kernel `{}` had error message: {}",
                    id,
                    &String::from_utf8_lossy(&output.stderr)
                )
            }
        });

        let id = self.id.clone();
        let status = self.status.clone();
        let url = connection.heartbeat_url();
        let monitor_task = tokio::spawn(async move {
            use tokio::time::{sleep, Duration};

            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::REQ).expect("UNable to create socket");

            let result = socket.connect(&url);
            if let Err(error) = result {
                tracing::error!(
                    "When connecting to heartbeat socket for kernel `{}`: {}",
                    id,
                    error
                );
                *(status.write().await) = Status::Unresponsive;
                return;
            }

            loop {
                let result = socket.send("", 0).and_then(|_| socket.recv_msg(0));
                if let Err(error) = result {
                    tracing::error!("When checking for heartbeat for kernel `{}`: {}", id, error);
                    *(status.write().await) = Status::Unresponsive;
                    return;
                } else {
                    tracing::debug!("Got heartbeat reply from kernel `{}`", id)
                }
                sleep(Duration::from_secs(1)).await;
            }
        });

        self.connection = Some(connection);
        self.pid = pid;
        self.run_task = Some(run_task);
        self.monitor_task = Some(monitor_task);

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        match &self.run_task {
            Some(join_handle) => join_handle.abort(),
            None => {}
        }

        Ok(())
    }

    fn get(&self, _name: &str) -> Result<Node> {
        Ok(Node::String("TODO".to_string()))
    }

    fn set(&mut self, _name: &str, _value: Node) -> Result<()> {
        Ok(())
    }

    fn exec(&mut self, _code: &str) -> Result<Vec<Node>> {
        Ok(vec![Node::String("TODO".to_string())])
    }
}

/// The global store of Jupyter kernels
///
/// Note that `super::KernelSpace` holds instances of kernels for each document whereas this
/// holds instances of the kernels specs read from `kernel.json` as an optimization to avoid
/// re-reading them from disk.
static KERNEL_SPECS: Lazy<Arc<RwLock<HashMap<String, JupyterKernel>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// A Jupyter kernel connection
///
/// See https://jupyter-client.readthedocs.io/en/stable/kernels.html#connection-files
#[derive(Debug, Clone, Defaults, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct Connection {
    path: PathBuf,

    #[def = "\"tcp\".to_string()"]
    transport: String,

    #[def = "\"127.0.0.1\".to_string()"]
    ip: String,

    #[def = "\"hmac-sha256\".to_string()"]
    signature_scheme: String,

    #[def = "Connection::generate_key()"]
    key: String,

    #[def = "Connection::pick_port()"]
    control_port: u16,

    #[def = "Connection::pick_port()"]
    shell_port: u16,

    #[def = "Connection::pick_port()"]
    stdin_port: u16,

    #[def = "Connection::pick_port()"]
    hb_port: u16,

    #[def = "Connection::pick_port()"]
    iopub_port: u16,
}

impl Connection {
    fn new(id: &str) -> Self {
        let name = format!("stencila-{}.json", id);
        Connection {
            path: JupyterKernel::runtime_dir().join(name),
            ..Default::default()
        }
    }

    fn generate_key() -> String {
        keys::generate()
    }

    fn pick_port() -> u16 {
        portpicker::pick_unused_port().expect("There are no free ports")
    }

    fn write_file(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(&self.path, json)?;
        Ok(())
    }

    fn base_url(&self) -> String {
        format!("{}://{}:", transport = self.transport, ip = self.ip)
    }

    fn heartbeat_url(&self) -> String {
        [self.base_url(), self.hb_port.to_string()].concat()
    }
}
