use std::{
    fs::write,
    path::Path,
    process::Stdio,
    sync::atomic::{AtomicU64, Ordering},
};

use which::which;

// Re-exports for the convenience of internal crates implementing
// the `Microkernel` trait
pub use kernel::{
    common, format, schema, tests, Kernel, KernelAvailability, KernelForks, KernelInstance,
    KernelInterrupt, KernelKill, KernelSignal, KernelStatus, KernelTerminate,
};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, eyre, Context, OptionExt, Result},
        itertools::Itertools,
        serde_json,
        strum::Display,
        tempfile::TempDir,
        tokio::{
            self,
            fs::{File, OpenOptions},
            io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
            process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
            sync::{mpsc, watch},
        },
        tracing, which,
    },
    schema::{
        ExecutionMessage, MessageLevel, Node, Null, SoftwareApplication, SoftwareSourceCode,
        Variable,
    },
};

/// A specification for a minimal, lightweight execution kernel in a spawned process
#[async_trait]
pub trait Microkernel: Sync + Send + Kernel {
    /// Get the name of the executable (e.g. `python`) used by this microkernel
    fn executable_name(&self) -> String;

    /// Get the arguments that the executable should be spawned with
    ///
    /// Use the `{{script}}` placeholder for path of the microkernel
    /// script. This default implementation has that placeholder as
    /// the only argument; override it to add more arguments.
    fn executable_arguments(&self) -> Vec<String> {
        vec!["{{script}}".to_string()]
    }

    /// Get the script to run for the microkernel
    ///
    /// For most microkernels the script will be written in an external file
    /// and then `include_str`d into the implementation of this function.
    ///
    /// If you want to break the microkernel implementation into more than one
    /// file then include them and concat them in this method.
    fn microkernel_script(&self) -> String;

    /// Whether the executable used by this microkernel is available on this machine
    ///
    /// Returns `true` if an executable with `executable_name()` is in the `PATH`,
    /// and `false` otherwise.
    fn executable_available(&self) -> bool {
        which(self.executable_name()).is_ok()
    }

    /// An implementation of `Kernel::availability` for microkernels
    ///
    /// Returns `Available` if the microkernel's executable is available
    /// of this machine. Otherwise returns `Installable` to indicate that
    /// it could be available if installed.
    fn microkernel_availability(&self) -> KernelAvailability {
        if self.executable_available() {
            KernelAvailability::Available
        } else {
            KernelAvailability::Installable
        }
    }

    /// An implementation of `Kernel::supports_interrupt` for microkernels
    fn microkernel_supports_interrupt(&self) -> KernelInterrupt {
        if cfg!(unix) {
            KernelInterrupt::Yes
        } else {
            KernelInterrupt::No
        }
    }

    /// An implementation of `Kernel::supports_terminate` for microkernels
    fn microkernel_supports_terminate(&self) -> KernelTerminate {
        if cfg!(unix) {
            KernelTerminate::Yes
        } else {
            KernelTerminate::No
        }
    }

    /// An implementation of `Kernel::supports_kill` for microkernels
    fn microkernel_supports_kill(&self) -> KernelKill {
        if cfg!(unix) {
            KernelKill::Yes
        } else {
            KernelKill::No
        }
    }

    /// An implementation of `Kernel::supports_forks` for microkernels
    fn microkernel_supports_forks(&self) -> KernelForks {
        if cfg!(unix) {
            KernelForks::Yes
        } else {
            KernelForks::No
        }
    }

