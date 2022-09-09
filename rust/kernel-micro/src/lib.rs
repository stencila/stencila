use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use kernel::{
    common::{
        async_trait::async_trait,
        dirs,
        eyre::{bail, eyre, Result},
        once_cell::sync::Lazy,
        regex::Regex,
        serde::Serialize,
        serde_json,
        tokio::{
            self,
            fs::File,
            io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
            process::{Child, ChildStderr, ChildStdin, ChildStdout},
            sync::{broadcast, mpsc, Mutex, MutexGuard, RwLock},
        },
        tracing,
    },
    stencila_schema::{CodeError, Node, Object},
    Kernel, KernelStatus, KernelTrait, KernelType, TagMap, Task, TaskMessages, TaskOutputs,
    TaskResult,
};

// Line end flags for the Microkernel protocol
// On Windows, Rscript (and possibly other binaries) escapes unicode on stdout and stderr
// So the _ALT flags are provided for these instances (or where it is not possible to output Unicode at all).

/// Indicates the end of kernel startup, kernel is ready to perform tasks.
const READY: &str = "\u{10ACDC}";
const READY_ALT: &str = "<U+0010ACDC>";

/// Indicates the end of a task result ("outputs" on `stderr` and "messages" on `stderr`).
const RESULT: &str = "\u{10CB40}";
const RESULT_ALT: &str = "<U+0010CB40>";

/// Indicates the end of a task, kernel is ready for next task.
const TASK: &str = "\u{10ABBA}";
const TASK_ALT: &str = "<U+0010ABBA>";

/// Indicates that the task should be run in a forked process.
/// Allow dead code because these not used on Windows
#[allow(dead_code)]
const FORK: &str = "\u{10DE70}";
#[allow(dead_code)]
const FORK_ALT: &str = "<U+0010DE70>";

/// Indicates a newline in the code
const NEWLINE: &str = "\u{10B522}";

#[derive(Debug, Serialize)]
#[serde(crate = "kernel::common::serde")]
pub struct MicroKernel {
    /// The name of the kernel
    ///
    /// The convention for Microkernels, is to prefix the name with `u`.
    name: String,

    /// The language of the kernel
    ///
    /// Used to be able to return a `Kernel` spec.
    languages: Vec<String>,

    /// Is the kernel available on the current operating system?
    available: bool,

    /// Is the kernel interrupt-able on the current operating system?
    interruptable: bool,

    /// Is the kernel fork-able on the current operating system?
    forkable: bool,

    /// A specification of the runtime executable needed for the kernel
    runtime: (String, String),

    /// Arguments that should be supplied to the runtime binary
    ///
    /// Use the argument `"{{script}}"` as a placeholder for the name
    /// of the script file.
    args: Vec<String>,

    /// The script that runs the kernel
    #[serde(skip)]
    script: (String, String),

    /// Other files that the kernel uses (e.g. codec)
    #[serde(skip)]
    others: Vec<(String, String)>,

    /// The code template for setting a variable
    #[serde(skip)]
    set_template: String,

    /// The code template for getting a variable
    #[serde(skip)]
    get_template: String,

    /// The working directory of the kernel (when it was started)
    directory: Option<PathBuf>,

    /// The process id of the kernel
    pid: Option<u32>,

    /// The process id of the parent kernel (if a fork)
    parent_pid: Option<u32>,

    /// The child process of the kernel (`None` for forks)
    #[serde(skip)]
    child: Option<Child>,

    /// The state of the microkernel
    ///
    /// A single Mutex so be able to efficiently synchronize access to the
    /// input and output streams
    #[serde(skip)]
    state: Option<Arc<Mutex<MicroKernelState>>>,

    #[serde(skip)]
    status: Arc<RwLock<KernelStatus>>,
}

#[derive(Debug)]
enum Stdin {
    Child(BufWriter<ChildStdin>),
    #[allow(dead_code)] // Not used on Windows
    File(BufWriter<File>),
}

#[derive(Debug)]
enum Stdout {
    Child(BufReader<ChildStdout>),
    #[allow(dead_code)] // Not used on Windows
    File(BufReader<File>),
}

#[derive(Debug)]
enum Stderr {
    Child(BufReader<ChildStderr>),
    File(BufReader<File>),
}

