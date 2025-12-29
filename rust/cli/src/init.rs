use clap::Parser;
use eyre::Result;

use stencila_ask::ask;
use stencila_cli_utils::message;
use stencila_cloud::{create_workspace_watch, ensure_workspace};
use stencila_config::{ConfigTarget, config, config_set};

/// Initialize a workspace with stencila.toml configuration
#[derive(Debug, Parser)]
#[command(after_long_help = stencila_config::cli::INIT_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(flatten)]
    inner: stencila_config::cli::Init,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Capture directory before running inner init (which consumes self.inner)
        let dir = self.inner.dir.clone();

        // Run the base init from stencila_config
        self.inner.run().await?;

        // Canonicalize the directory
        let dir = dir.canonicalize()?;

        // Check if workspace already has a watch
        let cfg = config(&dir)?;
        if let Some(workspace) = &cfg.workspace
            && workspace.watch.is_some()
        {
            // Already watching, skip prompt
            return Ok(());
        }

        // Prompt user about workspace watch
        let answer = ask("Enable workspace watch? (auto-publish on git push)").await?;
        if !answer.is_yes() {
            return Ok(());
        }

        // Attempt to create workspace watch
        match try_setup_workspace_watch(&dir).await {
            Ok(()) => message!("👁️ Workspace watch enabled."),
            Err(e) => {
                message!("ℹ️ Could not enable workspace watch: {e}");
                message!("   You can enable it later with: stencila watch");
            }
        }

        Ok(())
    }
}

async fn try_setup_workspace_watch(dir: &std::path::Path) -> Result<()> {
    let (workspace_id, _) = ensure_workspace(dir).await?;
    let response = create_workspace_watch(&workspace_id).await?;
    config_set("workspace.watch", &response.id, ConfigTarget::Nearest)?;
    Ok(())
}
