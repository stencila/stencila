//! Workflow-context tools for LLM workflow state access.
//!
//! These tools let LLMs query and write workflow state on demand via tool
//! calls, rather than relying solely on prompt interpolation.

mod workflow_get_artifact;
mod workflow_get_context;
mod workflow_get_output;
mod workflow_get_run;
mod workflow_list_nodes;
mod workflow_set_context;
pub(crate) mod workflow_set_route;
mod workflow_store_artifact;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use stencila_db::rusqlite::Connection;

/// Register all workflow-context tools on an agent session.
///
/// Skips registration silently for CLI sessions (which don't support tools).
///
/// # Errors
///
/// Returns an error if tool registration fails on an API session.
#[allow(clippy::result_large_err)]
pub fn register_workflow_tools(
    session: &mut stencila_agents::session::AgentSession,
    conn: Arc<Mutex<Connection>>,
    run_id: String,
    context_writable: bool,
    artifacts_dir: PathBuf,
    workspace_root: PathBuf,
) -> stencila_agents::error::AgentResult<()> {
    let stencila_agents::session::AgentSession::Api(api_session) = session else {
        // CLI sessions don't support tool registration
        return Ok(());
    };

    let tools = [
        workflow_get_run::registered_tool(conn.clone(), run_id.clone()),
        workflow_get_context::registered_tool(conn.clone(), run_id.clone()),
        workflow_set_context::registered_tool(conn.clone(), run_id.clone(), context_writable),
        workflow_list_nodes::registered_tool(conn.clone(), run_id.clone()),
        workflow_get_output::registered_tool(conn.clone(), run_id.clone()),
        workflow_get_artifact::registered_tool(conn.clone(), run_id.clone()),
        workflow_store_artifact::registered_tool(conn, run_id, artifacts_dir, workspace_root),
    ];

    for tool in tools {
        api_session.register_tool(tool)?;
    }

    Ok(())
}