#[derive(Debug)]
struct MicroKernelState {
    stdin: Option<Stdin>,
    stdout: Stdout,
    stderr: Stderr,
}

impl MicroKernelState {
    /// Send a task to the microkernel and receive results
    async fn send_receive(&mut self, task: &[String]) -> Result<(TaskOutputs, TaskMessages)> {
        self.send_task(task).await?;
        self.receive_result().await
    }

    /// Send a task to the microkernel
    async fn send_task(&mut self, task: &[String]) -> Result<()> {
        match &mut self.stdin {
            Some(Stdin::Child(stdin)) => send_task(task, stdin).await,
            Some(Stdin::File(stdin)) => send_task(task, stdin).await,
            _ => bail!("The kernel has no stdin"),
        }
    }

    /// Receive outputs and messages from the microkernel
    async fn receive_result(&mut self) -> Result<(TaskOutputs, TaskMessages)> {
        match (&mut self.stdout, &mut self.stderr) {
            (Stdout::Child(stdout), Stderr::Child(stderr)) => receive_results(stdout, stderr).await,
            (Stdout::File(stdout), Stderr::File(stderr)) => receive_results(stdout, stderr).await,
            _ => unreachable!(),
        }
    }
}

impl MicroKernel {
    /// Create a new `MicroKernel`
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        languages: &[&str],
        available: bool,
        interruptable: bool,
        forkable: bool,
        runtime: (&str, &str),
        args: &[&str],
        script: (&str, &str),
        others: &[(&str, &str)],
        set_template: &str,
        get_template: &str,
    ) -> Self {
        Self {
            name: name.into(),
            languages: languages.iter().map(|lang| lang.to_string()).collect(),
            available,
            interruptable,
            forkable,
            runtime: (runtime.0.into(), runtime.1.into()),
            args: args.iter().map(|arg| arg.to_string()).collect(),
            script: (script.0.to_string(), script.1.to_string()),
            others: others
                .iter()
                .map(|file| (file.0.to_string(), file.1.to_string()))
                .collect(),
            set_template: set_template.into(),
            get_template: get_template.into(),

            directory: None,
            pid: None,
            parent_pid: None,
            child: None,
            state: None,
            status: Arc::new(RwLock::new(KernelStatus::Pending)),
        }
    }
}

impl Clone for MicroKernel {
    fn clone(&self) -> Self {
        Self {
            // Config fields that can be cloned
            // For forks, `script` and `other` is not necessary so as an optimization,
            // we could potentially avoid cloning these.
            name: self.name.clone(),
            languages: self.languages.clone(),
            available: self.available,
            interruptable: self.interruptable,
            forkable: self.forkable,
            runtime: self.runtime.clone(),
            args: self.args.clone(),
            script: self.script.clone(),
            others: self.others.clone(),
            set_template: self.set_template.clone(),
            get_template: self.get_template.clone(),

            // Runtime fields that should be set to None for the clone
            directory: None,
            pid: None,
            parent_pid: None,
            child: None,
            state: None,

            // Assume clones are initially pending
            status: Arc::new(RwLock::new(KernelStatus::Pending)),
        }
    }
}

/// Include a file as a (name, content) tuple
#[macro_export]
macro_rules! include_file {
    ($name:literal) => {
        ($name, include_str!($name))
    };
}

/// A signaller for a Microkernel
///
/// A `MicroKernelSignaller` can be created once a kernel has started and used to interrupt
/// or kill it asynchronously while it is executing.
pub struct MicroKernelSignaller {
    // The process id of the microkernel
    #[allow(dead_code)] // Not used on Windows
    pid: u32,
}

impl MicroKernelSignaller {
    /// Create a new signaller for a [`MicroKernel`]
    pub fn new(microkernel: &MicroKernel) -> Result<Self> {
        match microkernel.pid {
            Some(pid) => Ok(Self { pid }),
            None => bail!("Microkernel has no process id; has it been started?"),
        }
    }

