//! Tool handler (§4.10).
//!
//! Executes shell commands specified in node attributes. Captures
//! stdout and stderr, with optional timeout support.

use std::path::Path;

use async_trait::async_trait;
use indexmap::IndexMap;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::Outcome;

/// Handler for tool (shell command) nodes.
///
/// Reads the `tool_command` attribute from the node and executes it
/// via `sh -c`. An optional `timeout` attribute (duration) limits
/// execution time.
pub struct ToolHandler;

#[async_trait]
impl Handler for ToolHandler {
    async fn execute(
        &self,
        node: &Node,
        _context: &Context,
        _graph: &Graph,
        _logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        let Some(command) = node.get_str_attr("tool_command") else {
            return Ok(Outcome::fail(format!(
                "node '{}' has type 'tool' but no 'tool_command' attribute",
                node.id
            )));
        };

        let timeout = node.get_attr("timeout").and_then(|v| match v {
            crate::graph::AttrValue::Duration(d) => Some(d.inner()),
            crate::graph::AttrValue::String(s) => crate::types::Duration::from_spec_str(s)
                .ok()
                .map(crate::types::Duration::inner),
            _ => None,
        });

        let result = run_command(command, timeout).await;

        match result {
            Ok(output) => {
                if output.success {
                    let mut outcome = Outcome::success();
                    outcome.context_updates = IndexMap::new();
                    outcome.context_updates.insert(
                        "tool.output".to_string(),
                        serde_json::Value::String(output.stdout),
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
