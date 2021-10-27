use super::{Kernel, KernelStatus, KernelTrait};
use crate::{
    errors::incompatible_language,
    utils::{jupyter::translate_error, keys, uuids},
};
use async_trait::async_trait;
use defaults::Defaults;
use derivative::Derivative;
use eyre::{bail, eyre, Result};
use hmac::{Hmac, NewMac};
use once_cell::sync::Lazy;
use path_slash::PathBufExt;
use reqwest::StatusCode;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use sha2::Sha256;
use std::{
    collections::HashMap,
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};
use stencila_schema::Node;
use tokio::{
    process::Command,
    sync::{mpsc, Mutex, RwLock},
    task::JoinHandle,
};
use zmq::Socket;

/// A Jupyter server
///
/// Used to access information about currently running kernels so that they
/// can be associated with notebook files and connected to if necessary.
#[skip_serializing_none]
#[derive(Debug, Defaults, JsonSchema, Deserialize, Serialize)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct JupyterServer {
    base_url: String,
    hostname: String,
    notebook_dir: PathBuf,
    password: bool,
    pid: u32,
    port: u32,
    secure: bool,
    sock: String,
    token: String,
    url: String,
}

impl JupyterServer {
    /// Get a list of running Jupyter servers
    ///
    /// Scans the Jupyter runtime directory for `nbserver-*.json` files and
    /// checks that they are running by requesting from the URL with the token.
    /// This avoids issues with "zombie" `nbserver-*.json` files.
    pub async fn running() -> Result<HashMap<String, JupyterServer>> {
        let pattern = JupyterKernel::data_dir()
            .join("runtime")
            .join("nbserver-*.json")
            .to_slash_lossy();

        let files = glob::glob(&pattern)?.flatten();

        let client = reqwest::Client::new();

        let mut map = HashMap::new();
        for entry in files {
            let json = fs::read_to_string(entry)?;
            let server: JupyterServer = serde_json::from_str(&json)?;

            let url = format!("{}api/status?token={}", server.url, server.token);
            match client.get(url).send().await {
                Ok(response) => {
                    if response.status() == StatusCode::FORBIDDEN {
                        tracing::debug!("Unable to authenticate with Jupyter server; skipping");
                        continue;
                    }
                }
                Err(..) => {
                    tracing::debug!("Unable to send request to Jupyter server; skipping");
                    continue;
                }
            };

            map.insert(server.url.clone(), server);
        }

        Ok(map)
    }
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

    /// The details (e.g. port numbers, HMAC keys) of the connection to the kernel
    ///
    /// Written to a connection file and passed to the kernel when it is started.
    #[serde(skip_deserializing)]
    connection: Option<JupyterConnection>,

    /// The kernel session id
    ///
    /// Note that within a Stencila project session there may be several Jupyter kernel sessions.
    /// These are independent concepts. From the Jupyter docs:
    ///
    /// "A client session id, in message headers from a client, should be unique among all clients
    /// connected to a kernel. When a client reconnects to a kernel, it should use the same client
    /// session id in its message headers. When a client restarts, it should generate a new client
    /// session id."
    #[def = "uuids::generate(uuids::Family::Generic)"]
    session: String,

    /// The status of the kernel
    #[def = "Arc::new(RwLock::new(KernelStatus::Pending))"]
    #[serde(skip)]
    status: Arc<RwLock<KernelStatus>>,

    /// Details of the kernel and connection to it once started
    #[serde(skip)]
    details: Option<JupyterDetails>,
}

/// Runtime details of a kernel and the connection to it
///
/// Used to group most of the details that do not / can not be serialized
/// and which are only available once the kernel has been started
#[derive(Derivative)]
#[derivative(Debug)]
struct JupyterDetails {
    /// The system id of the kernel process
    ///
    /// Will be `None` if the kernel was started externally.
    pid: Option<u32>,

    /// The HMAC used when signing messages
    ///
    /// Derived from the connection's `key`.
    hmac: HmacSha256,

    /// The socket to send Jupyter "shell" commands to
    #[derivative(Debug = "ignore")]
    shell_socket: Arc<Mutex<Socket>>,

    /// The receiver for IOPub messages
    iopub_receiver: mpsc::Receiver<JupyterMessage>,

    /// The async task that runs the kernel
    ///
    /// Will be `None` if the kernel was started externally.
    run_task: Option<JoinHandle<()>>,

    /// The async task that subscribes to messages from the kernel
    subscribe_task: JoinHandle<()>,

    /// The async task that monitors the kernel
    monitor_task: JoinHandle<()>,
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
            session: self.session.clone(),
            status: self.status.clone(),

            ..Default::default()
        }
    }
}