    /// Interrupt the [`MicroKernel`]
    pub fn interrupt(&self) {
        #[cfg(not(target_os = "windows"))]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            tracing::debug!("Sending interrupt to microkernel with pid `{}`", self.pid);
            if let Err(error) = signal::kill(Pid::from_raw(self.pid as i32), Signal::SIGINT) {
                tracing::warn!(
                    "While interrupting microkernel with pid `{}`: {}",
                    self.pid,
                    error
                )
            }
        }
    }

    /// Kill the [`MicroKernel`]
    pub fn kill(&self) {
        #[cfg(not(target_os = "windows"))]
        {
            use nix::errno::Errno::ESRCH;
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            // Note that killing a microkernel this way may cause it to become a zombie process (shown as <defunct> by `ps`)
            // if the parent kernel process is still waiting for its exit signal. This depends on how the kernel is
            // implement (e.g. in `r-kernel.r` we use the `estranged` flag to avoid this).
            if let Err(error) = signal::kill(Pid::from_raw(self.pid as i32), Signal::SIGKILL) {
                // Only warn if the error is not "No such process" (in case it already ended)
                if error != ESRCH {
                    tracing::warn!(
                        "While killing microkernel with pid `{}`: {}",
                        self.pid,
                        error
                    )
                }
            }
        }
    }
}

#[async_trait]
impl KernelTrait for MicroKernel {
    /// Get the [`Kernel`] specification
    async fn spec(&self) -> Kernel {
        // Override `self.available == true` with check that binary is installed
        let available = if !self.available {
            false
        } else {
            let (name, semver) = &self.runtime;
            binaries::installed(name, semver).await
        };

        Kernel {
            name: self.name.clone(),
            r#type: KernelType::Micro,
            languages: self.languages.clone(),
            available,
            interruptable: self.interruptable,
            forkable: self.forkable,
        }
    }

    /// Start the kernel
    ///
    /// An override of `KernelTrait::start` that searches for the preferred executable
    /// and runs it using specified commands, including the kernel script file if specified
    /// in the arguments.
    async fn start(&mut self, directory: &Path) -> Result<()> {
        // Resolve the directory where kernels are run
        let user_data_dir = dirs::data_dir().unwrap_or_else(|| {
            env::current_dir().expect("Should always be able to get current dir")
        });
        let dir = match env::consts::OS {
            "macos" | "windows" => user_data_dir.join("Stencila").join("Microkernels"),
            _ => user_data_dir.join("stencila").join("microkernels"),
        };
        fs::create_dir_all(&dir)?;

        // Copy over script file
        let script_path = dir.join(&self.script.0);
        fs::write(&script_path, &self.script.1)?;

        // Copy over other files
        for other in &self.others {
            fs::write(dir.join(&other.0), &other.1)?;
        }

        // Construct args array, inserting script path where appropriate
        let args: Vec<String> = self
            .args
            .iter()
            .map(|arg| match arg.as_str() {
                "{{script}}" => script_path.display().to_string(),
                _ => arg.to_string(),
            })
            .collect();
        let args: Vec<&str> = args.iter().map(|arg| arg.as_str()).collect();

        // Start child process
        let (name, semver) = &self.runtime;
        let binary = binaries::installation(name, semver).await?;
        let mut child = binary.interact(&args, directory)?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| eyre!("Child has no stdin handle"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| eyre!("Child has no stdout handle"))?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| eyre!("Child has no stderr handle"))?;

        self.directory = Some(directory.to_owned());
        self.pid = child.id();
        self.child = Some(child);

        let stdin = BufWriter::new(stdin);
        let mut stdout = BufReader::new(stdout);
        let mut stderr = BufReader::new(stderr);

        startup_warnings(&self.name, &mut stdout, &mut stderr).await;

        self.state = Some(Arc::new(Mutex::new(MicroKernelState {
            stdin: Some(Stdin::Child(stdin)),
            stdout: Stdout::Child(stdout),
            stderr: Stderr::Child(stderr),
        })));