    /// An implementation of `Kernel::create_instance` for microkernels
    fn microkernel_create_instance(&self, index: u64) -> Result<Box<dyn KernelInstance>> {
        tracing::debug!("Creating microkernel instance");

        // Assign an id for the instance using the index, if necessary, to ensure it is unique
        let id = if index == 0 {
            self.name()
        } else {
            format!("{}-{index}", self.name())
        };

        // Get the path to the executable, failing early if it can not be found
        let executable_name = self.executable_name();
        let executable_path = which(&executable_name).map_err(|error| {
            eyre!(
                "While searching for '{executable_name}' on PATH '{}': {error}",
                std::env::var("PATH").unwrap_or_default()
            )
        })?;

        // Always write the script file, even if it already exists, to allow for changes
        // to the microkernel's script
        let kernels_dir = app::get_app_dir(app::DirType::Kernels, true)?;
        let script_file = kernels_dir.join(self.name());
        write(&script_file, self.microkernel_script())?;

        // Replace placeholder in args with the script path
        let args: Vec<String> = self
            .executable_arguments()
            .into_iter()
            .map(|arg| {
                if arg == "{{script}}" {
                    script_file.to_string_lossy().to_string()
                } else {
                    arg
                }
            })
            .collect();

        // Create the command
        let mut command = Command::new(executable_path);
        command
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set up status and status channel
        let status = KernelStatus::Pending;
        let status_sender = MicrokernelInstance::setup_status_channel(status);

        Ok(Box::new(MicrokernelInstance {
            id,
            command: Some(command),
            child: None,
            pid: 0,
            status,
            status_sender,
            signal_sender: None,
            pipes_dir: None,
            input: None,
            output: None,
            errors: None,
            forks: Default::default(),
        }))
    }
}

/// An instance of a microkernel
pub struct MicrokernelInstance {
    /// The id of the microkernel instance
    id: String,

    /// The command used to start the microkernel instance (for main processes only, not forks)
    command: Option<Command>,

    /// The child process (for main processes only, not forks)
    child: Option<Child>,

    /// The process identifier
    pid: u32,

    /// The status of the microkernel instance
    status: KernelStatus,

    /// A channel sender for the status of the microkernel instance
    status_sender: watch::Sender<KernelStatus>,

    /// A channel sender for sending signals to the microkernel instance
    signal_sender: Option<mpsc::Sender<KernelSignal>>,

    /// The temporary directory for FIFO pipes (for forks only)
    ///
    /// Retained as a field because the temporary dir is automatically
    /// deleted when the `TempDir` is dropped.
    #[allow(unused)]
    pipes_dir: Option<TempDir>,

    /// The input stream for the process
    input: Option<MicrokernelInput>,

    /// The output stream for the process
    output: Option<MicrokernelOutput>,

    /// The error stream for the process
    errors: Option<MicrokernelErrors>,

    /// A counter of forks of this microkernel instance
    forks: AtomicU64,
}

/// An input stream for a microkernel instance
enum MicrokernelInput {
    /// Standard input (stdin)
    Standard(BufWriter<ChildStdin>),

    /// Pipe input (for forks; not used on Windows)
    #[allow(dead_code)]
    Pipe(BufWriter<File>),
}

/// An output stream for a microkernel instance
enum MicrokernelOutput {
    /// Standard output (stdout)
    Standard(BufReader<ChildStdout>),

    /// Pipe output (for forks; not used on Windows)
    #[allow(dead_code)]
    Pipe(BufReader<File>),
}

/// An error stream for a microkernel instance
enum MicrokernelErrors {
    /// Standard error (stderr)
    Standard(BufReader<ChildStderr>),

    /// Pipe output (for forks; not used on Windows)
    #[allow(dead_code)]
    Pipe(BufReader<File>),
}

/// A Unicode flag used within messages sent and received to/from microkernels
#[derive(Display)]
#[strum(serialize_all = "UPPERCASE")]
enum MicrokernelFlag {
    /// Sent by the microkernel instance to signal it is ready for a task
    Ready,
    /// Sent by Rust to signal a newline (`\n`) within the code of a task
    Line,
    /// Sent by Rust to signal the start of an `execute` task
    Exec,
    /// Sent by Rust to signal the start of an `evaluation` task
    Eval,
    /// Sent by Rust to signal the start of a `fork` task
    Fork,
    /// Sent by Rust to get runtime information about the kernel
    Info,
    /// Sent by Rust to get a list of packages/libraries available to the kernel
    Pkgs,
    /// Sent by Rust to signal the start of a `list` task
    List,
    /// Sent by Rust to signal the start of a `get` task
    Get,
    /// Sent by Rust to signal the start of a `set` task
    Set,
    /// Sent by Rust to signal the start of a `remove` task
    Remove,
    /// Sent by the microkernel instance to signal the end of an output or message
    End,
}

