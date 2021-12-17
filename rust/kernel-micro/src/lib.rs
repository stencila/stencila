use kernel::{
    async_trait::async_trait,
    eyre::{bail, eyre, Result},
    stencila_schema::{CodeError, Node, Object},
    Kernel, KernelStatus, KernelTrait, KernelType,
};
use serde::Serialize;
use std::{env, fs};
use tempfile::tempdir;
use tokio::{
    fs::File,
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout},
};

// Line end flags for the Microkernel protocol
// On Windows, Rscript (and possibly other binaries) escapes unicode on stdout and stderr
// So the _ALT flags are provided for these instances (or where it is not possible to output Unicode at all).

/// Indicates the end of kernel startup, kernel is ready to perform tasks.
const READY: char = '\u{10ACDC}';
const READY_ALT: &str = "<U+0010ACDC>";

/// Indicates the end of a task result ("outputs" on `stderr` and "messages" on `stderr`).
const RESULT: char = '\u{10CB40}';
const RESULT_ALT: &str = "<U+0010CB40>";

/// Indicates the end of a task, kernel is ready for next task.
const TASK: char = '\u{10ABBA}';
const TASK_ALT: &str = "<U+0010ABBA>";

/// Indicates that the task should be run in a forked process.
const FORK: char = '\u{10DE70}';
#[allow(dead_code)]
const FORK_ALT: &str = "<U+0010DE70>";

#[derive(Debug, Serialize)]
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

    /// The current status of the kernel
    status: KernelStatus,

    /// The process id of the kernel
    pid: Option<u32>,

    /// The process id of the parent kernel (if a fork)
    forked_from: Option<u32>,

    /// The child process of the kernel (`None` for forks)
    #[serde(skip)]
    child: Option<Child>,

    /// The writer for the stdin stream of the child process
    #[serde(skip)]
    stdin: Option<Stdin>,

    /// The reader for the stdout stream of the child process
    #[serde(skip)]
    stdout: Option<Stdout>,

    /// The reader for the stderr stream of the child process
    #[serde(skip)]
    stderr: Option<Stderr>,
}

#[derive(Debug)]
enum Stdin {
    Child(BufWriter<ChildStdin>),
    #[allow(dead_code)]
    File(BufWriter<File>),
}

#[derive(Debug)]
enum Stdout {
    Child(BufReader<ChildStdout>),
    File(BufReader<File>),
}

#[derive(Debug)]
enum Stderr {
    Child(BufReader<ChildStderr>),
    File(BufReader<File>),
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

            status: KernelStatus::Pending,
            pid: None,
            forked_from: None,
            child: None,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }
}

