use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail};

use stencila_cli_utils::{AsFormat, Code, ToStdout, color_print::cstr, message};
use stencila_format::Format;

use crate::{
    MANAGED_CONFIG_KEYS, config,
    utils::{ConfigTarget, config_set, config_unset, config_value},
};

/// Manage Stencila configuration
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>

  <dim># Show the current configuration</dim>
  <b>stencila config</b>

  <dim># Show configuration as JSON</dim>
  <b>stencila config get</b> <c>--as</c> <g>json</g>

  <dim># Get a specific config value</dim>
  <b>stencila config get</b> <g>site.id</g>

  <dim># Set a value in the nearest stencila.toml</dim>
  <b>stencila config set</b> <g>site.id</g> <g>mysite123</g>

  <dim># Set a value in user config</dim>
  <b>stencila config set</b> <c>--user</c> <g>site.id</g> <g>mysite123</g>

  <dim># Set a value in local override file</dim>
  <b>stencila config set</b> <c>--local</c> <g>site.id</g> <g>mysite123</g>

  <dim># Remove a value</dim>
  <b>stencila config unset</b> <g>site.id</g>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Get(Get),
    Set(Set),
    Unset(Unset),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            // Default to showing the entire config
            return Get::default().run().await;
        };

        match command {
            Command::Get(get) => get.run().await,
            Command::Set(set) => set.run().await,
            Command::Unset(unset) => unset.run().await,
        }
    }
}

/// Get configuration value(s)
#[derive(Debug, Default, Args)]
#[command(after_long_help = GET_AFTER_LONG_HELP)]
struct Get {
    /// Config key in dot notation (e.g., `site.id`)
    ///
    /// If omitted, shows the entire configuration.
    /// Supports nested paths and array access (e.g., `packages[0].name`).
    key: Option<String>,

    /// Output format (toml, json, or yaml, default: toml)
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static GET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>

  <dim># Show entire configuration</dim>
  <b>stencila config get</b>

  <dim># Show as JSON</dim>
  <b>stencila config get</b> <c>--as</c> <g>json</g>

  <dim># Get a specific value</dim>
  <b>stencila config get</b> <g>site.id</g>

  <dim># Get nested value</dim>
  <b>stencila config get</b> <g>site.settings.theme</g>

  <dim># Get array element</dim>
  <b>stencila config get</b> <g>packages[0].name</g>
"
);

impl Get {
    async fn run(self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        let format = self.r#as.map(Into::into).unwrap_or(Format::Toml);

        if let Some(key) = self.key {
            // Get specific value using Figment's find_value()
            match config_value(&cwd, &key)? {
                Some(value) => {
                    Code::new_from(format, &value)?.to_stdout();
                }
                None => {
                    bail!("Config key `{}` not found", key);
                }
            }
        } else {
            // Get entire config
            let cfg = config(&cwd)?;

            // Check if config is empty (all fields are None)
            if cfg.site.is_none() && cfg.routes.is_none() {
                message(cstr!(
                    "💡 No configuration values are currently set.\n\n\
                    Use <b>stencila config set</> <g>key</> <g>value</> to set a value, \
                    or add a `stencila.toml` file."
                ));
            } else {
                Code::new_from(format, &cfg)?.to_stdout();
            }
        }

        Ok(())
    }
}

/// Set a configuration value
#[derive(Debug, Args)]
#[command(after_long_help = SET_AFTER_LONG_HELP)]
struct Set {
    /// Config key in dot notation (e.g., `site.id`)
    key: String,

    /// Value to set
    ///
    /// Values are automatically parsed as bool, number, or string.
    value: String,

    /// Set in user config (~/.config/stencila/stencila.toml)
    ///
    /// Creates the file if it doesn't exist.
    #[arg(long, conflicts_with = "local")]
    user: bool,

    /// Set in local override (stencila.local.yaml)
    ///
    /// Finds the nearest stencila.local.yaml or creates one in the current directory.
    /// Local overrides are typically not checked into version control.
    #[arg(long, conflicts_with = "user")]
    local: bool,
}

pub static SET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>

  <dim># Set in nearest stencila.toml (or create in CWD)</dim>
  <b>stencila config set</b> <g>site.id</g> <g>mysite123</g>

  <dim># Set in user config</dim>
  <b>stencila config set</b> <c>--user</c> <g>site.id</g> <g>mysite123</g>

  <dim># Set in local override</dim>
  <b>stencila config set</b> <c>--local</c> <g>site.id</g> <g>mysite123</g>

  <dim># Set nested value</dim>
  <b>stencila config set</b> <g>site.settings.theme</g> <g>dark</g>

  <dim># Set boolean</dim>
  <b>stencila config set</b> <g>site.settings.enabled</g> <g>true</g>

  <dim># Set number</dim>
  <b>stencila config set</b> <g>site.settings.port</g> <g>8080</g>
"
);

impl Set {
    async fn run(self) -> Result<()> {
        // Check if this key is managed by a specific command
        for managed in MANAGED_CONFIG_KEYS {
            if self.key == managed.key {
                bail!(
                    "The `{}` configuration should not be set directly.\n\n\
                    Please use the dedicated command instead: *{}*\n\n\
                    {}",
                    managed.key,
                    managed.command,
                    managed.reason
                );
            }
        }

        let target = if self.user {
            ConfigTarget::User
        } else if self.local {
            ConfigTarget::Local
        } else {
            ConfigTarget::Nearest
        };

        let config_file = config_set(&self.key, &self.value, target)?;

        message!("✅ Set `{}` in `{}`", self.key, config_file.display());

        Ok(())
    }
}

/// Remove a configuration value
#[derive(Debug, Args)]
#[command(after_long_help = UNSET_AFTER_LONG_HELP)]
struct Unset {
    /// Config key in dot notation (e.g., `site.id`)
    key: String,

    /// Remove from user config
    #[arg(long, conflicts_with = "local")]
    user: bool,

    /// Remove from local override
    #[arg(long, conflicts_with = "user")]
    local: bool,
}

pub static UNSET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>

  <dim># Remove from nearest stencila.toml</dim>
  <b>stencila config unset</b> <g>site.id</g>

  <dim># Remove from user config</dim>
  <b>stencila config unset</b> <c>--user</c> <g>site.id</g>

  <dim># Remove from local override</dim>
  <b>stencila config unset</b> <c>--local</c> <g>site.id</g>

  <dim># Remove nested value</dim>
  <b>stencila config unset</b> <g>site.settings.theme</g>
"
);

impl Unset {
    async fn run(self) -> Result<()> {
        let target = if self.user {
            ConfigTarget::User
        } else if self.local {
            ConfigTarget::Local
        } else {
            ConfigTarget::Nearest
        };

        let config_file = config_unset(&self.key, target)?;

        message!("🗑️ Removed `{}` from `{}`", self.key, config_file.display());

        Ok(())
    }
}