impl MicrokernelFlag {
    /// Get the flag as a Unicode code point
    ///
    /// Returns a Unicode code point in the "Supplementary Private Use Area-B".
    /// See https://en.wikipedia.org/wiki/Private_Use_Areas
    fn as_unicode(&self) -> &str {
        use MicrokernelFlag::*;
        match self {
            Ready => "\u{10ACDC}",
            Line => "\u{10ABBA}",
            Info => "\u{10EE15}",
            Pkgs => "\u{10BEC4}",
            Exec => "\u{10B522}",
            Eval => "\u{1010CC}",
            Fork => "\u{10DE70}",
            List => "\u{10C155}",
            Get => "\u{10A51A}",
            Set => "\u{107070}",
            Remove => "\u{10C41C}",
            End => "\u{10CB40}",
        }
    }
}

#[async_trait]
impl KernelInstance for MicrokernelInstance {
    fn name(&self) -> String {
        self.id.clone()
    }

    async fn status(&self) -> Result<KernelStatus> {
        self.get_status()
    }

    fn status_watcher(&self) -> Result<watch::Receiver<KernelStatus>> {
        Ok(self.status_sender.subscribe())
    }

    fn signal_sender(&self) -> Result<mpsc::Sender<KernelSignal>> {
        match &self.signal_sender {
            Some(sender) => Ok(sender.clone()),
            None => bail!("Microkernel has not started yet!"),
        }
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        self.set_status(KernelStatus::Starting)?;

        let Some(command) = &mut self.command else {
            // Must be a fork (already started, so return early)
            return Ok(());
        };

        tracing::info!("Starting {command:?}");

        // Spawn the binary in the directory with stdin, stdout and stderr piped to/from it
        let mut child = command.current_dir(directory).spawn().wrap_err_with(|| {
            format!(
                "unable to start microkernel {}: {:?}",
                self.name(),
                self.command
            )
        })?;

        let pid = child
            .id()
            .ok_or_eyre("Microkernel child process has no PID")?;
        self.pid = pid;

        // Create streams for input, output and errors
        let stdin = child.stdin.take().ok_or_eyre("Child has no stdin handle")?;
        let stdout = child
            .stdout
            .take()
            .ok_or_eyre("Child has no stdout handle")?;
        let stderr = child
            .stderr
            .take()
            .ok_or_eyre("Child has no stderr handle")?;

        // Create stream readers and writers
        let stdin_writer = BufWriter::new(stdin);
        let mut stdout_reader = BufReader::new(stdout);
        let mut stderr_reader = BufReader::new(stderr);

        // Emit any startup warnings and clear streams
        startup_warnings(&mut stdout_reader, &mut stderr_reader).await;

        self.input = Some(MicrokernelInput::Standard(stdin_writer));
        self.output = Some(MicrokernelOutput::Standard(stdout_reader));
        self.errors = Some(MicrokernelErrors::Standard(stderr_reader));

        // Setup signalling channel
        self.signal_sender = Some(Self::setup_signals_channel(self.id.clone(), pid));

        // Check status of the process in case start up errors
        // have caused it to fail
        let status = self
            .get_status()
            .wrap_err_with(|| eyre!("Unable to check status of starting kernel"))?;
        if matches!(status, KernelStatus::Failed | KernelStatus::Stopped) {
            bail!("Startup of `{}` kernel failed; perhaps the runtime version on this machine is not supported?", self.name())
        }

        self.set_status(KernelStatus::Ready)?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.set_status(KernelStatus::Stopping)?;

        tracing::debug!("Killing kernel with pid `{:?}`", self.pid);
        if let Some(mut child) = self.child.take() {
            // Main kernel instance
            // Spawn as task so this thread does not wait unnecessarily
            tokio::spawn(async move {
                child.kill().await.ok();
            });
        } else {
            // Forked kernel instance
            #[cfg(unix)]
            {
                use nix::{
                    sys::signal::{kill, Signal},
                    unistd::Pid,
                };

                if let Err(error) = kill(Pid::from_raw(self.pid as i32), Signal::SIGKILL) {
                    tracing::warn!("While killing `{}` kernel: {error}", self.name())
                }
            }
        }

        self.set_status(KernelStatus::Stopped)?;

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        self.send_receive(MicrokernelFlag::Exec, [code]).await
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        let (mut outputs, messages) = self.send_receive(MicrokernelFlag::Eval, [code]).await?;

        let output = if outputs.is_empty() {
            Node::Null(Null)
        } else {
            outputs.swap_remove(0)
        };

        Ok((output, messages))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        let (mut nodes, messages) = self.send_receive(MicrokernelFlag::Info, []).await?;
        self.check_for_errors(messages, "getting info")?;

        match nodes.pop() {
            Some(Node::SoftwareApplication(node)) => Ok(node),
            node => bail!("Expected a `SoftwareApplication`, got {node:#?}"),
        }
    }

