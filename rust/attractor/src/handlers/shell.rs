//! Shell handler (§4.10).
//!
//! Executes shell commands specified in node attributes. Captures
//! stdout and stderr, with optional timeout support. Expands pipeline
//! context variables (`$last_output`, `$last_stage`, etc.) in the
//! command string before execution, and stores the trimmed stdout as
//! `last_output` so downstream conditions and stages can reference it.

use async_trait::async_trait;

use super::shared::{build_output_outcome, expand_runtime_variables};
use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::Outcome;

/// Handler for shell command nodes.
///
/// Reads the `shell_command` attribute from the node and executes it
/// via `sh -c`. Pipeline context variables are expanded in the command
/// before execution. An optional `timeout` attribute (duration) limits
/// execution time. On success the trimmed stdout is stored as
/// `last_output` (and `last_output_full`) so subsequent pipeline stages
/// and condition expressions can reference the shell output.
pub struct ShellHandler;

#[async_trait]
impl Handler for ShellHandler {
    async fn execute(
        &self,
        node: &Node,
        context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        let Some(raw_command) = node.get_str_attr("shell_command") else {
            return Ok(Outcome::fail(format!(
                "node '{}' has type 'shell' but no 'shell_command' attribute",
                node.id
            )));
        };

        let command = expand_runtime_variables(raw_command, context);

        let timeout = node.get_attr("timeout").and_then(|v| match v {
            crate::graph::AttrValue::Duration(d) => Some(d.inner()),
            crate::graph::AttrValue::String(s) => crate::types::Duration::from_spec_str(s)
                .ok()
                .map(crate::types::Duration::inner),
            _ => None,
        });

        let result = run_command(&command, timeout).await;

        match result {
            Ok(output) => {
                if output.success {
                    let stdout = output.stdout.trim().to_string();
                    let mut outcome = build_output_outcome(&node.id, &stdout, context);
                    outcome.context_updates.insert(
                        "shell.output".to_string(),
                        serde_json::Value::String(stdout),
                    );
                    Ok(outcome)
                } else {
                    Ok(Outcome::fail(format!(
                        "Command exited with non-zero status: {}",
                        output.stderr
                    )))
                }
            }
            Err(e) => Ok(Outcome::fail(format!("Command execution failed: {e}"))),
        }
    }
}

struct CommandOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

/// Collect the full output of a child process, optionally with a timeout.
///
/// Drains stdout and stderr concurrently with the wait to avoid pipe
/// backpressure deadlocks (the OS pipe buffer is ~64 KB; a child that
/// fills it blocks until the parent reads).
async fn run_command(
    command: &str,
    timeout: Option<std::time::Duration>,
) -> AttractorResult<CommandOutput> {
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Take the pipes before awaiting — we drain them concurrently with
    // the wait so the child never blocks on a full pipe buffer.
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let collect_future = async {
        let (status, stdout, stderr) =
            tokio::join!(child.wait(), read_pipe(stdout_pipe), read_pipe(stderr_pipe),);
        let status = status?;
        Ok(CommandOutput {
            success: status.success(),
            stdout,
            stderr,
        })
    };

    if let Some(duration) = timeout {
        if let Ok(result) = tokio::time::timeout(duration, collect_future).await {
            result
        } else {
            // Kill the child process to avoid leaking long-running subprocesses.
            // Ignore kill errors — the process may have already exited.
            let _ = child.kill().await;
            Ok(CommandOutput {
                success: false,
                stdout: String::new(),
                stderr: format!("Command timed out after {duration:?}"),
            })
        }
    } else {
        collect_future.await
    }
}

async fn read_pipe<R: tokio::io::AsyncRead + Unpin>(pipe: Option<R>) -> String {
    use tokio::io::AsyncReadExt;
    let Some(mut reader) = pipe else {
        return String::new();
    };
    let mut buf = Vec::new();
    let _ = reader.read_to_end(&mut buf).await;
    String::from_utf8_lossy(&buf).to_string()
}