        *self.status.write().await = KernelStatus::Idle;
        Ok(())
    }

    /// Stop the kernel
    ///
    /// An override of `KernelTrait::stop` that kills the child process.
    async fn stop(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            // For main kernels
            tracing::debug!("Killing kernel with pid `{:?}`", self.pid);
            child.kill().await?;
            self.child = None;
        } else if let Some(..) = self.pid {
            // For forks
            MicroKernelSignaller::new(self)?.kill();
            self.pid = None;
        }

        *self.status.write().await = KernelStatus::Finished;
        Ok(())
    }

    /// Get the status of the kernel
    async fn status(&self) -> Result<KernelStatus> {
        Ok(self.status.read().await.clone())
    }

    /// Get a symbol from the kernel
    async fn get(&mut self, name: &str) -> Result<Node> {
        let code = self.get_template.replace("{{name}}", name);

        let (outputs, messages) = self.state().await.send_receive(&[code]).await?;

        if let Some(output) = outputs.first() {
            Ok(output.clone())
        } else {
            // TODO: When messages include CodeWarning etc log those
            // and only bail on CodeError
            let message = messages
                .first()
                .map(|message| message.error_message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            bail!("Unable to get symbol `{}`: {}", name, message)
        }
    }

    /// Set a symbol in the kernel
    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        let json = serde_json::to_string(&value)?;
        let code = self
            .set_template
            .replace("{{name}}", name)
            .replace("{{json}}", &json);

        let (_outputs, messages) = self.state().await.send_receive(&[code.to_string()]).await?;

        if messages.is_empty() {
            Ok(())
        } else {
            // TODO: When messages include CodeWarning etc log those
            // and only bail on CodeError
            let message = messages
                .first()
                .map(|message| message.error_message.clone())
                .unwrap_or_else(|| "Unknown error".to_string());
            bail!("Unable to set symbol `{}`: {}", name, message)
        }
    }

    /// Execute code in the kernel synchronously
    async fn exec_sync(&mut self, code: &str, _tags: Option<&TagMap>) -> Result<Task> {
        let mut task = Task::begin_sync();
        let (outputs, messages) = self.state().await.send_receive(&[code.to_string()]).await?;
        let result = TaskResult::new(outputs, messages);
        task.end(result);

        Ok(task)
    }

    /// Execute code in the kernel asynchronously
    async fn exec_async(&mut self, code: &str, _tags: Option<&TagMap>) -> Result<Task> {
        // Setup channels and execution task
        let (result_forwarder, ..) = broadcast::channel(1);
        let (interrupt_sender, interrupt_receiver) = if self.interruptable {
            let (sender, receiver) = mpsc::channel(1);
            (Some(sender), Some(receiver))
        } else {
            (None, None)
        };
        let task = Task::begin(Some(result_forwarder.clone()), interrupt_sender);

        // Start async task to wait for result and send on to receivers.
        let task_id = task.id.clone();
        let code = code.to_string();
        let state = self.state.as_ref().unwrap().clone();
        let status = self.status.clone();
        tokio::spawn(async move {
            let mut state = state.lock().await;

            *status.write().await = KernelStatus::Busy;

            let (outputs, messages) = match state.send_receive(&[code]).await {
                Ok((ouputs, messages)) => (ouputs, messages),
                Err(error) => {
                    tracing::error!(
                        "When receiving result for exec_async task `{}`: {}",
                        task_id,
                        error
                    );
                    return;
                }
            };
            let result = TaskResult::new(outputs, messages);
            if let Err(error) = result_forwarder.send(result) {
                // The result receiver at the other end of the channel was dropped
                // (e.g. the task was interrupted) so just `debug!`
                tracing::debug!(
                    "When sending result for exec_async task `{}`: {}",
                    task_id,
                    error
                );
            }

            *status.write().await = KernelStatus::Idle;
        });

        if let Some(mut interrupt_receiver) = interrupt_receiver {
            // Start async task to listen for interruption message
            // This should finish when the `interrupter` is either triggered or dropped
            let task_id = task.id.clone();
            let signaller = MicroKernelSignaller::new(self)?;
            tokio::spawn(async move {
                tracing::trace!("Began interrupter for exec_async task `{}", task_id);
                if let Some(..) = interrupt_receiver.recv().await {
                    tracing::debug!("Interrupting exec_async task `{}`", task_id);
                    signaller.interrupt()
                }
                tracing::trace!("Ended interrupter for exec_async task `{}`", task_id);
            });
        }

        Ok(task)
    }

    /// Execute code in a fork of the kernel
    #[cfg(not(target_os = "windows"))]
    async fn exec_fork(&mut self, code: &str, _tags: Option<&TagMap>) -> Result<Task> {
        if !self.is_forkable().await {
            bail!("Kernel `{}` is not forkable", self.name);
        }

        // Setup channels and execution task
        let (sender, _receiver) = broadcast::channel(1);
        let (interrupt_sender, mut interrupt_receiver) = mpsc::channel(1);
        let task = Task::begin(Some(sender.clone()), Some(interrupt_sender));

        // Start the fork and create signaller for it
        let mut fork = self.create_fork(code).await?;
        let signaller = MicroKernelSignaller::new(&fork)?;

        // Start async task to wait for result and send to receivers
        let task_id = task.id.clone();
        tokio::spawn(async move {
            tracing::trace!("Began exec_fork task `{}`", task_id);
            let result = match fork.state().await.receive_result().await {
                Ok((outputs, messages)) => TaskResult::new(outputs, messages),
                Err(error) => {
                    tracing::error!(
                        "When receiving result for exec_fork task `{}`: {}",
                        task_id,
                        error
                    );
                    TaskResult::internal_error("Error while receiving result from fork")
                }
            };
            if let Err(error) = fork.stop().await {
                tracing::warn!("When stopping fork for task `{}`: {}", task_id, error);
            }

            if let Err(error) = sender.send(result) {
                tracing::debug!(
                    "When sending result for exec_fork task `{}`: {}",
                    task_id,
                    error
                );
            }
            tracing::trace!("Ended exec_fork task `{}`", task_id);
        });

        // Start async task to listen for interruption message
        // This should finish when the `interrupter` is either triggered or dropped
        let task_id = task.id.clone();
        tokio::spawn(async move {
            tracing::trace!("Began interrupter for exec_fork task `{}` began", task_id);
            if let Some(..) = interrupt_receiver.recv().await {
                tracing::debug!("Interrupting exec_fork task `{}`", task_id);
                signaller.kill()
            }
            tracing::trace!("Ended interrupter for exec_fork task `{}` ended", task_id);
        });

        Ok(task)
    }
}