    async fn packages(&mut self) -> Result<Vec<SoftwareSourceCode>> {
        let (nodes, messages) = self.send_receive(MicrokernelFlag::Pkgs, []).await?;

        self.check_for_errors(messages, "getting packages")?;

        nodes
            .into_iter()
            .map(|node| match node {
                Node::SoftwareSourceCode(ssc) => Ok(ssc),
                _ => bail!("Expected a `SoftwareSourceCode`, got {node:#?}"),
            })
            .collect::<Result<Vec<_>>>()
    }

    async fn list(&mut self) -> Result<Vec<Variable>> {
        let (nodes, messages) = self.send_receive(MicrokernelFlag::List, []).await?;

        self.check_for_errors(messages, "listing variables")?;

        nodes
            .into_iter()
            .map(|node| match node {
                Node::Variable(var) => Ok(var),
                _ => bail!("Expected a `Variable`, got: {node:#?}"),
            })
            .collect::<Result<Vec<_>>>()
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        let (mut nodes, messages) = self.send_receive(MicrokernelFlag::Get, [name]).await?;

        self.check_for_errors(messages, "getting variable")?;

        let node = if nodes.is_empty() {
            None
        } else {
            Some(nodes.swap_remove(0))
        };

        Ok(node)
    }

    async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        let (.., messages) = self
            .send_receive(MicrokernelFlag::Set, [name, &serde_json::to_string(value)?])
            .await?;

        self.check_for_errors(messages, "setting variable")?;

