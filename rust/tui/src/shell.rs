use std::sync::{Arc, Mutex};

use tokio::{process::Command, sync::oneshot};

/// The result of a completed shell command.
pub struct CommandResult {
    /// The captured stdout+stderr output.
    pub output: String,
    /// The process exit code (0 = success).
    pub exit_code: i32,
}

/// A shell command running asynchronously in the background.
///
/// The command is spawned via `tokio::process::Command` and its output is
/// collected in a shared buffer. The main loop polls for completion via
/// `try_take_result()` on each tick event.
pub struct RunningShellCommand {
    /// The original command string.
    command: String,
    /// Shared slot for the result. The spawned task writes `Some(result)` when done.
    result: Arc<Mutex<Option<CommandResult>>>,
    /// Sender to signal cancellation to the spawned task.
    cancel_tx: Option<oneshot::Sender<()>>,
}

impl RunningShellCommand {
    /// The command string that was submitted.
    #[cfg(test)]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Check if the command has finished and take the result.
    ///
    /// Returns `None` if still running.
    pub fn try_take_result(&mut self) -> Option<CommandResult> {
        self.result.lock().ok().and_then(|mut guard| guard.take())
    }

    /// Cancel the running command by killing the child process.
    ///
    /// Returns the command string. Output is not recoverable after cancellation
    /// because the pipe draining task is also cancelled.
    pub fn cancel(self) -> String {
        // Signal the spawned task to kill the child
        if let Some(tx) = self.cancel_tx {
            let _ = tx.send(());
        }
        self.command
    }
}

/// Build a platform-appropriate shell command.
///
/// On Unix, uses `sh -c <command>`. On Windows, uses `cmd /C <command>`.
fn build_shell_command(command: &str) -> Command {
    #[cfg(unix)]
    {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        cmd
    }
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(command);
        cmd
    }
}

/// Spawn a shell command asynchronously.
///
/// The command is run via the platform shell (`sh -c` on Unix, `cmd /C` on
/// Windows) so that shell features like pipes and redirects work. stdout and
/// stderr are captured separately and merged in the result.
pub fn spawn_command(command: String) -> RunningShellCommand {
    let result: Arc<Mutex<Option<CommandResult>>> = Arc::new(Mutex::new(None));
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    let result_clone = Arc::clone(&result);
    let cmd_clone = command.clone();

    tokio::spawn(async move {
        let child = build_shell_command(&cmd_clone)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                if let Ok(mut guard) = result_clone.lock() {
                    *guard = Some(CommandResult {
                        output: format!("Failed to spawn command: {e}"),
                        exit_code: -1,
                    });
                }
                return;
            }
        };

        // Take stdout/stderr handles so we can drain them concurrently with wait().
        // This prevents deadlock: if the child fills the OS pipe buffer (~64KB)
        // before exiting, it blocks on write. Draining concurrently ensures the
        // pipes are consumed while the process is still running.
        let child_stdout = child.stdout.take();
        let child_stderr = child.stderr.take();

        // Race between (wait + drain) and cancellation
        tokio::select! {
            (status, stdout_str, stderr_str) = async {
                let (status, stdout_str, stderr_str) = tokio::join!(
                    child.wait(),
                    read_stdout_pipe(child_stdout),
                    read_stderr_pipe(child_stderr),
                );
                (status, stdout_str, stderr_str)
            } => {
                let mut combined = stdout_str;
                if !stderr_str.is_empty() {
                    if !combined.is_empty() && !combined.ends_with('\n') {
                        combined.push('\n');
                    }
                    combined.push_str(&stderr_str);
                }
                // Trim trailing newline for cleaner display
                if combined.ends_with('\n') {
                    combined.pop();
                }

                let exit_code = match status {
                    Ok(s) => s.code().unwrap_or(-1),
                    Err(_) => -1,
                };

                if let Ok(mut guard) = result_clone.lock() {
                    *guard = Some(CommandResult {
                        output: combined,
                        exit_code,
                    });
                }
            }
            _ = cancel_rx => {
                // Cancellation requested — kill the child process
                let _ = child.kill().await;
            }
        }
    });

    RunningShellCommand {
        command,
        result,
        cancel_tx: Some(cancel_tx),
    }
}

/// Read all bytes from an optional stdout pipe handle and return as a string.
async fn read_stdout_pipe(pipe: Option<tokio::process::ChildStdout>) -> String {
    use tokio::io::AsyncReadExt;

    let Some(mut pipe) = pipe else {
        return String::new();
    };
    let mut buf = Vec::new();
    let _ = pipe.read_to_end(&mut buf).await;
    String::from_utf8_lossy(&buf).into_owned()
}

/// Read all bytes from an optional stderr pipe handle and return as a string.
async fn read_stderr_pipe(pipe: Option<tokio::process::ChildStderr>) -> String {
    use tokio::io::AsyncReadExt;

    let Some(mut pipe) = pipe else {
        return String::new();
    };
    let mut buf = Vec::new();
    let _ = pipe.read_to_end(&mut buf).await;
    String::from_utf8_lossy(&buf).into_owned()
}

// Tests use Unix shell commands (false, sleep, >&2 redirection) and are not
// portable to Windows. The runtime `build_shell_command` function handles
// Windows via `cmd /C`, but the test commands themselves are Unix-specific.
#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_echo_command() {
        let mut running = spawn_command("echo hello".to_string());
        assert_eq!(running.command(), "echo hello");

        // Poll until done
        let result = loop {
            if let Some(r) = running.try_take_result() {
                break r;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "hello");
    }

    #[tokio::test]
    async fn spawn_failing_command() {
        let mut running = spawn_command("false".to_string());

        let result = loop {
            if let Some(r) = running.try_take_result() {
                break r;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };

        assert_ne!(result.exit_code, 0);
    }

    #[tokio::test]
    async fn cancel_running_command() {
        let running = spawn_command("sleep 60".to_string());

        // Give it a moment to start
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let cmd = running.cancel();
        assert_eq!(cmd, "sleep 60");
    }

    #[tokio::test]
    async fn spawn_command_with_stderr() {
        let mut running = spawn_command("echo out && echo err >&2".to_string());

        let result = loop {
            if let Some(r) = running.try_take_result() {
                break r;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };

        assert_eq!(result.exit_code, 0);
        assert!(result.output.contains("out"));
        assert!(result.output.contains("err"));
    }
}