impl JupyterKernel {
    /// Get a list of Jupyter kernels available in the current environment
    pub async fn available() -> Result<Vec<String>> {
        let mut list = Vec::new();

        for dir in JupyterKernel::data_dirs() {
            let kernels = dir.join("kernels");
            if !kernels.exists() {
                continue;
            }

            for dir in kernels.read_dir()?.flatten() {
                let path = dir.path().join("kernel.json");
                if path.exists() {
                    let name = dir.file_name().to_string_lossy().to_string();
                    let kernel = JupyterKernel::read(&name, &path).await?;
                    list.push(kernel.language(None)?)
                }
            }
        }

        Ok(list)
    }

    /// Get a list of Jupyter kernels that are currently running
    ///
    /// Generating a kernel list could be done by scanning the disk for kernel connection files instead.
    /// However, to be able to associate each kernel with a `ipynb` file (and generate the
    /// kernel's `notebook` field) we need to access the Jupyter Server API .
    pub async fn running() -> Result<HashMap<String, serde_json::Value>> {
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();

        let client = reqwest::Client::new();
        for (url, server) in JupyterServer::running().await? {
            // Get the list of sessions (which allow association of notebooks and kernels)
            let url = format!("{}api/sessions?token={}", url, server.token);
            let response = match client.get(url).send().await {
                Ok(response) => response,
                Err(error) => {
                    tracing::debug!(
                        "Unable to send request to Jupyter server; it is probably not running: {}",
                        error
                    );
                    continue;
                }
            };
            let response = match response.error_for_status() {
                Ok(response) => response,
                Err(error) => bail!(error),
            };
            let json = response.text().await?;

            let sessions: Vec<serde_json::Value> = serde_json::from_str(&json)?;
            for session in sessions.into_iter() {
                if let Some(kernel) = session.get("kernel") {
                    let mut kernel = kernel.clone();
                    let id = kernel
                        .get("id")
                        .and_then(|id| id.as_str())
                        .unwrap_or_default()
                        .to_string();
                    if let Some(notebook) = session.get("notebook").and_then(|nb| nb.get("path")) {
                        if let Ok(notebook) = server
                            .notebook_dir
                            .join(notebook.as_str().unwrap_or_default())
                            .canonicalize()
                            .map(|path| path.to_slash_lossy())
                        {
                            kernel["notebook"] = serde_json::to_value(notebook)?;
                        }
                    }
                    map.insert(id, kernel);
                }
            }
        }

        Ok(map)
    }

    /// Create a `JupyterKernel`.
    pub async fn create(id: &str, language: &str) -> Result<Kernel> {
        let mut kernel = JupyterKernel::find(language).await?;
        kernel.id = id.to_string();

        Ok(Kernel::Jupyter(kernel))
    }

    /// Connect to a running kernel
    ///
    /// Gets a list of running kernels (see `running()`) and matches the `id_or_path` against
    /// the kernel's id or path.
    pub async fn connect(id_or_path: &str) -> Result<(String, Kernel)> {
        // Attempt to resolve the `id_or_path` into an `id`.
        let running = JupyterKernel::running().await?;
        let id = (|| {
            if let Ok(path) = PathBuf::from(id_or_path).canonicalize() {
                // Path of a file was passed so see if it is matched
                let path = path.to_slash_lossy();
                for (id, kernel) in running {
                    if kernel
                        .get("notebook")
                        .and_then(|nb| nb.as_str())
                        .unwrap_or_default()
                        == path
                    {
                        return Ok(id);
                    }
                }
                bail!("Unable to find running kernel for notebook file `{}`. Perhaps you need to start one?", path)
            } else {
                // Assume that an id was passed; check that it is running
                for id in running.keys() {
                    if id_or_path == id {
                        return Ok(id.to_string());
                    }
                }
                bail!(
                    "Unable to find a running kernel with an id matching `{}`",
                    id_or_path
                )
            };
        })()?;

        // Use the id to read the connection file
        let connection = JupyterConnection::read_file(&id)?;

        // Create a new kernel instance with the connection and initialize it
        let mut kernel = Self::default();
        kernel.initialize(connection, None, None).await?;

        Ok((id, Kernel::Jupyter(kernel)))
    }