impl MicroKernel {
    /// Get the state of the microkernel
    async fn state(&self) -> MutexGuard<'_, MicroKernelState> {
        self.state
            .as_ref()
            .expect("Should only be called after the kernel has started")
            .lock()
            .await
    }

    /// Create a fork of the kernel
    ///
    /// If `code` is NOT empty then the child fork process should execute it and exit
    /// immediately (after sending results and messages over stdout/stderr).
    ///
    /// If `code` IS empty then the child fork process should remain alive and wait for other tasks.
    #[cfg(not(target_os = "windows"))]
    pub async fn create_fork(&self, code: &str) -> Result<MicroKernel> {
        // Avoid "unused import" linter warning on Windows by importing here
        use kernel::common::tempfile::tempdir;

        // Create pipes in a temporary directory (which gets cleaned up when dropped)
        // Not that a stdin is not required for temporary forks since code is sent on
        // the same request.
        use nix::{sys::stat, unistd::mkfifo};
        let pipes_dir = tempdir()?;
        let fork_stdin = if code.is_empty() {
            let fork_stdin = pipes_dir.path().join("stdin.pipe");
            mkfifo(&fork_stdin, stat::Mode::S_IRWXU)?;
            Some(fork_stdin)
        } else {
            None
        };
        let fork_stdout = pipes_dir.path().join("stdout.pipe");
        mkfifo(&fork_stdout, stat::Mode::S_IRWXU)?;
        let fork_stderr = pipes_dir.path().join("stderr.pipe");
        mkfifo(&fork_stderr, stat::Mode::S_IRWXU)?;

        let mut state = self.state().await;

        // Send fork task to the kernel with paths of `stdin` (if any), `stdout` and `stderr`
        // and any code to execute. The kernel process expects these four "argument lines" to FORK,
        // although `stdin` and `code` may be empty lines.
        let task = vec![
            FORK.to_string(),
            fork_stdin
                .as_ref()
                .map_or_else(String::new, |path| path.display().to_string()),
            fork_stdout.display().to_string(),
            fork_stderr.display().to_string(),
            code.to_string(),
        ];
        state.send_task(&task).await?;

        // Receive the process id of the fork from the kernel
        let (outputs, messages) = state.receive_result().await?;
        for message in messages {
            tracing::error!("While forking kernel: {}", message.error_message)
        }
        let fork_pid = if let Some(Node::Integer(pid)) = outputs.first() {
            *pid as u32
        } else {
            bail!("Did not receive a pid for fork")
        };

        // Open the fork `stdin`, `stdout` and `stderr` FIFO pipes. These calls will block until the child
        // process has opened the pipes for reading/writing. So perhaps this should have a timeout
        // in case that fails.
        let fork_stdin = match fork_stdin {
            Some(fork_stdin) => {
                tracing::trace!("Creating {}", fork_stdin.display());
                // This must use `create` so the pipe is opened in write-only mode
                // Otherwise this process and the fork will both be waiting on each other
                // to act as the writer for their reader.
                let fork_stdin = File::create(fork_stdin).await?;
                tracing::trace!("Fork has opened stdin pipe for reading");
                Some(fork_stdin)
            }
            None => None,
        };
        tracing::trace!("Waiting to open {}", fork_stdout.display());
        let fork_stdout = File::open(fork_stdout).await?;
        tracing::trace!("Waiting to open {}", fork_stderr.display());
        let fork_stderr = File::open(fork_stderr).await?;
        tracing::trace!("Fork has opened stdout and stderr pipes for writing");

        Ok(Self {
            parent_pid: self.pid,
            pid: Some(fork_pid),
            child: None,
            state: Some(Arc::new(Mutex::new(MicroKernelState {
                stdin: fork_stdin.map(|stream| Stdin::File(BufWriter::new(stream))),
                stdout: Stdout::File(BufReader::new(fork_stdout)),
                stderr: Stderr::File(BufReader::new(fork_stderr)),
            }))),
            ..self.clone()
        })
    }

    #[cfg(target_os = "windows")]
    pub async fn create_fork(&self, _code: &str) -> Result<MicroKernel> {
        bail!("This method should never be called on Windows because process forking is not available")
    }

    /// Create a "knife" of the kernel
    ///
    /// A "knife" is how we duplicate kernels that can not be forked. A knife is not
    /// an exact duplicate of the kernel because it does not have a copy-on-write copy of the kernel's
    /// memory (including variables and imported modules). Instead, we just spawn a child process
    /// that is as similar as possible to the parent (i.e. same binary etc)
    pub async fn create_knife(&self) -> Result<MicroKernel> {
        let mut knife = self.clone();
        match &self.directory {
            Some(dir) => knife.start(dir).await?,
            None => bail!("Attempting to start a 'knife' from a kernel which has no directory yet (has not been started?)")
        }
        Ok(knife)
    }
}

