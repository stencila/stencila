use std::sync::{Arc, Mutex};

use tokio::{process::Command, sync::oneshot};

/// Match the content_width calculation in ui/messages.rs:
/// NUM_GUTTER (3) + sidebar char (1) + space (1) = 5 columns of chrome.
const TUI_CHROME: u16 = 5;

/// The result of a completed shell command.
pub struct CommandResult {
    /// The captured stdout+stderr output.
    pub output: String,
    /// The process exit code (0 = success).
    pub exit_code: i32,
}

/// Whether TUI rendering env vars should be applied to this shell command.
///
/// This is intentionally a curated list to avoid changing behavior for arbitrary
/// user commands. Currently, only commands that start with `stencila` are
/// opted-in.
fn should_apply_tui_env(command: &str) -> bool {
    command.trim_start().starts_with("stencila")
}

/// Compute the effective content width used by the TUI message area.
fn tui_columns() -> Option<String> {
    crossterm::terminal::size()
        .ok()
        .map(|(w, _)| w.saturating_sub(TUI_CHROME).max(1).to_string())
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
    let cmd_clone = command.clone();
    let apply_tui_env = should_apply_tui_env(&command);
    let columns = if apply_tui_env {
        tui_columns().unwrap_or_default()
    } else {
        String::new()
    };

    spawn_child(command, move || {
        let mut cmd = build_shell_command(&cmd_clone);

        if apply_tui_env {
            cmd.env("FORCE_COLOR", "1");
            if !columns.is_empty() {
                cmd.env("COLUMNS", &columns);
            }
        }

        cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
    })
}

/// Spawn a CLI command as a direct process (not via shell).
///
/// Unlike `spawn_command()` which uses `sh -c`, this takes an explicit
/// program and argv vector and spawns the process directly. This avoids
/// shell quoting issues for CLI passthrough commands.
///
/// Sets `FORCE_COLOR=1` so the CLI emits ANSI colors even though stdout is
/// a pipe, and `COLUMNS` to the available content width (terminal width
/// minus the TUI gutter, sidebar, and padding) so table output fits
/// without wrapping.
pub fn spawn_command_argv(
    program: String,
    args: Vec<String>,
    display: String,
) -> RunningShellCommand {
    let columns = tui_columns().unwrap_or_default();

    spawn_child(display, move || {
        let mut cmd = Command::new(&program);
        cmd.args(&args)
            .env("FORCE_COLOR", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if !columns.is_empty() {
            cmd.env("COLUMNS", &columns);
        }

        cmd.spawn()
    })
}

/// Shared spawn infrastructure used by both `spawn_command` and `spawn_command_argv`.
///
/// `display` is the human-readable command string stored in the result.
/// `build` is a closure that configures and spawns the `tokio::process::Child`.
fn spawn_child(
    display: String,
    build: impl FnOnce() -> std::io::Result<tokio::process::Child> + Send + 'static,
) -> RunningShellCommand {
    let result: Arc<Mutex<Option<CommandResult>>> = Arc::new(Mutex::new(None));
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    let result_clone = Arc::clone(&result);

    tokio::spawn(async move {
        let mut child = match build() {
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
        command: display,
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

    #[tokio::test]
    async fn spawn_argv_echo() {
        let mut running = spawn_command_argv(
            "echo".to_string(),
            vec!["hello world".to_string()],
            "echo hello world".to_string(),
        );
        assert_eq!(running.command(), "echo hello world");

        let result = loop {
            if let Some(r) = running.try_take_result() {
                break r;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };

        assert_eq!(result.exit_code, 0);
        // echo with argv passes the whole argument including space
        assert_eq!(result.output, "hello world");
    }

    #[tokio::test]
    async fn spawn_argv_preserves_args_with_spaces() {
        // Verify that args with spaces are passed correctly (not re-split)
        let mut running = spawn_command_argv(
            "printf".to_string(),
            vec!["%s\n".to_string(), "arg with spaces".to_string()],
            "printf '%s\\n' 'arg with spaces'".to_string(),
        );

        let result = loop {
            if let Some(r) = running.try_take_result() {
                break r;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        };

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.output, "arg with spaces");
    }

    #[test]
    fn apply_tui_env_for_curated_shell_commands() {
        assert!(should_apply_tui_env("stencila"));
        assert!(should_apply_tui_env("stencila kernels list"));
        assert!(should_apply_tui_env("   stencila --help"));

        assert!(!should_apply_tui_env("echo stencila"));
        assert!(!should_apply_tui_env("./stencila --help"));
        assert!(!should_apply_tui_env("STENCILA --help"));
    }
}
