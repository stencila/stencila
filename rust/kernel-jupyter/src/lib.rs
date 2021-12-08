use codec_ipynb::{translate_error, translate_mime_bundle, translate_stderr};
use defaults::Defaults;
use derivative::Derivative;
use kernel::{
    async_trait::async_trait,
    eyre::{bail, eyre, Result},
    serde::{Deserialize, Serialize},
    stencila_schema::{CodeError, Node},
    Kernel, KernelStatus, KernelTrait, KernelType,
};
use once_cell::sync::Lazy;
use path_slash::PathBufExt;
use serde_json::json;
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    process::Command,
    sync::{broadcast, Mutex, RwLock},
    task::JoinHandle,
    time::{sleep, timeout},
};
use zmq::Socket;

mod connection;
pub mod dirs;
mod messages;
mod server;

use crate::{
    connection::JupyterConnection,
    dirs::kernel_dirs,
    messages::{
        HmacSha256, JupyterDisplayData, JupyterExecuteResult, JupyterKernelInfoReply,
        JupyterMessage, JupyterMessageType, JupyterStatus, JupyterStream,
    },
};

pub use server::JupyterServer;

// A UUID for kernels
uuids::uuid_family!(JupyterKernelId, "ke");

// A UUID for sessions
uuids::uuid_family!(JupyterSessionId, "se");

/// A kernel that delegates to a Jupyter kernel
///
/// Most of the fields of this `struct` reflect those in a "kernel spec" and are read from a `kernel.json` file.
/// See https://jupyter-client.readthedocs.io/en/stable/kernels.html#kernel-specs.
/// Comments below are copied from there.
#[skip_serializing_none]
#[derive(Debug, Defaults, Deserialize, Serialize)]
#[serde(default)]
pub struct JupyterKernel {
    /// The id of the kernel instance
    id: JupyterKernelId,

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
    session: JupyterSessionId,

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
    #[allow(dead_code)]
    pid: Option<u32>,

    /// The HMAC used when signing messages
    ///
    /// Derived from the connection's `key`.
    hmac: HmacSha256,

    /// The socket to send Jupyter "shell" commands to
    #[derivative(Debug = "ignore")]
    shell_socket: Arc<Mutex<Socket>>,

    /// The sender for IOPub messages
    iopub_sender: broadcast::Sender<JupyterMessage>,

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
    /// Create a new `JupyterKernel`.
    pub async fn new(selector: &str) -> Result<JupyterKernel> {
        let mut kernel = JupyterKernel::find(selector).await?;
        kernel.id = JupyterKernelId::new();

        Ok(kernel)
    }

    /// Does the kernel support execution of a specific language?
    pub fn supports(&self, language: &str) -> bool {
        self.language == language
    }

