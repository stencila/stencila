use kernel::{
    async_trait::async_trait,
    eyre::{bail, eyre, Result},
    stencila_schema::{CodeError, Node},
    Kernel, KernelStatus, KernelTrait, KernelType,
};
use serde::Serialize;
use std::{env, fs};
use tokio::{
    io::{BufReader, BufWriter},
    process::{Child, ChildStderr, ChildStdin, ChildStdout},
};

// Re-exports for the convenience of crates that implement `MicroKernelTrait`
pub use kernel;

/// The Unicode code point used as the separator between results
/// (both "outputs" on `stderr` and "messages" on `stderr`)
const RES_SEP: char = '\u{10ABBA}';

/// The Unicode code point used as the separator between individual
/// Microkernel "transactions".
const TRANS_SEP: char = '\u{10ACDC}';

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

    /// Is the microkernel available on the current machine?
    ///
    /// Returns `true` if the operating system is listed in `oses` and
    /// a runtime matching the semver requirements in `runtime` is found to be installed.
    pub async fn available(&self) -> bool {
        if !self.oses.contains(&std::env::consts::OS.to_string()) {
            return false;
        }
        let (name, semver) = &self.runtime;
        binaries::installed(name, semver).await
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
        use tokio::io::AsyncBufReadExt;
        use tokio::io::AsyncWriteExt;

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

        // Send code to the kernel
        tracing::debug!("Sending on stdin");
        let escaped = code.replace("\n", "\\n");
        if let Err(error) = stdin.write_all([&escaped, "\n"].concat().as_bytes()).await {
            self.status = KernelStatus::Failed;
            bail!("When writing code to kernel: {}", error)
        }
        if let Err(error) = stdin.flush().await {
            self.status = KernelStatus::Failed;
            bail!("When flushing code to kernel: {}", error)
        }

        // Capture outputs separating them as we go
        let mut output = String::new();
        let mut outputs = Vec::new();
        loop {
            let line = match stdout.lines().next_line().await {
                Ok(Some(line)) => line,
                Ok(None) => break,
                Err(error) => {
                    self.status = KernelStatus::Failed;
                    bail!("When receiving outputs from kernel: {}", error)
                }
            };

            tracing::debug!("Received on stdout: {}", line);
            if let Some(line) = line.strip_suffix(RES_SEP) {
                output.push_str(line);
                if !output.is_empty() {
                    outputs.push(output.clone());
                    output.clear();
                }
            } else if let Some(line) = line.strip_suffix(TRANS_SEP) {
                output.push_str(line);
                if !output.is_empty() {
                    outputs.push(output.clone());
                }
                break;
            } else {
                output.push_str(&line);
                output.push('\n');
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
        loop {
            let line = match stderr.lines().next_line().await {
                Ok(Some(line)) => line,
                Ok(None) => break,
                Err(error) => {
                    self.status = KernelStatus::Failed;
                    bail!("When receiving messages from kernel: {}", error)
                }
            };

            tracing::debug!("Received on stderr: {}", line);
            if let Some(line) = line.strip_suffix(RES_SEP) {
                message.push_str(line);
                if !message.is_empty() {
                    messages.push(message.clone());
                    message.clear();
                }
            } else if let Some(line) = line.strip_suffix(TRANS_SEP) {
                message.push_str(line);
                if !message.is_empty() {
                    messages.push(message.clone());
                }
                break;
            } else {
                message.push_str(&line);
                message.push('\n');
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
}