    /// Find a `JupyterKernel` for the given language.
    ///
    /// Searches for an installed kernel with support for the language.
    /// Is optimized to avoid unnecessary disk reads.
    pub async fn find(language: &str) -> Result<JupyterKernel> {
        let specs = KERNEL_SPECS.read().await;

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
            if !kernels.exists() {
                continue;
            }

            // Is there is a kernelspec with the same name?
            let path = kernels.join(language).join("kernel.json");
            if path.exists() {
                let kernel = JupyterKernel::read(language, &path).await?;
                if kernel.language(Some(language.to_string())).is_ok() {
                    return Ok(kernel);
                }
            }

            // Is there is a kernelspec that supports the language?
            for dir in kernels.read_dir()?.flatten() {
                let path = dir.path().join("kernel.json");
                if path.exists() {
                    let name = dir.file_name().to_string_lossy().to_string();
                    let kernel = JupyterKernel::read(&name, &path).await?;
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

    /// Initialize the kernel
    ///
    /// - Establishes the necessary socket connections and monitoring tasks for the kernel.
    /// - Gets the kernel info (e.g. language, which if not started by Stencila will otherwise be unavailable)
    /// - Runs any startup code for the language needed to interact with the kernel from Stencila
    async fn initialize(
        &mut self,
        connection: JupyterConnection,
        pid: Option<u32>,
        run_task: Option<JoinHandle<()>>,
    ) -> Result<()> {
        let ctx = zmq::Context::new();

        // Generate HMAC template
        let hmac =
            HmacSha256::new_from_slice(connection.key.as_bytes()).expect("Unable to generate HMAC");

        // Create the shell socket
        let shell_socket = ctx.socket(zmq::REQ)?;
        shell_socket.connect(&connection.shell_url())?;

        // Create the channel that IOPub messages get sent on
        let (iopub_sender, mut iopub_receiver) = mpsc::channel(100);

        // Spawn a task to listen to IOPub messages from the kernel and publish
        // them on a Rust channel so that `exec()` and other methods can listen for
        // them.
        let id = self.id.clone();
        let status = self.status.clone();
        let url = connection.iopub_url();
        let hmac_clone = hmac.clone();
        let subscribe_task = tokio::spawn(async move {
            let socket = ctx.socket(zmq::SUB).expect("Unable to create IOPub socket");

            let result = socket
                .connect(&url)
                .and_then(|_| socket.set_subscribe("".as_bytes()));
            if let Err(error) = result {
                tracing::error!(
                    "When connecting or subscribing to IOPub socket for kernel `{}`: {}",
                    id,
                    error
                );
                *(status.write().await) = KernelStatus::Unresponsive;
                return;
            }

            // Send an initial "fake" message to signal that this thread is ready to start receiving
            let init_message =
                JupyterMessage::new(JupyterMessageType::stream, json!({"name": "<init>"}));
            if let Err(error) = iopub_sender.send(init_message).await {
                tracing::error!(
                    "Unable to on-send IOPub init message for kernel `{}`: {}",
                    id,
                    error
                )
            }

            loop {
                let result = JupyterMessage::receive(&hmac_clone.clone(), &socket);
                match result {
                    Ok(message) => {
                        let msg_type = message.header.msg_type.clone();
                        if matches!(msg_type, JupyterMessageType::error) {
                            tracing::debug!(
                                "IOPub error message from kernel `{}`: {:?}",
                                id,
                                message.content
                            )
                        }
                        if let Err(error) = iopub_sender.send(message).await {
                            tracing::error!(
                                "Unable to on-send IOPub message for kernel `{}`: {}",
                                id,
                                error
                            )
                        } else {
                            tracing::debug!(
                                "On-sent IOPub message from kernel `{}`: {:?}",
                                id,
                                msg_type
                            )
                        }
                    }
                    Err(error) => tracing::error!(
                        "When receiving on IOPub socket for kernel `{}`: {}",
                        id,
                        error
                    ),
                }
            }
        });

        // Spawn a task to monitor the kernel
        let id = self.id.clone();
        let status = self.status.clone();
        let url = connection.heartbeat_url();
        let monitor_task = tokio::spawn(async move {
            let ctx = zmq::Context::new();
            let socket = ctx
                .socket(zmq::REQ)
                .expect("Unable to create heartbeat socket");

            let result = socket.connect(&url);
            if let Err(error) = result {
                tracing::error!(
                    "When connecting to heartbeat socket for kernel `{}`: {}",
                    id,
                    error
                );
                *(status.write().await) = KernelStatus::Unresponsive;
                return;
            }

            loop {
                let result = socket.send("", 0).and_then(|_| socket.recv_msg(0));
                if let Err(error) = result {
                    tracing::error!("When checking for heartbeat for kernel `{}`: {}", id, error);
                    *(status.write().await) = KernelStatus::Unresponsive;
                    return;
                } else {
                    tracing::debug!("Got heartbeat reply from kernel `{}`", id)
                }
                sleep(Duration::from_secs(30)).await;
            }
        });

        // Wait for IOPub init message from the `subscribe_task`. This needs to be done before any `execute_request`
        // messages are sent to ensure that we are already listening for results.
        while let Some(message) = iopub_receiver.recv().await {
            if matches!(message.header.msg_type, JupyterMessageType::stream)
                && message
                    .content
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    == "<init>"
            {
                tracing::debug!("Got IOPub init message for kernel `{}`", self.id);
                break;
            }
        }

        // Get the kernel info. Apart from getting the info this seems to be necessary before
        // sending an `execute_request` to give time for the kernel to "get started" (and confirm
        // that it has).
        let request = JupyterMessage::kernel_info_request();
        request.send(&self.session, &hmac, &shell_socket)?;
        let reply = JupyterMessage::receive(&hmac, &shell_socket)?;
        tracing::debug!("Got kernel info for kernel `{}`: {:#?}", self.id, reply);
        let kernel_info: JupyterKernelInfoReply = reply.content()?;

        // Set the language if its empty (e.g. connected to an already running kernel)
        if self.language.is_empty() {
            if let Some(language) = kernel_info
                .language_info
                .get("name")
                .and_then(|name| name.as_str())
            {
                self.language = language.to_string()
            }
        }

        // Despite the above checks, for some kernels (e.g Python and Javascript), it seems
        // necessary to wait for a little before making an execution request to avoid it
        // hanging waiting for IOPub messages
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(100)).await;

        // Update status
        *(self.status.write().await) = KernelStatus::Idle;

        // Store details
        self.connection = Some(connection);
        self.details = Some(JupyterDetails {
            hmac,
            pid,
            shell_socket: Arc::new(Mutex::new(shell_socket)),
            iopub_receiver,
            run_task,
            subscribe_task,
            monitor_task,
        });

        // Run any startup code
        let language = self.language(None)?;
        if let Some(code) = startup(&language)? {
            self.exec(&code).await?;
        }

        Ok(())
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
    async fn read(name: &str, path: &Path) -> Result<JupyterKernel> {
        let json = fs::read_to_string(path)?;
        let mut kernel: JupyterKernel = serde_json::from_str(&json)?;
        kernel.name = name.to_string();
        kernel.path = path.to_path_buf();

        let mut specs = KERNEL_SPECS.write().await;
        specs.insert(name.to_string(), kernel.clone());

        Ok(kernel)
    }
}

#[async_trait]
impl KernelTrait for JupyterKernel {
    fn language(&self, language: Option<String>) -> Result<String> {
        let canonical = Ok(self.language.to_lowercase());
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

    async fn start(&mut self) -> Result<()> {
        let connection = JupyterConnection::new(&self.id);
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
        let pid = child.id().expect("Unable to get child process id");

        // Spawn a task to wait on the kernel process and update status
        // when it exits.
        let id = self.id.clone();
        let status = self.status.clone();
        let run_task = tokio::spawn(async move {
            let output = child
                .wait_with_output()
                .await
                .expect("Kernel could not be executed");

            if output.status.success() {
                tracing::debug!("Kernel `{}` exited successfully", id);
                *(status.write().await) = KernelStatus::Finished;
            } else {
                tracing::error!(
                    "Kernel `{}` had non-zero exit status: {}",
                    id,
                    output.status
                );
                *(status.write().await) = KernelStatus::Failed;
            }

            if !output.stderr.is_empty() {
                tracing::error!(
                    "Kernel `{}` had error message: {}",
                    id,
                    &String::from_utf8_lossy(&output.stderr)
                )
            }
        });

        // Initialize the connection
        self.initialize(connection, Some(pid), Some(run_task)).await
    }

    async fn stop(&mut self) -> Result<()> {
        let language = self.language(None)?;
        if let Some(code) = shutdown(&language)? {
            self.exec(&code).await?;
        }

        if let Some(JupyterDetails {
            run_task,
            subscribe_task,
            monitor_task,
            ..
        }) = &self.details
        {
            subscribe_task.abort();
            monitor_task.abort();
            if let Some(run_task) = run_task {
                run_task.abort()
            }
        }

        if let Some(connection) = &self.connection {
            if let Err(error) = connection.remove_file() {
                tracing::warn!("While deleting Jupyter kernel connection file: {}", error)
            };
        }

        Ok(())
    }

    async fn status(&self) -> KernelStatus {
        self.status.read().await.clone()
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        let language = self.language(None)?;
        if let Some(code) = get(&language, name)? {
            let json = self.exec(&code).await?;
            if let Some(Node::String(json)) = json.first() {
                let node = serde_json::from_str(json)?;
                Ok(node)
            } else {
                bail!("While getting symbol from Jupyter kernel did not get JSON string")
            }
        } else {
            bail!(
                "Getting a symbol from a `{}` language kernel is not currently supported",
                language
            )
        }
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let language = self.language(None)?;
        let json = serde_json::to_string(&value)?;
        if let Some(code) = set(&language, name, &json)? {
            self.exec(&code).await?;
            Ok(())
        } else {
            bail!(
                "Setting a symbol in a `{}` language kernel is not currently supported",
                language
            )
        }
    }

    async fn exec(&mut self, code: &str) -> Result<Vec<Node>> {
        let JupyterDetails {
            hmac,
            shell_socket,
            iopub_receiver,
            ..
        } = self.details.as_mut().expect("Should be started");

        let socket = shell_socket.lock().await;

        let request = JupyterMessage::execute_request(code);
        tracing::debug!("Sending request: {:#?}", request);
        request.send(&self.session, hmac, &socket)?;

        let mut outputs: Vec<Node> = Vec::new();
        let stdout = "".to_string();
        let stderr = "".to_string();

        // TODO: timeout on recv()?
        while let Some(message) = iopub_receiver.recv().await {
            if let Some(parent_header) = &message.parent_header {
                if parent_header.msg_id == request.header.msg_id {
                    tracing::debug!(
                        "Handling IOPub message {:?}: {:#?}",
                        message.header.msg_type,
                        message.content
                    );
                    match message.header.msg_type {
                        // Some kernels use `execute_result`, others `display_data`, even
                        // for simple, text outputs
                        JupyterMessageType::execute_result | JupyterMessageType::display_data => {
                            // TODO decode output properly, this just gets the plain
                            // text representation.
                            let output = message
                                .content
                                .get("data")
                                .and_then(|value| value.get("text/plain"))
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_string();
                            let output = Node::String(output);
                            outputs.push(output);
                        }
                        JupyterMessageType::stream => {
                            // TODO accumulate stdout and stderr
                        }
                        JupyterMessageType::error => {
                            let error = translate_error(&message.content, &self.language);
                            tracing::error!("{}", error.error_message)
                        }
                        JupyterMessageType::status => {
                            let status: JupyterStatus = message.content()?;
                            match status.execution_state.as_str() {
                                "starting" => {
                                    *(self.status.write().await) = KernelStatus::Starting;
                                }
                                "busy" => {
                                    *(self.status.write().await) = KernelStatus::Busy;
                                }
                                "idle" => {
                                    *(self.status.write().await) = KernelStatus::Idle;
                                    tracing::debug!("Received idle status");
                                    break;
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                } else {
                    tracing::debug!(
                        "Ignoring IOPub message because {:?} != {:#?}",
                        parent_header.msg_id,
                        request.header.msg_id
                    );
                }
            }
        }
        if !stdout.is_empty() {
            outputs.push(Node::String(stdout))
        }
        if !stderr.is_empty() {
            // TODO: add to errors
            outputs.push(Node::String(stderr))
        }

        let response = JupyterMessage::receive(hmac, &socket)?;
        tracing::debug!("Received response {:#?}", response);
        // TODO deal with response.content.status == 'error' and "aborted"

        Ok(outputs)
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
#[derive(Debug, Clone, Defaults, JsonSchema, Deserialize, Serialize)]
#[schemars(deny_unknown_fields)]
#[serde(default)]
struct JupyterConnection {
    /// The path to the connection file
    #[serde(skip_deserializing)]
    path: PathBuf,

    /// The transport protocol to use for ZeroMQ
    #[def = "\"tcp\".to_string()"]
    transport: String,

    /// The IP address of the kernel
    #[def = "\"127.0.0.1\".to_string()"]
    ip: String,

    /// The message signature scheme
    #[def = "\"hmac-sha256\".to_string()"]
    signature_scheme: String,

    /// The HMAC key
    key: String,

    /// The control port
    #[def = "JupyterConnection::pick_port()"]
    control_port: u16,

    /// The shell port
    #[def = "JupyterConnection::pick_port()"]
    shell_port: u16,

    /// The stdin port
    #[def = "JupyterConnection::pick_port()"]
    stdin_port: u16,

    /// The heartbeat port
    #[def = "JupyterConnection::pick_port()"]
    hb_port: u16,

    /// The iopub port
    #[def = "JupyterConnection::pick_port()"]
    iopub_port: u16,
}

type HmacSha256 = Hmac<Sha256>;

impl JupyterConnection {
    /// Create a new connection
    ///
    /// # Arguments
    ///
    /// `id`: The id of the kernel
    fn new(id: &str) -> Self {
        let name = format!("stencila-{}.json", id);
        let path = JupyterKernel::runtime_dir().join(name);
        let key = keys::generate();

        JupyterConnection {
            path,
            key,
            ..Default::default()
        }
    }

    /// Pick a port to use for one of the connection sockets
    fn pick_port() -> u16 {
        portpicker::pick_unused_port().expect("There are no free ports")
    }

    /// Read a connection file from disk
    fn read_file(id: &str) -> Result<Self> {
        let path = JupyterKernel::data_dir()
            .join("runtime")
            .join(format!("kernel-{}.json", id));
        let file = File::open(&path)?;
        let mut connection: Self = serde_json::from_reader(file)?;
        connection.path = path;
        Ok(connection)
    }

    /// Write the connection file to disk
    ///
    /// The file is created with permissions that only allow the current user to read the file.
    /// On Mac and Linux using mode `600` and on Windows using share mode `0`.
    fn write_file(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }

        let mut options = OpenOptions::new();
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(any(target_os = "windows"))]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }

        // Using `create_new` is the safest way to create the file to
        // avoid a time-of-check to time-of-use race condition / attack
        let mut file = options
            .read(true)
            .write(true)
            .create_new(true)
            .open(&self.path)?;

        let json = serde_json::to_string_pretty(&self)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    /// Remove the connection file from disk
    fn remove_file(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?
        }
        Ok(())
    }

    /// Get the base URI for the connection
    fn base_url(&self) -> String {
        format!("{}://{}:", transport = self.transport, ip = self.ip)
    }

    /// Get the URL of the control channel
    fn _control_url(&self) -> String {
        [self.base_url(), self.control_port.to_string()].concat()
    }

    /// Get the URL of the shell channel
    fn shell_url(&self) -> String {
        [self.base_url(), self.shell_port.to_string()].concat()
    }

    /// Get the URL of the iopub channel
    fn iopub_url(&self) -> String {
        [self.base_url(), self.iopub_port.to_string()].concat()
    }

    /// Get the URL of the heartbeat channel
    fn heartbeat_url(&self) -> String {
        [self.base_url(), self.hb_port.to_string()].concat()
    }
}

/// The type of a Jupyter message
///
/// This list is from https://jupyter-client.readthedocs.io/en/stable/messaging.html.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
enum JupyterMessageType {
    // Messages on the shell (ROUTER/DEALER) channel
    execute_request,
    execute_reply,
    inspect_request,
    inspect_reply,
    complete_request,
    complete_reply,
    history_request,
    history_reply,
    is_complete_request,
    is_complete_reply,
    connect_request,
    connect_reply,
    comm_info_request,
    comm_info_reply,
    kernel_info_request,
    kernel_info_reply,
    // Messages on the Control (ROUTER/DEALER) channel
    shutdown_request,
    shutdown_reply,
    interrupt_request,
    interrupt_reply,
    debug_request,
    debug_reply,
    // Messages on the IOPub (PUB/SUB) channel
    stream,
    display_data,
    update_display_data,
    execute_input,
    execute_result,
    error,
    status,
    clear_output,
    debug_event,
    // Messages on the stdin (ROUTER/DEALER) channel
    input_request,
    input_reply,
}

/// The header of a Jupyter message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#message-header.
/// Note that communication with some kernels may fail if one of more of these fields
/// is missing.
#[derive(Debug, Clone, Defaults, Deserialize, Serialize)]
#[serde(default)]
struct JupyterMessageHeader {
    /// The version of the message protocol
    #[def = "\"5.3\".to_string()"]
    version: String,

    /// The type of message
    #[def = "JupyterMessageType::execute_request"]
    msg_type: JupyterMessageType,

    /// A unique identifier for the message
    #[def = "uuids::generate(uuids::Family::Generic)"]
    msg_id: String,

    /// A unique identifier for the kernel session
    session: String,

    /// The name of the user
    ///
    /// We currently leave this blank but it is required by some kernels (e.g. `IJulia`)
    username: String,

    /// ISO 8601 timestamp for when the message was created
    #[def = "chrono::Utc::now().to_rfc3339()"]
    date: String,
}

impl JupyterMessageHeader {
    /// Create a new message header
    fn new(msg_type: JupyterMessageType) -> Self {
        JupyterMessageHeader {
            msg_type,
            ..Default::default()
        }
    }
}

// Each message type has its own structure to the message `content`.
// The following content type definitions implement some of those structures
// on an as needed bases to reduce the need to use lots of `get("...")` calls
// on `serde_json::Value` (the default content type). Note that,
// for both convenience and robustness, `serde_json::Value` is still used for
// some fields in these structs.
//
// Those definitions, including comments, are taken from
// https://jupyter-client.readthedocs.io/en/stable/messaging.html.

/// Content of a `kernel_info_reply` message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#kernel-info
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
struct JupyterKernelInfoReply {
    /// 'ok' if the request succeeded or 'error', with error information as in all other replies.
    status: String,

    /// Version of messaging protocol.
    /// The first integer indicates major version.  It is incremented when
    /// there is any backward incompatible change.
    /// The second integer indicates minor version.  It is incremented when
    /// there is any backward compatible change.
    protocol_version: String,

    /// The kernel implementation name
    /// (e.g. 'ipython' for the IPython kernel)
    implementation: String,

    /// Implementation version number.
    /// The version number of the kernel's implementation
    /// (e.g. IPython.__version__ for the IPython kernel)
    implementation_version: String,

    /// Information about the language of code for the kernel
    language_info: serde_json::Value,

    /// A banner of information about the kernel,
    /// which may be displayed in console environments.
    banner: String,

    /// A boolean flag which tells if the kernel supports debugging in the notebook.
    /// Default is False
    debugger: bool,

    /// Optional: A list of dictionaries, each with keys 'text' and 'url'.
    /// These will be displayed in the help menu in the notebook UI.
    help_links: serde_json::Value,
}

/// Content of an `execute_request` message
#[derive(Debug, Defaults, Serialize)]
#[serde(default)]
struct JupyterExecuteRequest {
    // Source code to be executed by the kernel, one or more lines.
    code: String,

    // A boolean flag which, if True, signals the kernel to execute
    // this code as quietly as possible.
    // silent=True forces store_history to be False,
    // and will *not*:
    //   - broadcast output on the IOPUB channel
    //   - have an execute_result
    // The default is False.
    #[def = "false"]
    silent: bool,

    // A boolean flag which, if True, signals the kernel to populate history
    // The default is True if silent is False.  If silent is True, store_history
    // is forced to be False.
    #[def = "true"]
    store_history: bool,

    // A dict mapping names to expressions to be evaluated in the
    // user's dict. The rich display-data representation of each will be evaluated after execution.
    // See the display_data content for the structure of the representation data.
    #[def = "json!({})"]
    user_expressions: serde_json::Value,

    // Some frontends do not support stdin requests.
    // If this is true, code running in the kernel can prompt the user for input
    // with an input_request message (see below). If it is false, the kernel
    // should not send these messages.
    #[def = "false"]
    allow_stdin: bool,

    // A boolean flag, which, if True, aborts the execution queue if an exception is encountered.
    // If False, queued execute_requests will execute even if this request generates an exception.
    #[def = "false"]
    stop_on_error: bool,
}

/// Content of a `display_data` message
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
struct JupyterDisplayData {
    /// The data dict contains key/value pairs, where the keys are MIME
    /// types and the values are the raw data of the representation in that
    /// format.
    data: HashMap<String, serde_json::Value>,

    /// Any metadata that describes the data
    metadata: HashMap<String, serde_json::Value>,

    /// Optional transient data introduced in 5.1. Information not to be
    /// persisted to a notebook or other documents. Intended to live only
    /// during a live kernel session.
    transient: HashMap<String, serde_json::Value>,
}

/// Content of an `execute_result` message
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
struct JupyterExecuteResult {
    // The counter for this execution is also provided so that clients can
    // display it, since IPython automatically creates variables called _N
    // (for prompt N).
    execution_count: u32,

    // `data` and `metadata` are identical to a display_data message.
    // the object being displayed is that passed to the display hook,
    // i.e. the *result* of the execution.
    data: HashMap<String, serde_json::Value>,
    metadata: HashMap<String, serde_json::Value>,
}

/// Content of a `status` message
#[derive(Debug, Defaults, Deserialize)]
#[serde(default)]
struct JupyterStatus {
    /// When the kernel starts to handle a message, it will enter the 'busy'
    /// state and when it finishes, it will enter the 'idle' state.
    /// The kernel will publish state 'starting' exactly once at process startup.
    execution_state: String,
}

/// A Jupyter message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#general-message-format.
/// Some of the below documentation is copied from there.
#[derive(Debug, Deserialize, Serialize)]
struct JupyterMessage {
    /// ZeroMQ socket identities
    identities: Vec<String>,

    /// The message header
    ///
    /// "The message header contains information about the message, such as unique identifiers
    /// for the originating session and the actual message id, the type of message, the version
    /// of the Jupyter protocol, and the date the message was created."
    header: JupyterMessageHeader,

    /// The header of the parent message
    ///
    /// "When a message is the “result” of another message, such as a side-effect (output or status)
    /// or direct reply, the `parent_header` is a copy of the `header` of the message that “caused”
    /// the current message. `_reply` messages MUST have a `parent_header`, and side-effects typically
    /// have a parent. If there is no parent, an empty dict should be used. This parent is used by
    /// clients to route message handling to the right place, such as outputs to a cell.""
    parent_header: Option<JupyterMessageHeader>,

    /// Metadata about the message
    ///
    /// "The metadata dict contains information about the message that is not part of the content.
    /// This is not often used, but can be an extra location to store information about requests and
    /// replies, such as extensions adding information about request or execution context.""
    metadata: serde_json::Value,

    /// The content of the message
    ///
    /// "The content dict is the body of the message. Its structure is dictated by the `msg_type`
    /// field in the header, described in detail for each message below."
    content: serde_json::Value,
}

const DELIMITER: &[u8] = b"<IDS|MSG>";

/// A Jupyter message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#the-wire-protocol
impl JupyterMessage {
    /// Create a new message
    fn new<Content: Serialize>(msg_type: JupyterMessageType, content: Content) -> Self {
        Self {
            identities: Vec::new(),
            header: JupyterMessageHeader::new(msg_type),
            parent_header: None,
            metadata: json!({}),
            content: serde_json::to_value(content).expect("Unable to serialize to a value"),
        }
    }

    /// Create an `kernel_info_request` message
    ///
    /// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#kernel-info
    fn kernel_info_request() -> Self {
        Self::new(JupyterMessageType::kernel_info_request, json!({}))
    }

    /// Create an `execute_request` message
    fn execute_request(code: &str) -> Self {
        Self::new(
            JupyterMessageType::execute_request,
            JupyterExecuteRequest {
                code: code.to_string(),
                ..Default::default()
            },
        )
    }

    /// Send the message
    fn send(&self, session: &str, hmac: &HmacSha256, socket: &Socket) -> Result<()> {
        let mut parts: Vec<&[u8]> = Vec::with_capacity(7);

        for part in &self.identities {
            parts.push(part.as_bytes());
        }

        parts.push(DELIMITER);

        let mut header = self.header.clone();
        header.session = session.to_string();
        let header = serde_json::to_string(&header)?;
        let header = header.as_bytes();

        // "If there is no parent, an empty dict should be used"
        let parent_header = match &self.parent_header {
            Some(header) => serde_json::to_string(header)?,
            None => "{}".to_string(),
        };
        let parent_header = parent_header.as_bytes();

        let metadata = serde_json::to_string(&self.metadata)?;
        let metadata = metadata.as_bytes();

        let content = serde_json::to_string(&self.content)?;
        let content = content.as_bytes();

        use hmac::Mac;
        let mut hmac = hmac.clone();
        hmac.update(header);
        hmac.update(parent_header);
        hmac.update(metadata);
        hmac.update(content);
        let output = hmac.finalize();
        let hmac = hex::encode(output.into_bytes().as_slice());
        parts.push(hmac.as_bytes());

        parts.push(header);
        parts.push(parent_header);
        parts.push(metadata);
        parts.push(content);

        socket.send_multipart(&parts, 0)?;

        Ok(())
    }

    /// Receive a message
    fn receive(hmac: &HmacSha256, socket: &Socket) -> Result<Self> {
        let parts = socket.recv_multipart(0)?;

        let delimiter = parts
            .iter()
            .position(|part| &part[..] == DELIMITER)
            .ok_or_else(|| eyre!("Message is missing delimiter"))?;

        let identities = parts[..delimiter]
            .iter()
            .map(|identity| String::from_utf8_lossy(identity).to_string())
            .collect();

        if parts.len() < delimiter + 5 {
            bail!("Message does not have enough parts")
        }
        let msg_hmac = &parts[delimiter + 1];
        let header = &parts[delimiter + 2];
        let parent_header = &parts[delimiter + 3];
        let metadata = &parts[delimiter + 4];
        let content = &parts[delimiter + 5];

        use hmac::Mac;
        let mut hmac = hmac.clone();
        hmac.update(header);
        hmac.update(parent_header);
        hmac.update(metadata);
        hmac.update(content);
        if let Err(error) = hmac.verify(&hex::decode(&msg_hmac)?) {
            bail!("Unable to verify message HMAC: {}", error);
        }

        let header = serde_json::from_slice(header)?;
        let parent_header = serde_json::from_slice(parent_header)?;
        let metadata = serde_json::from_slice(metadata)?;
        let content = serde_json::from_slice(content)?;

        Ok(Self {
            identities,
            header,
            parent_header,
            metadata,
            content,
        })
    }

    /// Get the content of a message as a particular type
    fn content<Content: DeserializeOwned>(self) -> Result<Content> {
        let content = serde_json::from_value(self.content)?;
        Ok(content)
    }
}

/// Language specific code to be run at kernel startup
fn startup(language: &str) -> Result<Option<String>> {
    Ok(match language {
        "python" => Some("import json".to_string()),
        _ => None,
    })
}

/// Language specific code to be run at kernel shutdown
fn shutdown(_language: &str) -> Result<Option<String>> {
    Ok(None)
}

/// Language specific code for getting a variable
fn get(language: &str, name: &str) -> Result<Option<String>> {
    Ok(match language {
        "javascript" => Some(format!("JSON.stringify({})", name)),
        "python" => Some(format!("json.dumps({})", name)),
        "r" => Some(format!("jsonlite::toJSON({})", name)),
        _ => None,
    })
}

/// Language specific code for setting a variable
fn set(language: &str, name: &str, json: &str) -> Result<Option<String>> {
    Ok(match language {
        "javascript" => Some(format!(
            "let {} = JSON.parse(\"{}\")",
            name,
            json.replace("\"", "\\\"")
        )),
        "python" => Some(format!(
            "{} = json.loads(\"{}\")",
            name,
            json.replace("\"", "\\\"")
        )),
        "r" => Some(format!(
            "{} = jsonlite::fromJSON(\"{}\")",
            name,
            json.replace("\"", "\\\"")
        )),
        _ => None,
    })
}
