use std::{collections::HashSet, sync::Arc};

use eyre::{Result, bail};
use stencila_cli_utils::{Code, ToStdout};
use stencila_format::Format;
use stencila_mcp::cli::Codemode;

pub async fn run(args: Codemode) -> Result<()> {
    let dir = args.dir.canonicalize().unwrap_or(args.dir);
    let all_configs = stencila_mcp::discover(&dir);

    let configs: Vec<stencila_mcp::McpServerConfig> = if args.server.is_empty() {
        all_configs.into_iter().filter(|c| c.enabled).collect()
    } else {
        let known_ids: HashSet<&str> =
            all_configs.iter().map(|c| c.id.as_str()).collect();

        for id in &args.server {
            if !known_ids.contains(id.as_str()) {
                bail!(
                    "Unknown MCP server `{id}`. Use `stencila mcp list` to see available servers."
                );
            }
        }

        let requested: HashSet<&str> =
            args.server.iter().map(String::as_str).collect();

        all_configs
            .into_iter()
            .filter(|c| requested.contains(c.id.as_str()) && c.enabled)
            .collect()
    };

    if configs.is_empty() {
        if args.server.is_empty() {
            eprintln!("No MCP servers available. Use `stencila mcp add` to add one.");
            return Ok(());
        }
        bail!("No enabled servers matched the requested IDs.");
    }

    let pool = Arc::new(stencila_mcp::ConnectionPool::new(configs));
    let errors = pool.connect_all().await;

    if !errors.is_empty() {
        pool.start_shutdown();
        let (id, err) = &errors[0];
        bail!("Failed to connect to MCP server `{id}`: {err}");
    }

    // Collect connected servers in deterministic (sorted by ID) order
    let mut servers: Vec<Arc<dyn stencila_mcp::McpServer>> = pool
        .connected_servers()
        .await
        .into_iter()
        .map(|s| s as Arc<dyn stencila_mcp::McpServer>)
        .collect();
    servers.sort_by(|a, b| a.server_id().cmp(b.server_id()));

    let declarations = stencila_codemode::generate_declarations(&servers).await?;

    Code::new(Format::JavaScript, &declarations).to_stdout();

    pool.start_shutdown();

    Ok(())
}
