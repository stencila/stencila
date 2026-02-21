//! Workflow-context tools for LLM pipeline state access.
//!
//! These tools let LLMs query and write pipeline state on demand via tool
//! calls, rather than relying solely on prompt interpolation.

mod get_artifact;
mod get_node_output;
mod get_workflow_context;
mod get_workflow_run;
mod list_completed_nodes;
mod set_workflow_context;
mod store_artifact;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// Register all workflow-context tools on an agent session.
///
/// Skips registration silently for CLI sessions (which don't support tools).
///
/// # Errors
///
/// Returns an error if tool registration fails on an API session.
#[allow(clippy::result_large_err)]
pub fn register_workflow_tools(
    session: &mut stencila_agents::agent_session::AgentSession,
    conn: Arc<Mutex<Connection>>,
    run_id: String,
    context_writable: bool,
    artifacts_dir: PathBuf,
    workspace_root: PathBuf,
) -> stencila_agents::error::AgentResult<()> {
    let stencila_agents::agent_session::AgentSession::Api(api_session) = session else {
        // CLI sessions don't support tool registration
        return Ok(());
    };

    let tools = [
        get_workflow_run::registered_tool(conn.clone(), run_id.clone()),
        get_workflow_context::registered_tool(conn.clone(), run_id.clone()),
        set_workflow_context::registered_tool(conn.clone(), run_id.clone(), context_writable),
        list_completed_nodes::registered_tool(conn.clone(), run_id.clone()),
        get_node_output::registered_tool(conn.clone(), run_id.clone()),
        get_artifact::registered_tool(conn.clone(), run_id.clone()),
        store_artifact::registered_tool(conn, run_id, artifacts_dir, workspace_root),
    ];

    for tool in tools {
        api_session.register_tool(tool)?;
    }

    Ok(())
}