/// Send a task to a kernel on stdin
async fn send_task<W: AsyncWrite + Unpin>(task: &[String], stdin: &mut BufWriter<W>) -> Result<()> {
    let task = task.join("\n");
    let task = task.replace('\n', NEWLINE);
    let task = [&task, "\n"].concat();
    tracing::trace!("Sending task on stdin");
    if let Err(error) = stdin.write_all(task.as_bytes()).await {
        bail!("When writing code to kernel: {}", error)
    }
    if let Err(error) = stdin.flush().await {
        bail!("When flushing code to kernel: {}", error)
    }
    Ok(())
}

/// Receive outputs on stdout and messages on stderr during kernel startup
/// (until READY flag). Used to "clear" streams and be ready to accept tasks but
/// to also report any messages received.
async fn startup_warnings<R1: AsyncBufRead + Unpin, R2: AsyncBufRead + Unpin>(
    name: &str,
    stdout: &mut R1,
    stderr: &mut R2,
) {
    match receive_results(stdout, stderr).await {
        Ok((.., messages)) => {
            if !messages.is_empty() {
                let messages = messages
                    .into_iter()
                    .map(|message| message.error_message)
                    .collect::<Vec<String>>()
                    .join("\n");
                tracing::warn!(
                    "While starting kernel `{}` got output on stderr: {}",
                    name,
                    messages
                )
            }
        }
        Err(error) => {
            tracing::error!("While starting kernel `{}`: {}", name, error);
        }
    }
}