        Ok(())
    }

    async fn remove(&mut self, name: &str) -> Result<()> {
        let (.., messages) = self.send_receive(MicrokernelFlag::Remove, [name]).await?;

        self.check_for_errors(messages, "removing variable")?;

        Ok(())
    }

    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        #[cfg(unix)]
        {
            use kernel::common::tempfile::tempdir;
            use nix::{sys::stat, unistd::mkfifo};

            // Create FIFO pipes for stdin, stdout and stderr of fork
            let pipes_dir = tempdir()?;
            let stdin_path = pipes_dir.path().join("stdin.pipe");
            mkfifo(&stdin_path, stat::Mode::S_IRWXU)?;
            let stdout_path = pipes_dir.path().join("stdout.pipe");
            mkfifo(&stdout_path, stat::Mode::S_IRWXU)?;
            let stderr_path = pipes_dir.path().join("stderr.pipe");
            mkfifo(&stderr_path, stat::Mode::S_IRWXU)?;

            // Send task to microkernel process
            let (outputs, messages) = self
                .send_receive(
                    MicrokernelFlag::Fork,
                    [
                        stdin_path.to_string_lossy().as_ref(),
                        stdout_path.to_string_lossy().as_ref(),
                        stderr_path.to_string_lossy().as_ref(),
                    ],
                )
                .await?;

            self.check_for_errors(messages, "creating fork")?;

            // Get the PID of the fork
            let Some(Node::Integer(pid)) = outputs.first() else {
                bail!(
                    "Did not receive pid for fork of microkernel `{}`",
                    self.name()
                )
            };
            let pid = *pid as u32;

            // Open the fork's `stdin` FIFO pipe. This must be opened in write-only mode.
            // Otherwise this process and the fork will both be waiting on each other
            // to act as the writer for their reader.
            tracing::trace!("Creating {}", stdin_path.display());
            let stdin_file = OpenOptions::new().write(true).open(&stdin_path).await?;
            tracing::trace!("Fork has opened stdin pipe for reading");

            // Open the fork's `stdout` and `stderr` FIFO pipes. These calls will block until the child
            // process has opened the pipes for reading/writing. So perhaps this should have a timeout
            // in case that fails.
            tracing::trace!("Waiting to open {}", stdout_path.display());
            let stdout_file = File::open(stdout_path).await?;
            tracing::trace!("Waiting to open {}", stderr_path.display());
            let stderr_file = File::open(stderr_path).await?;
            tracing::trace!("Fork has opened stdout and stderr pipes for writing");

            // Create stream readers and writers
            let stdin_writer = BufWriter::new(stdin_file);
            let mut stdout_reader = BufReader::new(stdout_file);
            let mut stderr_reader = BufReader::new(stderr_file);

            // Emit any startup warnings and clear streams
            startup_warnings(&mut stdout_reader, &mut stderr_reader).await;

            // Create stream readers and writers
            let input = Some(MicrokernelInput::Pipe(stdin_writer));
            let output = Some(MicrokernelOutput::Pipe(stdout_reader));
            let errors = Some(MicrokernelErrors::Pipe(stderr_reader));

            // Create fork id
            let id = format!(
                "{}-fork-{}",
                self.id,
                self.forks.fetch_add(1, Ordering::SeqCst)
            );

            let status = KernelStatus::Ready;
            let status_sender = Self::setup_status_channel(status);

            let signal_sender = Some(Self::setup_signals_channel(id.clone(), pid));

            Ok(Box::new(Self {
                id,
                command: None,
                child: None,
                pid,
                status,
                status_sender,
                signal_sender,
                pipes_dir: Some(pipes_dir),
                input,
                output,
                errors,
                forks: Default::default(),
            }))
        }

        #[cfg(windows)]
        {
            bail!("Microkernel forking is not supported on Windows");
        }
    }
}

impl MicrokernelInstance {
    /// Whether a microkernel instance is a fork
    fn is_fork(&self) -> bool {
        self.command.is_none()
    }

    /// Crate a channel for broadcasting status updates
    fn setup_status_channel(init: KernelStatus) -> watch::Sender<KernelStatus> {
        let (status_sender, _) = watch::channel(init);

        status_sender
    }

