use std::{fs::write, path::Path, process::Stdio};

use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, OptionExt, Result},
        itertools::Itertools,
        serde_json,
        tokio::{
            fs::File,
            io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter},
            process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
        },
        tracing,
    },
    schema::{ExecutionError, Node, Variable},
};
use which::which;

// Re-exports for the convenience of internal crates implementing
// the `Microkernel` trait
pub use kernel::{
    common, format, schema, Kernel, KernelAvailability, KernelForking, KernelInstance, KernelStatus,
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
        which(&self.executable_name()).is_ok()
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

    /// An implementation of `Kernel::create_instance` for microkernels
    fn microkernel_instance(&self) -> Result<MicrokernelInstance> {
        // Assign an id
        let id = self.id(); // TODO make this unique

        // Get the path to the executable, failing early if it can not be found
        let executable_path = which(self.executable_name())?;

        // Always write the script file, even if it already exists, to allow for changes
        // to the microkernel's script
        let kernels_dir = app::get_app_dir(app::DirType::Kernels, true)?;
        let script_file = kernels_dir.join(self.id());
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

        Ok(MicrokernelInstance {
            id,
            command,
            status: KernelStatus::Pending,
            child: None,
            pid: 0,
            input: None,
            output: None,
            errors: None,
        })
    }
}

/// An instance of a microkernel
pub struct MicrokernelInstance {
    /// The id of the microkernel instance
    id: String,

    command: Command,

    /// The status of the microkernel instance
    status: KernelStatus,

    /// The child process (for main processes only, not forks)
    child: Option<Child>,

    /// The process identifier
    pid: u32,

    /// The input stream for the microkernel instance
    input: Option<MicrokernelInput>,

    /// The output stream for the microkernel instance
    output: Option<MicrokernelOutput>,

    /// The error stream for the microkernel instance
    errors: Option<MicrokernelErrors>,
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
    File(BufReader<File>),
}

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
    /// Sent by Rust to signal the start of a `list` task
    List,
    /// Sent by Rust to signal the start of a `get` task
    Get,
    /// Sent by Rust to signal the start of a `set` task
    Set,
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
            Exec => "\u{10B522}",
            Eval => "\u{1010CC}",
            Fork => "\u{10DE70}",
            List => "\u{10C155}",
            Get => "\u{10A51A}",
            Set => "\u{107070}",
            End => "\u{10CB40}",
        }
    }
}

#[async_trait]
impl KernelInstance for MicrokernelInstance {
    fn id(&self) -> String {
        self.id.clone()
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(self.status)
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        // Spawn the binary in the directory with stdin, stdout and stderr piped to/from it
        let mut child = self.command.current_dir(directory).spawn()?;

        self.pid = child
            .id()
            .ok_or_eyre("Microkernel child process has no PID")?;

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

        // Emit any startup warnings and clear streams
        let stdin = BufWriter::new(stdin);
        let mut stdout = BufReader::new(stdout);
        let mut stderr = BufReader::new(stderr);
        startup_warnings(&mut stdout, &mut stderr).await;

        // Create stream readers and writers
        self.input = Some(MicrokernelInput::Standard(stdin));
        self.output = Some(MicrokernelOutput::Standard(stdout));
        self.errors = Some(MicrokernelErrors::Standard(stderr));

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(child) = self.child.as_mut() {
            // For main kernels
            tracing::debug!("Killing kernel with pid `{:?}`", self.pid);
            child.kill().await?;
            self.child = None;
        }

        self.status = KernelStatus::Stopped;

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        self.send_receive(MicrokernelFlag::Exec, code).await
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        self.send_receive(MicrokernelFlag::Eval, code).await
    }

    async fn fork(&mut self, _code: &str) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        bail!("Not yet implemented")
    }

    async fn list(&mut self) -> Result<Vec<Variable>> {
        let (nodes, messages) = self.send_receive(MicrokernelFlag::List, "").await?;

        if !messages.is_empty() {
            bail!(
                "While listing variables in microkernel `{}`: {}",
                self.id(),
                messages.iter().map(|message| message.to_string()).join("")
            )
        }

        nodes
            .into_iter()
            .map(|node| match node {
                Node::Variable(var) => Ok(var),
                _ => bail!("Expected `Variable`s got `{}`", node.to_string()),
            })
            .collect::<Result<Vec<_>>>()
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        let (mut nodes, messages) = self.send_receive(MicrokernelFlag::Get, name).await?;

        if !messages.is_empty() {
            bail!(
                "While getting variable `{name}` in microkernel `{}`: {}",
                self.id(),
                messages.iter().map(|message| message.to_string()).join("")
            )
        }

        let node = if nodes.is_empty() {
            None
        } else {
            Some(nodes.swap_remove(0))
        };

        Ok(node)
    }

    async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        let parts = &[
            name,
            MicrokernelFlag::Line.as_unicode(),
            &serde_json::to_string(value)?,
        ]
        .concat();

        self.send(MicrokernelFlag::Set, parts).await
    }
}

impl MicrokernelInstance {
    /// Send a task to the microkernel instance and receive results
    async fn send_receive(
        &mut self,
        flag: MicrokernelFlag,
        code: &str,
    ) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        self.send(flag, code).await?;
        self.receive().await
    }

    /// Send a task to this microkernel instance
    async fn send(&mut self, flag: MicrokernelFlag, code: &str) -> Result<()> {
        let Some(input) = self.input.as_mut() else {
            bail!("Microkernel `{}` has not been started yet!", self.id());
        };

        match input {
            MicrokernelInput::Standard(input) => send_task(flag, code, input).await,
            MicrokernelInput::Pipe(input) => send_task(flag, code, input).await,
        }
    }

    /// Receive outputs and messages from this microkernel instance
    async fn receive(&mut self) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
        let (Some(output),Some(errors)) = (self.output.as_mut(),self.errors.as_mut()) else {
            bail!("Microkernel has not been started yet!");
        };

        match (output, errors) {
            (MicrokernelOutput::Standard(output), MicrokernelErrors::Standard(errors)) => {
                receive_results(output, errors).await
            }
            (MicrokernelOutput::Pipe(output), MicrokernelErrors::File(errors)) => {
                receive_results(output, errors).await
            }
            _ => unreachable!(),
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
                    .map(|message| message.error_message)
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
        &flag.as_unicode(),
        code.replace('\n', MicrokernelFlag::Line.as_unicode())
            .as_str(),
        "\n",
    ]
    .concat();

    tracing::trace!("Sending task on writer");
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
) -> Result<(Vec<Node>, Vec<ExecutionError>)> {
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
    let messages = items.into_iter().map(ExecutionError::new).collect_vec();

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