impl Clone for MicroKernel {
    fn clone(&self) -> Self {
        let Self {
            name,
            languages,
            available,
            interruptable,
            forkable,
            runtime,
            args,
            set_template,
            get_template,
            ..
        } = self;
        Self {
            // Small properties required for fork operation and/or display
            name: name.clone(),
            languages: languages.clone(),
            available: *available,
            interruptable: *interruptable,
            forkable: *forkable,
            runtime: runtime.clone(),
            args: args.clone(),
            // Large properties not required for fork operation
            script: (String::new(), String::new()),
            others: Vec::new(),
            // Small properties that may be needed for fork to get symbols
            set_template: set_template.clone(),
            get_template: get_template.clone(),

            status: KernelStatus::Pending,
            pid: None,
            forked_from: None,
            child: None,
            stdin: None,
            stdout: None,
            stderr: None,
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

/// An interrupter for a Microkernel
///
/// A `MicroKernelInterrupter` can be created once a kernel has started and used to interrupt
/// it asynchronously while it is executing.
pub struct MicroKernelInterrupter {
    // The process id of the microkernel
    pid: u32,
}

impl MicroKernelInterrupter {
    pub fn new(microkernel: &MicroKernel) -> Result<Self> {
        match microkernel.child.as_ref().and_then(|child| child.id()) {
            Some(pid) => Ok(Self { pid }),
            None => bail!("Microkernel has no process id; has it been started?"),
        }
    }

    pub fn interrupt(&self) {
        #[cfg(not(target_os = "windows"))]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;

            signal::kill(Pid::from_raw(self.pid as i32), Signal::SIGINT).unwrap()
        }
    }
}

#[async_trait]
impl KernelTrait for MicroKernel {
    /// Get the [`Kernel`] specification
    fn spec(&self) -> Kernel {
        Kernel {
            name: self.name.clone(),
            r#type: KernelType::Micro,
            languages: self.languages.clone(),
        }
    }

    /// Is the kernel available on the current machine?
    ///
    /// Returns `true` if the operating system is listed in `oses` and
    /// a runtime matching the semver requirements in `runtime` is found to be installed.
    async fn available(&self) -> bool {
        if !self.available {
            return false;
        }
        let (name, semver) = &self.runtime;
        binaries::installed(name, semver).await
    }

    /// Is the kernel interruptable on the current machine?
    ///
    /// Although the microkernel itself may handle interrupts across operating systems,
    /// here we only support if for *nix. So return false, if on Windows
    async fn interruptable(&self) -> bool {
        self.interruptable && cfg!(not(target_os = "windows"))
    }

    /// Is the kernel forkable on the current machine?
    async fn forkable(&self) -> bool {
        self.forkable
    }

    /// Start the kernel
    ///
    /// An override of `KernelTrait::start` that searches for the preferred executable
    /// and runs it using specified commands, including the kernel script file if specified
    /// in the arguments.
    async fn start(&mut self) -> Result<()> {
        self.status = KernelStatus::Starting;

        // Resolve the directory where kernels are run
        let user_data_dir = dirs::data_dir().unwrap_or_else(|| env::current_dir().unwrap());
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

        // Start child process
        let (name, semver) = &self.runtime;
        let binary = binaries::installation(name, semver).await?;
        let mut child = binary.interact(&args)?;

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

        self.pid = child.id();
        self.child = Some(child);
        self.stdin = Some(Stdin::Child(BufWriter::new(stdin)));
        self.stdout = Some(Stdout::Child(BufReader::new(stdout)));
        self.stderr = Some(Stderr::Child(BufReader::new(stderr)));

        // Wait for READY flags
        let (_outputs, messages) = self.receive_results().await?;
        if !messages.is_empty() {
            let messages = messages
                .into_iter()
                .map(|message| message.error_message)
                .collect::<Vec<String>>()
                .join("\n");
            tracing::warn!(
                "While starting kernel `{}` got output on stderr: {}",
                self.name,
                messages
            )
        }

        self.status = KernelStatus::Idle;

        Ok(())
    }

    /// Stop the kernel
    ///
    /// An override of `KernelTrait::stop` that kills the child process.
    async fn stop(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            // For main kernels
            self.status = KernelStatus::Stopping;
            tracing::debug!("Killing kernel with pid `{:?}`", self.pid);
            child.kill().await?;
            self.child = None;
            self.status = KernelStatus::Finished;
        } else if let Some(pid) = self.pid {
            // For forks
            self.status = KernelStatus::Stopping;
            #[cfg(not(target_os = "windows"))]
            {
                use nix::sys::signal::{self, Signal};
                use nix::unistd::Pid;

                // Note that killing the for from here may cause it to be
                // a zombie process (<defunct>) if the parent kernel process is still waiting
                // for its exit signal. This depends on how the parent kernel forks
                // (in `r-kernel.r` we use the `estranged` flag to avoid this).
                tracing::debug!("Killing kernel fork with pid `{}`", pid);
                signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL).unwrap()
            }
            self.pid = None;
            self.status = KernelStatus::Finished;
        }
        Ok(())
    }

    /// Get the status of the kernel
    async fn status(&self) -> Result<KernelStatus> {
        Ok(self.status.clone())
    }

    /// Get a symbol from the kernel
    async fn get(&mut self, name: &str) -> Result<Node> {
        let code = self.get_template.replace("{{name}}", name);

        let (outputs, messages) = self.exec(&code).await?;

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

        let (.., messages) = self.exec(&code).await?;

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

    /// Execute some code in the kernel
    async fn exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)> {
        // Send the code to the kernel
        let task = [&code.replace("\n", "\\n"), "\n"].concat();
        if let Err(error) = self.send_task(&task).await {
            self.status = KernelStatus::Failed;
            bail!(error)
        };

        self.receive_results().await
    }

    /// Fork the kernel and execute code in the fork
    #[cfg(not(target_os = "windows"))]
    async fn fork_exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)> {
        if !self.forkable().await {
            bail!("Kernel `{}` is not forkable", self.name);
        }

        let mut fork = self.fork(code).await?;
        let results = fork.receive_results().await;
        fork.stop().await?;

        results
    }

    /// Interrupt the task currently being run by the kernel
    async fn interrupt(&mut self) -> Result<()> {
        if !self.interruptable().await {
            tracing::warn!("Kernel `{}` is not interruptable", self.name);
        } else {
            MicroKernelInterrupter::new(self)?.interrupt()
        }
        Ok(())
    }
}