    /// Get a list of Jupyter kernels available in the current environment
    pub async fn available() -> Result<Vec<Kernel>> {
        let mut list = Vec::new();

        for kernels in kernel_dirs() {
            if !kernels.exists() {
                continue;
            }

            for dir in kernels.read_dir()?.flatten() {
                let path = dir.path().join("kernel.json");
                if path.exists() {
                    let name = dir.file_name().to_string_lossy().to_string();
                    let kernel = JupyterKernel::read(&name, &path).await?;
                    list.push(kernel.spec())
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

    /// Connect to a running kernel
    ///
    /// Gets a list of running kernels (see `running()`) and matches the `id_or_path` against
    /// the kernel's id or path.
    pub async fn connect(id_or_path: &str) -> Result<(String, JupyterKernel)> {
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

        Ok((id, kernel))
    }

    /// Find a `JupyterKernel` for the given selector (name or language).
    ///
    /// Searches for an installed kernel with a matching name an/or support for the language.
    /// Is optimized to avoid unnecessary disk reads.
    pub async fn find(selector: &str) -> Result<JupyterKernel> {
        let specs = KERNEL_SPECS.read().await;

        // Is there is a kernelspec already read with the same name?
        if let Some(kernel) = specs.get(selector) {
            return Ok(kernel.clone());
        }

        // Is there is a kernelspec already read that supports the language?
        for kernel in specs.values() {
            if kernel.supports(selector) {
                return Ok(kernel.clone());
            }
        }

        drop(specs);

        // For each Jupyter data directory..
        for kernel in kernel_dirs() {
            if !kernel.exists() {
                continue;
            }

            // Is there is a kernelspec with a matching name?
            let path = kernel.join(selector).join("kernel.json");
            if path.exists() {
                let kernel = JupyterKernel::read(selector, &path).await?;
                return Ok(kernel);
            }

            // Is there is a kernelspec that supports the language?
            for dir in kernel.read_dir()?.flatten() {
                let path = dir.path().join("kernel.json");
                if path.exists() {
                    let name = dir.file_name().to_string_lossy().to_string();
                    let kernel = JupyterKernel::read(&name, &path).await?;
                    if kernel.supports(selector) {
                        return Ok(kernel);
                    }
                }
            }
        }

        bail!(
            "Unable to find a Jupyter kernel for language `{}`; perhaps you need to install one?",
            selector
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
        // Generate HMAC template
        let hmac = connection.hmac()?;

        // Create the shell socket
        let ctx = zmq::Context::new();
        let shell_socket = ctx.socket(zmq::REQ)?;
        shell_socket.connect(&connection.shell_url())?;

        // Create the channel that IOPub messages get sent on
        let (iopub_sender, mut iopub_receiver) = broadcast::channel(256);

        // Spawn a task to listen to IOPub messages from the kernel and publish
        // them on a Rust channel so that `exec()` and other methods can listen for
        // them.
        let id = self.id.clone();
        let status = self.status.clone();
        let url = connection.iopub_url();
        let iopub_sender_clone = iopub_sender.clone();
        let hmac_clone = hmac.clone();
        let subscribe_task = tokio::spawn(async move {
            let ctx = zmq::Context::new();
            let socket = ctx.socket(zmq::SUB).expect("Unable to create IOPub socket");

            let result = socket
                .connect(&url)
                .and_then(|_| socket.set_subscribe("".as_bytes()));
            if let Err(error) = result {
                tracing::error!(
                    "When connecting or subscribing to IOPub socket for Jupyter kernel `{}`: {}",
                    id,
                    error
                );
                *(status.write().await) = KernelStatus::Unresponsive;
                return;
            }

            // Send an initial "fake" message to signal that this thread is ready to start receiving
            let init_message =
                JupyterMessage::new(JupyterMessageType::stream, json!({"name": "<init>"}));
            if let Err(error) = iopub_sender_clone.send(init_message) {
                tracing::error!(
                    "Unable to send IOPub init message for Jupyter kernel `{}`: {}",
                    id,
                    error
                )
            } else {
                tracing::debug!("Sent IOPub init message for Jupyter kernel `{}`", id);
            }

            loop {
                let result = JupyterMessage::receive(&hmac_clone.clone(), &socket, None);
                match result {
                    Ok(message) => {
                        let msg_type = message.header.msg_type.clone();
                        if matches!(msg_type, JupyterMessageType::error) {
                            tracing::debug!(
                                "IOPub error message from Jupyter kernel `{}`: {:?}",
                                id,
                                message.content
                            )
                        }
                        if let Err(error) = iopub_sender_clone.send(message) {
                            tracing::error!(
                                "Unable to broadcast IOPub message for Jupyter kernel `{}`: {}",
                                id,
                                error
                            )
                        } else {
                            tracing::debug!(
                                "Broadcast IOPub message from Jupyter kernel `{}`: {:?}",
                                id,
                                msg_type
                            )
                        }
                    }
                    Err(error) => tracing::error!(
                        "When receiving on IOPub socket for Jupyter kernel `{}`: {}",
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
                    tracing::debug!("Got heartbeat reply from Jupyter kernel `{}`", id)
                }
                sleep(Duration::from_secs(30)).await;
            }
        });

        // Wait for IOPub init message from the `subscribe_task`. This needs to be done before any `execute_request`
        // messages are sent to ensure that we are already listening for results.
        tracing::debug!(
            "Waiting for IOPub init message for Jupyter kernel `{}`",
            self.id
        );
        while let Ok(message) = iopub_receiver.recv().await {
            if matches!(message.header.msg_type, JupyterMessageType::stream)
                && message
                    .content
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    == "<init>"
            {
                tracing::debug!("Got IOPub init message for Jupyter kernel `{}`", self.id);
                break;
            }
        }

        // Get the kernel info. Apart from getting the info this seems to be necessary before
        // sending an `execute_request` to give time for the kernel to "get started" (and confirm
        // that it has).
        let request = JupyterMessage::kernel_info_request();
        request.send(&self.session, &hmac, &shell_socket)?;
        let reply = JupyterMessage::receive(&hmac, &shell_socket, None)?;
        tracing::debug!(
            "Got kernel info for Jupyter kernel `{}`: {:#?}",
            self.id,
            reply
        );
        let kernel_info: JupyterKernelInfoReply = reply.content();

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
        sleep(Duration::from_millis(100)).await;

        // Update status
        *(self.status.write().await) = KernelStatus::Idle;

        // Store details
        self.connection = Some(connection);
        self.details = Some(JupyterDetails {
            hmac,
            pid,
            shell_socket: Arc::new(Mutex::new(shell_socket)),
            iopub_sender,
            run_task,
            subscribe_task,
            monitor_task,
        });

        // Run any startup code
        if let Some(code) = startup(&self.language)? {
            self.exec(&code).await?;
        }

        Ok(())
    }

    async fn exec_results(
        request_id: &str,
        language: &str,
        status: Arc<RwLock<KernelStatus>>,
        mut iopub_receiver: broadcast::Receiver<JupyterMessage>,
    ) -> Result<(Vec<Node>, Vec<CodeError>)> {
        let mut outputs: Vec<Node> = Vec::new();
        let mut errors: Vec<CodeError> = Vec::new();

        let mut stdout = "".to_string();
        let mut stderr = "".to_string();

        while let Ok(message) = iopub_receiver.recv().await {
            if let Some(parent_header) = &message.parent_header {
                if parent_header.msg_id == request_id {
                    let msg_type = &message.header.msg_type;
                    tracing::debug!(
                        "Handling Jupyter IOPub message {:?}: {:#?}",
                        msg_type,
                        message.content
                    );
                    match &msg_type {
                        JupyterMessageType::execute_result | JupyterMessageType::display_data => {
                            let bundle = match msg_type {
                                JupyterMessageType::execute_result => {
                                    message.content::<JupyterExecuteResult>().data
                                }
                                JupyterMessageType::display_data => {
                                    message.content::<JupyterDisplayData>().data
                                }
                                _ => unreachable!(),
                            };
                            if let Some(output) = translate_mime_bundle(&bundle) {
                                outputs.push(output);
                            }
                            // TODO: consider removing this in favour of waiting for idle
                            break;
                        }
                        JupyterMessageType::stream => {
                            let JupyterStream { name, text } = message.content();
                            match name.as_str() {
                                "stdout" => stdout.push_str(&text),
                                "stderr" => stderr.push_str(&text),
                                _ => (),
                            }
                        }
                        JupyterMessageType::error => {
                            let error = translate_error(&message.content, language);
                            errors.push(error);
                            // TODO: consider removing this in favour of waiting for idle
                            break;
                        }
                        JupyterMessageType::status => {
                            let mut guard = status.write().await;
                            let status: JupyterStatus = message.content();
                            match status.execution_state.as_str() {
                                "starting" => {
                                    *guard = KernelStatus::Starting;
                                }
                                "busy" => {
                                    *guard = KernelStatus::Busy;
                                }
                                "idle" => {
                                    *guard = KernelStatus::Idle;
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
                        "Ignoring Jupyter IOPub message because {:?} != {:#?}",
                        parent_header.msg_id,
                        request_id
                    );
                }
            }
        }

        if !stdout.is_empty() {
            let node = Node::String(stdout);
            outputs.push(node);
        }
        if !stderr.is_empty() {
            let error = translate_stderr(&serde_json::Value::String(stderr));
            errors.push(error);
        }

        Ok((outputs, errors))
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
    fn spec(&self) -> Kernel {
        Kernel::new(&self.name, KernelType::Jupyter, &[&self.language])
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
        let pid = child.id().ok_or_else(|| eyre!("Unable to get child pid"))?;

        // Spawn a task to wait on the kernel process and update status
        // when it exits.
        let id = self.id.clone();
        let status = self.status.clone();
        let run_task = tokio::spawn(async move {
            let output = child
                .wait_with_output()
                .await
                .expect("Jupyter kernel could not be started");

            if output.status.success() {
                tracing::debug!("Jupyter  kernel `{}` exited successfully", id);
                *(status.write().await) = KernelStatus::Finished;
            } else {
                tracing::error!(
                    "Jupyter kernel `{}` had non-zero exit status: {}",
                    id,
                    output.status
                );
                *(status.write().await) = KernelStatus::Failed;
            }

            if !output.stderr.is_empty() {
                tracing::error!(
                    "Jupyter kernel `{}` had error message: {}",
                    id,
                    &String::from_utf8_lossy(&output.stderr)
                )
            }
        });

        // Initialize the connection
        self.initialize(connection, Some(pid), Some(run_task)).await
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(code) = shutdown(&self.language)? {
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

    async fn status(&self) -> Result<KernelStatus> {
        let status = self.status.read().await.clone();
        Ok(status)
    }

    async fn get(&mut self, _name: &str) -> Result<Node> {
        bail!(
            "Getting a symbol from a `{}` Jupyter kernel is not currently supported",
            self.language
        )
        /*
        TODO: Reinstate in an immutable way
        if let Some(code) = get(&self.language, name)? {
            let (outputs, _errors) = self.exec(&code).await?;
            if let Some(Node::String(json)) = outputs.first() {
                let node = serde_json::from_str(json)?;
                Ok(node)
            } else {
                bail!("While getting symbol from Jupyter kernel did not get JSON string")
            }
            // TODO: Check for any errors
        } else {
            bail!(
                "Getting a symbol from a `{}` language kernel is not currently supported",
                self.language
            )
        }
        */
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let json = serde_json::to_string(&value)?;
        if let Some(code) = set(&self.language, name, &json)? {
            let (.., _errors) = self.exec(&code).await?;
            // TODO: Check for any errors
            Ok(())
        } else {
            bail!(
                "Setting a symbol in a `{}` Jupyter kernel is not currently supported",
                self.language
            )
        }
    }

    async fn exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)> {
        let JupyterDetails {
            hmac,
            shell_socket,
            iopub_sender,
            ..
        } = self.details.as_mut().expect("Should be started already");

        let socket = shell_socket.lock().await;

        // Send the request
        let request = JupyterMessage::execute_request(code);
        tracing::debug!("Sending request: {:#?}", request);
        request.send(&self.session, hmac, &socket)?;

        // Start a background task to gather the results of the execute request (outputs and errors)
        // TODO have a channel to make the wait task and return the results it has already received
        let language = self.language.clone();
        let status = self.status.clone();
        let iopub_receiver = iopub_sender.subscribe();
        let results_task = tokio::spawn(async move {
            JupyterKernel::exec_results(&request.header.msg_id, &language, status, iopub_receiver)
                .await
        });

        // Wait for the reply
        let reply = JupyterMessage::receive(hmac, &socket, None)?;
        tracing::debug!("Received response {:#?}", reply);
        // TODO deal with response.content.status == 'error' and "aborted"

        // Wait for the outputs
        let (outputs, errors) = match timeout(Duration::from_millis(1000), results_task).await {
            Ok(joined) => joined??,
            Err(_) => {
                tracing::warn!("Timed-out waiting for results from Jupyter kernel");
                (Vec::new(), Vec::new())
            }
        };

        Ok((outputs, errors))
    }
}

/// The global store of Jupyter kernels
///
/// Note that `super::KernelSpace` holds instances of kernels for each document whereas this
/// holds instances of the kernels specs read from `kernel.json` as an optimization to avoid
/// re-reading them from disk.
static KERNEL_SPECS: Lazy<Arc<RwLock<HashMap<String, JupyterKernel>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

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
fn _get(language: &str, name: &str) -> Result<Option<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::KernelStatus;

    #[tokio::test]
    async fn status() -> Result<()> {
        let kernel = JupyterKernel::new("python").await?;

        assert_eq!(kernel.status().await?, KernelStatus::Pending);

        Ok(())
    }
}