    /// Create a channel and task for forwarding signals to the microkernel process
    fn setup_signals_channel(id: String, pid: u32) -> mpsc::Sender<KernelSignal> {
        let (signal_sender, mut signal_receiver) = mpsc::channel(1);

        // Start a task to handle signals
        tokio::spawn(async move {
            while let Some(kernel_signal) = signal_receiver.recv().await {
                #[cfg(unix)]
                {
                    use nix::{
                        sys::signal::{kill, Signal},
                        unistd::Pid,
                    };

                    let (name, signal) = match kernel_signal {
                        KernelSignal::Interrupt => ("Interrupting", Signal::SIGINT),
                        KernelSignal::Terminate => ("Terminating", Signal::SIGTERM),
                        KernelSignal::Kill => ("Killing", Signal::SIGKILL),
                    };

                    tracing::debug!("{name} `{id}` kernel with pid `{pid}`");

                    if matches!(signal, Signal::SIGINT) {
                        // On Linux using `nix::sys::signal::kill` with `SIGINT` has no effect
                        // for some unknown reason.
                        // This is a workaround which uses the system's `kill` command.
                        let mut killer = match Command::new("kill")
                            .args(["-s", signal.as_str(), &pid.to_string()])
                            .spawn()
                        {
                            Ok(killer) => killer,
                            Err(error) => {
                                tracing::error!("While spawning killer: {error}");
                                continue;
                            }
                        };
                        if let Err(error) = killer.wait().await {
                            tracing::error!("While {name} `{id}` kernel: {error}")
                        }
                    } else if let Err(error) = kill(Pid::from_raw(pid as i32), signal) {
                        tracing::warn!("While {name} `{id}` kernel: {error}")
                    }
                }

                #[cfg(windows)]
                {
                    tracing::error!("Signals not yet supported on Windows",)
                }
            }
        });

        signal_sender
    }
    /// Get the status of the microkernel instance
    ///
    /// Will query the child process to check it is still alive and if
    /// not its exit code.
    fn get_status(&self) -> Result<KernelStatus> {
        if
        // Don't call `waitpid` if not yet started or has been explicitly stopped
        self.pid == 0 || matches!(
            self.status,
            KernelStatus::Pending | KernelStatus::Stopping | KernelStatus::Stopped
        )
        // Can't call `waitpid` on forks because they are not direct child processes
        || self.is_fork()
        {
            return Ok(self.status);
        }

        #[cfg(unix)]
        {
            use nix::{
                sys::{
                    signal,
                    wait::{waitpid, WaitPidFlag, WaitStatus},
                },
                unistd::Pid,
            };

            let pid = Pid::from_raw(self.pid as i32);
            match signal::kill(pid, None) {
                Ok(..) => match waitpid(pid, Some(WaitPidFlag::WNOHANG))? {
                    WaitStatus::StillAlive => Ok(self.status),
                    WaitStatus::Exited(.., code) => {
                        if code == 0 {
                            Ok(KernelStatus::Stopped)
                        } else {
                            Ok(KernelStatus::Failed)
                        }
                    }
                    _ => Ok(KernelStatus::Failed),
                },
                Err(..) => Ok(KernelStatus::Failed),
            }
        }

        #[cfg(windows)]
        {
            OK(self.status)
        }
    }

    /// Set the status of the microkernel instance and notify watchers
    fn set_status(&mut self, status: KernelStatus) -> Result<()> {
        self.status = status;
        self.status_sender.send_if_modified(|previous| {
            if status != *previous {
                tracing::trace!(
                    "Status of `{}` kernel changed from `{previous}` to `{status}`",
                    self.name()
                );
                *previous = status;
                true
            } else {
                false
            }
        });

        Ok(())
    }

    /// Send a task to the microkernel instance and receive results
    async fn send_receive<'lt, I>(
        &mut self,
        flag: MicrokernelFlag,
        args: I,
    ) -> Result<(Vec<Node>, Vec<ExecutionMessage>)>
    where
        I: IntoIterator<Item = &'lt str>,
    {
        self.set_status(KernelStatus::Busy)?;

        let args = args.into_iter().join(MicrokernelFlag::Line.as_unicode());

        self.send(flag, &args).await?;
        let result = self.receive().await;

        self.set_status(KernelStatus::Ready)?;

        result
    }

    /// Send a task to this microkernel instance
    async fn send(&mut self, flag: MicrokernelFlag, code: &str) -> Result<()> {
        let Some(input) = self.input.as_mut() else {
            bail!("Microkernel `{}` has not been started yet!", self.name());
        };

        match input {
            MicrokernelInput::Standard(input) => send_task(flag, code, input).await,
            MicrokernelInput::Pipe(input) => send_task(flag, code, input).await,
        }
    }

    /// Receive outputs and messages from this microkernel instance
    async fn receive(&mut self) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        let (Some(output), Some(errors)) = (self.output.as_mut(), self.errors.as_mut()) else {
            bail!("Microkernel has not been started yet!");
        };

        match (output, errors) {
            (MicrokernelOutput::Standard(output), MicrokernelErrors::Standard(errors)) => {
                receive_results(output, errors).await
            }
            (MicrokernelOutput::Pipe(output), MicrokernelErrors::Pipe(errors)) => {
                receive_results(output, errors).await
            }
            _ => unreachable!(),
        }
    }

    /// Create an `Err` if messages from the kernel include an error
    fn check_for_errors(&self, messages: Vec<ExecutionMessage>, action: &str) -> Result<()> {
        if messages.iter().any(|m| m.level == MessageLevel::Error) {
            bail!(
                "While {action} in microkernel `{}`: {}",
                self.name(),
                messages.into_iter().map(|message| message.message).join("")
            )
        } else {
            Ok(())
        }
    }
}