impl MicroKernel {
    // Send a task to the microkernel
    async fn send_task(&mut self, task: &str) -> Result<()> {
        match self.stdin.as_mut() {
            Some(Stdin::Child(stdin)) => send_task(task, stdin).await,
            Some(Stdin::File(stdin)) => send_task(task, stdin).await,
            None => bail!("Kernel has no stdin"),
        }
    }

    /// Receive outputs and messages from the microkernel
    async fn receive_results(&mut self) -> Result<(Vec<Node>, Vec<CodeError>)> {
        match (self.stdout.as_mut(), self.stderr.as_mut()) {
            (Some(Stdout::Child(stdout)), Some(Stderr::Child(stderr))) => {
                receive_results(stdout, stderr).await
            }
            (Some(Stdout::File(stdout)), Some(Stderr::File(stderr))) => {
                receive_results(stdout, stderr).await
            }
            _ => bail!("Kernel has no, or unexpected, stdout and/or stderr"),
        }
    }

    #[cfg(not(target_os = "windows"))]
    async fn fork(&mut self, code: &str) -> Result<MicroKernel> {
        // Create pipes in a temporary directory (which gets cleaned up when dropped)
        use nix::{sys::stat, unistd::mkfifo};
        let pipes_dir = tempdir().unwrap();
        let fork_stdout = pipes_dir.path().join("stdout.pipe");
        mkfifo(&fork_stdout, stat::Mode::S_IRWXU)?;
        let fork_stderr = pipes_dir.path().join("stderr.pipe");
        mkfifo(&fork_stderr, stat::Mode::S_IRWXU)?;

        // Send code and pipes to the kernel
        let task = format!(
            "{}|{};{}{}\n",
            code.replace("\n", "\\n"),
            fork_stdout.display(),
            fork_stderr.display(),
            FORK
        );
        if let Err(error) = self.send_task(&task).await {
            self.status = KernelStatus::Failed;
            bail!(error)
        };

        // Receive the process id of the fork from the kernel
        let (outputs, messages) = self.receive_results().await?;
        for message in messages {
            tracing::error!("While forking kernel: {}", message.error_message)
        }
        let fork_pid = if let Some(Node::Integer(pid)) = outputs.first() {
            *pid as u32
        } else {
            bail!("Did not receive a pid for fork")
        };

        // Open the fork `stdout` and `stderr` FIFO pipes. These calls will block until the child
        // process has opened the pipes for writing. So perhaps this should have a timeout
        // in case that fails.
        tracing::debug!("Waiting to open stdout");
        let fork_stdout = File::open(fork_stdout).await?;
        tracing::debug!("Waiting to open stderr");
        let fork_stderr = File::open(fork_stderr).await?;
        tracing::debug!("Fork has opened stdout and stderr");

        let mut fork = self.clone();
        fork.pid = Some(fork_pid);
        fork.forked_from = self.pid;
        fork.stdout = Some(Stdout::File(BufReader::new(fork_stdout)));
        fork.stderr = Some(Stderr::File(BufReader::new(fork_stderr)));
        Ok(fork)
    }
}

/// Send a task to a kernel on stdin
async fn send_task<W: AsyncWrite + Unpin>(task: &str, stdin: &mut BufWriter<W>) -> Result<()> {
    tracing::debug!("Sending task on stdin");
    if let Err(error) = stdin.write_all(task.as_bytes()).await {
        bail!("When writing code to kernel: {}", error)
    }
    if let Err(error) = stdin.flush().await {
        bail!("When flushing code to kernel: {}", error)
    }
    Ok(())
}

// Receive results (outputs on stdout and messages on stderr) from a kernel
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

        tracing::debug!("Received on stdout: {}", &line);
        if !handle_line(&line, &mut output, &mut outputs) {
            break;
        }
    }

    // Attempt to parse each output as JSON into a `Node`, falling back to a string.
    let outputs: Vec<Node> = outputs
        .into_iter()
        .map(|output| -> Node {
            match serde_json::from_str(&output) {
                Ok(Node::Entity(..)) => {
                    // An `Entity` will get matched before an `Object` but is less useful (all
                    // the properties get dropped) so catch this and parse as an object.
                    let object =
                        serde_json::from_str::<Object>(&output).unwrap_or_else(|_| Object::new());
                    Node::Object(object)
                }
                Ok(node) => node,
                Err(..) => Node::String(output),
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

        tracing::debug!("Received on stderr: {}", &line);
        if !handle_line(&line, &mut message, &mut messages) {
            break;
        }
    }

    // Attempt to parse each message as JSON into a `CodeMessage`.
    let messages: Vec<CodeError> = messages
        .iter()
        .map(|message| -> CodeError {
            serde_json::from_str(message).unwrap_or_else(|_| CodeError {
                error_message: message.into(),
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