/// Receive results (outputs on stdout and messages on stderr) from a kernel
async fn receive_results<R1: AsyncBufRead + Unpin, R2: AsyncBufRead + Unpin>(
    stdout: &mut R1,
    stderr: &mut R2,
) -> Result<(Vec<Node>, Vec<CodeError>)> {
    // Capture outputs separating them as we go
    let mut output = String::new();
    let mut outputs = Vec::new();
    let mut lines = stdout.lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line.to_string(),
            Ok(None) => break,
            Err(error) => {
                bail!("When receiving outputs from kernel: {}", error)
            }
        };

        tracing::trace!("Received on stdout: {}", &line);
        if !handle_line(&line, &mut output, &mut outputs) {
            break;
        }
    }

    // Attempt to parse each output as JSON into a `Node`, falling back to a string.
    let outputs: Vec<Node> = outputs
        .into_iter()
        .map(|output| -> Node {
            match serde_json::from_str(&output) {
                // A plain JSON object (ie.. not matching matching any of the entity types)
                // will be deserialized to a plain `Entity` (with all properties get dropped)
                // by `serde`. However, we want it to be an `Object` (wil properties retained)
                // so catch that case.
                Ok(Node::Entity(..)) => {
                    let object =
                        serde_json::from_str::<Object>(&output).unwrap_or_else(|_| Object::new());
                    Node::Object(object)
                }
                Ok(node) => node,
                Err(..) => Node::String(output.strip_suffix('\n').unwrap_or(&output).to_string()),
            }
        })
        .collect();

    // Capture messages separating them as we go
    let mut message = String::new();
    let mut messages = Vec::new();
    let mut lines = stderr.lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line.to_string(),
            Ok(None) => break,
            Err(error) => {
                bail!("When receiving messages from kernel: {}", error)
            }
        };

        tracing::trace!("Received on stderr: {}", &line);
        if !handle_line(&line, &mut message, &mut messages) {
            break;
        }
    }

    // Attempt to parse each message as JSON into a `CodeMessage`.
    let messages: Vec<CodeError> = messages
        .iter()
        .map(|message| -> CodeError {
            serde_json::from_str(message).unwrap_or_else(|_| CodeError {
                error_message: transform_message(message),
                ..Default::default()
            })
        })
        .collect();

    Ok((outputs, messages))
}

/// Handle a line of stdout or stderr
///
/// How the line is handled depends upon whether it has a result or task
/// flag at the end. Returns false at the end of a task.
fn handle_line(line: &str, current: &mut String, vec: &mut Vec<String>) -> bool {
    if let Some(line) = line
        .strip_suffix(RESULT)
        .or_else(|| line.strip_suffix(RESULT_ALT))
    {
        current.push_str(line);
        if !current.is_empty() {
            vec.push(current.clone());
            current.clear();
        }
        true
    } else if let Some(line) = line
        .strip_suffix(TASK)
        .or_else(|| line.strip_suffix(TASK_ALT))
        .or_else(|| line.strip_suffix(READY))
        .or_else(|| line.strip_suffix(READY_ALT))
    {
        current.push_str(line);
        if !current.is_empty() {
            vec.push(current.clone());
        }
        false
    } else {
        current.push_str(line);
        current.push('\n');
        true
    }
}

/**
 * Transform a string message
 *
 * This is used in instances when a message is returned from the kernel that
 * can not be parsed as JSON. It allows use to do some transformations of the messages
 * that are not possible, or would be complicated to do, in some microkernel scripts
 * (particularly those for shells like Bash).
 *
 * It is ad-hoc dealing with kernels on a case-by-case basis here, but in the absence
 * of a trait for microkernels is expedient.
 */
fn transform_message(message: &str) -> String {
    // Bash microkernel: strip the leading filename and re-index the line number
    static BASH_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^.*?: line (\d+):(.*)").expect("Should create regex"));
    if let Some(captures) = BASH_REGEX.captures(message) {
        let line = captures[1].parse::<u32>().unwrap_or(17).saturating_sub(16);
        let rest = &captures[2];
        return format!("line {}:{}", line, rest);
    }

    // ZSH microkernel: starts with a leading '(eval):', repeated for each line there is an error
    static ZSH_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"\(eval\):").expect("Should create regex"));
    if message.starts_with("(eval):") {
        return ZSH_REGEX.replace_all(message, "line ").trim().to_string();
    }

    // Default: unchanged message
    message.to_string()
}

pub mod tests;