/// Receive outputs on stdout and messages on stderr during kernel startup
/// (until READY flag). Used to "clear" streams and be ready to accept tasks but
/// to also report any messages received.
async fn startup_warnings<R1: AsyncBufRead + Unpin, R2: AsyncBufRead + Unpin>(
    stdout: &mut R1,
    stderr: &mut R2,
) {
    match receive_results(stdout, stderr).await {
        Ok((.., messages)) => {
            if !messages.is_empty() {
                let messages = messages
                    .into_iter()
                    .map(|message| message.message)
                    .collect::<Vec<String>>()
                    .join("\n");
                tracing::warn!("While starting kernel got output on stderr: {messages}")
            }
        }
        Err(error) => {
            tracing::error!("While starting kernel: {error}");
        }
    }
}

/// Send a task to a microkernel instance
async fn send_task<W: AsyncWrite + Unpin>(
    flag: MicrokernelFlag,
    code: &str,
    input_stream: &mut BufWriter<W>,
) -> Result<()> {
    let task = [
        flag.as_unicode(),
        MicrokernelFlag::Line.as_unicode(),
        code.replace('\n', MicrokernelFlag::Line.as_unicode())
            .as_str(),
        "\n",
    ]
    .concat();

    tracing::trace!("Sending {flag} task to microkernel");
    if let Err(error) = input_stream.write_all(task.as_bytes()).await {
        bail!("When writing code to kernel: {error}")
    }
    if let Err(error) = input_stream.flush().await {
        bail!("When flushing code to kernel: {error}")
    }

    Ok(())
}

/// Receive results (outputs and messages) from a microkernel instance
async fn receive_results<R1: AsyncBufRead + Unpin, R2: AsyncBufRead + Unpin>(
    output_stream: &mut R1,
    message_stream: &mut R2,
) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
    tracing::trace!("Receiving results from microkernel");

    // Collect separate output strings
    let mut item = String::new();
    let mut items = Vec::new();
    let mut lines = output_stream.lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line.to_string(),
            Ok(None) => break,
            Err(error) => {
                bail!("When receiving outputs from kernel: {}", error)
            }
        };

        //tracing::trace!("Received on output stream: {}", &line);
        if !handle_line(&line, &mut item, &mut items) {
            break;
        }
    }

    // Attempt to parse each output as JSON into a `Node`, falling back to a string.
    let outputs: Vec<Node> = items
        .into_iter()
        .map(|output| -> Node {
            match serde_json::from_str(&output) {
                Ok(node) => node,
                Err(..) => Node::String(output),
            }
        })
        .collect();

    let mut item = String::new();
    let mut items = Vec::new();
    let mut lines = message_stream.lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line.to_string(),
            Ok(None) => break,
            Err(error) => {
                bail!("When receiving outputs from kernel: {}", error)
            }
        };

        //tracing::trace!("Received on message stream: {}", &line);
        if !handle_line(&line, &mut item, &mut items) {
            break;
        }
    }
    let messages = items
        .into_iter()
        .map(|message| -> ExecutionMessage {
            match serde_json::from_str(&message) {
                Ok(message) => message,
                Err(..) => ExecutionMessage::new(MessageLevel::Error, message),
            }
        })
        .collect_vec();

    Ok((outputs, messages))
}

/// Handle a line on an output or error stream
fn handle_line(line: &str, item: &mut String, items: &mut Vec<String>) -> bool {
    if let Some(line) = line.strip_suffix(MicrokernelFlag::End.as_unicode()) {
        item.push_str(line);
        if !item.is_empty() {
            items.push(item.clone());
            item.clear();
        }
        true
    } else if let Some(line) = line.strip_suffix(MicrokernelFlag::Ready.as_unicode()) {
        item.push_str(line);
        if !item.is_empty() {
            items.push(item.clone());
        }
        false
    } else {
        item.push_str(line);
        item.push('\n');
        true
    }
}
