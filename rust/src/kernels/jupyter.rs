use super::{Kernel, KernelStatus, KernelTrait};
use crate::{
    errors::incompatible_language,
    utils::{keys, uuids},
};
use async_trait::async_trait;
use defaults::Defaults;
use derivative::Derivative;
use eyre::{bail, eyre, Result};
use hmac::{Hmac, NewMac};
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use sha2::Sha256;
use std::{
    collections::HashMap,
    env, fs,
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
    pid: u32,

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
    run_task: JoinHandle<()>,

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
    pub async fn list() -> Result<Vec<String>> {
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

    /// Create a `JupyterKernel` variant.
    pub async fn create(id: &str, language: &str) -> Result<Kernel> {
        let mut kernel = JupyterKernel::find(language).await?;
        kernel.id = id.to_string();

        Ok(Kernel::Jupyter(kernel))
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

        let hmac =
            HmacSha256::new_from_slice(connection.key.as_bytes()).expect("Unable to generate HMAC");

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

        let ctx = zmq::Context::new();

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
                    "Unable to send IOPub init message for kernel `{}`: {}",
                    id,
                    error
                )
            }

            loop {
                let result = JupyterMessage::receive(&hmac_clone.clone(), &socket);
                match result {
                    Ok(message) => {
                        let msg_type = message.header.msg_type.clone();
                        if let Err(error) = iopub_sender.send(message).await {
                            tracing::error!(
                                "Unable to send IOPub message for kernel `{}`: {}",
                                id,
                                error
                            )
                        } else {
                            tracing::debug!(
                                "Sent IOPub message from kernel `{}`: {:?}",
                                id,
                                msg_type
                            );
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
            use tokio::time::{sleep, Duration};

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
        let response = JupyterMessage::receive(&hmac, &shell_socket)?;
        tracing::debug!(
            "Got kernel info for kernel `{}`: {:#?}",
            self.id,
            response.content
        );

        // Despite the above checks, for some kernels (e.g Python and Javascript), it seems
        // necessary to wait for a little before making an execution request to avoid it
        // hanging waiting for IOPub messages
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_millis(100)).await;

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

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(JupyterDetails {
            run_task,
            subscribe_task,
            monitor_task,
            ..
        }) = &self.details
        {
            run_task.abort();
            subscribe_task.abort();
            monitor_task.abort();
        }
        Ok(())
    }

    async fn status(&self) -> KernelStatus {
        self.status.read().await.clone()
    }

    async fn get(&self, _name: &str) -> Result<Node> {
        Ok(Node::String("TODO".to_string()))
    }

    async fn set(&mut self, _name: &str, _value: Node) -> Result<()> {
        Ok(())
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
            if let Some(parent_header) = message.parent_header {
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
                            // TODO decode output
                            outputs.push(Node::String(message.content.to_string()));
                        }
                        JupyterMessageType::stream => {
                            // TODO accumulate stdout and stderr
                        }
                        JupyterMessageType::status => {
                            match message
                                .content
                                .get("execution_state")
                                .and_then(|value| value.as_str())
                                .unwrap_or_default()
                            {
                                "idle" => {
                                    *(self.status.write().await) = KernelStatus::Idle;
                                    tracing::debug!("Received idle status");
                                    break;
                                }
                                "busy" => {
                                    *(self.status.write().await) = KernelStatus::Busy;
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
#[derive(Debug, Clone, Defaults, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct JupyterConnection {
    /// The path to the connection file
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

    fn _control_url(&self) -> String {
        [self.base_url(), self.control_port.to_string()].concat()
    }

    fn shell_url(&self) -> String {
        [self.base_url(), self.shell_port.to_string()].concat()
    }

    fn iopub_url(&self) -> String {
        [self.base_url(), self.iopub_port.to_string()].concat()
    }

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

/// A Jupyter message
///
/// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#general-message-format.
/// Some of the below documentation is copied from there.
#[derive(Debug, Clone, Defaults, Deserialize, Serialize)]
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
    fn new(msg_type: JupyterMessageType, content: serde_json::Value) -> Self {
        Self {
            identities: Vec::new(),
            header: JupyterMessageHeader::new(msg_type),
            parent_header: None,
            metadata: json!({}),
            content,
        }
    }

    /// Create an `kernel_info_request` message
    ///
    /// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#kernel-info
    fn kernel_info_request() -> Self {
        Self::new(JupyterMessageType::kernel_info_request, json!({}))
    }

    /// Create an `execute_request` message
    ///
    /// See https://jupyter-client.readthedocs.io/en/stable/messaging.html#execute
    /// Document below copied from there.
    fn execute_request(code: &str) -> Self {
        Self::new(
            JupyterMessageType::execute_request,
            json!({
               // Source code to be executed by the kernel, one or more lines.
               "code": code,

               // A boolean flag which, if True, signals the kernel to execute
               // this code as quietly as possible.
               // silent=True forces store_history to be False,
               // and will *not*:
               //   - broadcast output on the IOPUB channel
               //   - have an execute_result
               // The default is False.
               "silent" : false,

               // A boolean flag which, if True, signals the kernel to populate history
               // The default is True if silent is False.  If silent is True, store_history
               // is forced to be False.
               "store_history" : false,

               // A dict mapping names to expressions to be evaluated in the
               // user's dict. The rich display-data representation of each will be evaluated after execution.
               // See the display_data content for the structure of the representation data.
               "user_expressions" : json!({}),

               // Some frontends do not support stdin requests.
               // If this is true, code running in the kernel can prompt the user for input
               // with an input_request message (see below). If it is false, the kernel
               // should not send these messages.
               "allow_stdin" : true,

               // A boolean flag, which, if True, aborts the execution queue if an exception is encountered.
               // If False, queued execute_requests will execute even if this request generates an exception.
               "stop_on_error" : true,
            }),
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
}
