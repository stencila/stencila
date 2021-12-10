use kernel::{
    async_trait::async_trait,
    eyre::{bail, eyre, Result},
    stencila_schema::{CodeError, Node},
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

/// The end of kernel startup, kernel is ready to process transactions.
const READY: char = '\u{10ACDC}';
const READY_ALT: &str = "<U+0010ACDC>";

/// The end of a result ("outputs" on `stderr` and "messages" on `stderr`).
const RESULT: char = '\u{10CB40}';
const RESULT_ALT: &str = "<U+0010CB40>";

/// The end of a transaction, kernel is ready for next transaction.
const TRANS: char = '\u{10ABBA}';
const TRANS_ALT: &str = "<U+0010ABBA>";

/// Fork the kernel
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

    /// Operating systems that the kernel will run on
    ///
    /// Possible OS names can be found here https://doc.rust-lang.org/std/env/consts/constant.OS.html
    oses: Vec<String>,

    /// The operating systems on which the kernel is forkable
    forkable: Vec<String>,

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

    /// The child process of the kernel
    #[serde(skip)]
    child: Option<Child>,

    /// The writer for the stdin stream of the child process
    #[serde(skip)]
    stdin: Option<BufWriter<ChildStdin>>,

    /// The reader for the stdout stream of the child process
    #[serde(skip)]
    stdout: Option<BufReader<ChildStdout>>,

    /// The reader for the stderr stream of the child process
    #[serde(skip)]
    stderr: Option<BufReader<ChildStderr>>,
}

impl MicroKernel {
    /// Create a new `MicroKernel`
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        languages: &[&str],
        oses: &[&str],
        forkable: &[&str],
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
            oses: oses.iter().map(|os| os.to_string()).collect(),
            forkable: forkable.iter().map(|os| os.to_string()).collect(),
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
        if !self.oses.contains(&std::env::consts::OS.to_string()) {
            return false;
        }
        let (name, semver) = &self.runtime;
        binaries::installed(name, semver).await
    }

    /// Is the kernel forkable on the current machine?
    async fn forkable(&self) -> bool {
        self.forkable.contains(&std::env::consts::OS.to_string())
    }

    /// Start the kernel
    ///
    /// An override of `KernelTrait::start` that searches for the preferred executable
    /// and runs it using specified commands, including the kernel script file if specified
    /// in the arguments.
    async fn start(&mut self) -> Result<()> {
        // Resolve the directory where kernels ar run
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

        self.child = Some(child);
        self.stdin = Some(BufWriter::new(stdin));
        self.stdout = Some(BufReader::new(stdout));
        self.stderr = Some(BufReader::new(stderr));
        self.status = KernelStatus::Starting;

        // Capture stdout until the READY flag
        let stdout = self.stdout.as_mut().unwrap();
        loop {
            match stdout.lines().next_line().await {
                Ok(Some(line)) => {
                    tracing::debug!("Received on stdout: {}", line);
                    if line.ends_with(READY) || line.ends_with(READY_ALT) {
                        break;
                    }
                }
                Ok(None) => bail!("Unexpected end of stdout"),
                Err(error) => bail!("When receiving stdout from kernel: {}", error),
            };
        }

        // Capture stderr until the READY flag and warn if any content
        let stderr = self.stderr.as_mut().unwrap();
        let mut err = String::new();
        loop {
            match stderr.lines().next_line().await {
                Ok(Some(line)) => {
                    tracing::debug!("Received on stderr: {}", line);
                    if let Some(line) = line
                        .strip_suffix(READY)
                        .or_else(|| line.strip_suffix(READY_ALT))
                    {
                        err.push_str(line);
                        err.push('\n');
                        break;
                    } else {
                        err.push_str(&line);
                        err.push('\n');
                    }
                }
                Ok(None) => bail!("Unexpected end of stderr"),
                Err(error) => bail!("When receiving stderr from kernel: {}", error),
            };
        }
        let err = err.trim();
        if !err.is_empty() {
            tracing::warn!(
                "While starting kernel `{}` got output on stderr: {}",
                self.name,
                err
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
            self.status = KernelStatus::Stopping;
            child.kill().await?;
            self.child = None;
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
        let stdin = self
            .stdin
            .as_mut()
            .expect("Kernel should have started and have stdin");
        let stdout = self
            .stdout
            .as_mut()
            .expect("Kernel should have started and have stdout");
        let stderr = self
            .stderr
            .as_mut()
            .expect("Kernel should have started and have stderr");

        // Send the code to the kernel
        let command = [&code.replace("\n", "\\n"), "\n"].concat();
        if let Err(error) = send_command(&command, stdin).await {
            self.status = KernelStatus::Failed;
            bail!(error)
        };

        // Receive outputs and messages
        receive_results(stdout, stderr).await
    }

    /// Fork the kernel and execute code in the fork
    #[cfg(not(target_os = "windows"))]
    async fn fork_exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)> {
        if !self.forkable().await {
            tracing::warn!(
                "Kernel `{}` is not forkable; executing in kernel itself",
                self.name
            );
            return self.exec(code).await;
        }

        let stdin = self
            .stdin
            .as_mut()
            .expect("Kernel should have started and have stdin");

        // Create pipes in a temporary directory (which gets cleaned up when dropped)
        use nix::{sys::stat, unistd::mkfifo};
        let pipes_dir = tempdir().unwrap();
        let stdout = pipes_dir.path().join("stdout.pipe");
        mkfifo(&stdout, stat::Mode::S_IRWXU)?;
        let stderr = pipes_dir.path().join("stderr.pipe");
        mkfifo(&stderr, stat::Mode::S_IRWXU)?;

        // Send code and pipes to the kernel
        let command = format!(
            "{}|{};{}{}\n",
            code.replace("\n", "\\n"),
            stdout.display(),
            stderr.display(),
            FORK
        );
        if let Err(error) = send_command(&command, stdin).await {
            self.status = KernelStatus::Failed;
            bail!(error)
        };

        // Open `stdout` and `stderr`. These calls will block until the child
        // process has opened the pipes for writing. So perhaps this should have a timeout
        // in case that fails.
        tracing::debug!("Waiting to open stdout");
        let stdout = File::open(stdout).await?;
        tracing::debug!("Waiting to open stderr");
        let stderr = File::open(stderr).await?;

        // Receive outputs and messages
        let mut stdout = BufReader::new(stdout);
        let mut stderr = BufReader::new(stderr);
        receive_results(&mut stdout, &mut stderr).await
    }
}

/// Send a command to a kernel on stdin
async fn send_command<W: AsyncWrite + Unpin>(
    command: &str,
    stdin: &mut BufWriter<W>,
) -> Result<()> {
    tracing::debug!("Sending command on stdin");
    if let Err(error) = stdin.write_all(command.as_bytes()).await {
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
        .iter()
        .map(|output| -> Node {
            serde_json::from_str(output).unwrap_or_else(|_| Node::String(output.clone()))
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
/// How the line is handled depends upon whether it has a result or transaction
/// separator at the end. Returns false at the end of a transaction.
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
        .strip_suffix(TRANS)
        .or_else(|| line.strip_suffix(TRANS_ALT))
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
