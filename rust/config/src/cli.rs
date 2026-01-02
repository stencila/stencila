use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail};
use stencila_ask::{Answer, ask_with_default, input, select, setup_defaults};
use stencila_cli_utils::{AsFormat, Code, ToStdout, color_print::cstr, message};
use stencila_format::Format;
use tokio::fs::create_dir_all;

use crate::{
    MANAGED_CONFIG_KEYS, config,
    init::{RepoAnalysis, RepoAnalyzer},
    utils::{ConfigTarget, config_set, config_unset, config_value},
};

/// Initialize a workspace with stencila.toml configuration
#[derive(Debug, Parser)]
#[command(after_long_help = INIT_AFTER_LONG_HELP)]
pub struct Init {
    /// The workspace directory to initialize
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Accept all defaults without prompting
    ///
    /// Useful for non-interactive/automated environments.
    #[arg(long, short)]
    yes: bool,

    /// Site root directory (skip interactive prompt)
    #[arg(long)]
    root: Option<String>,

    /// Home page file (skip interactive prompt)
    #[arg(long)]
    home: Option<String>,

    /// Output formats for executable documents (comma-separated)
    ///
    /// Applies to .smd, .qmd, .myst, .tex files.
    /// Example: --outputs html,pdf
    #[arg(long, value_delimiter = ',')]
    outputs: Option<Vec<String>>,
}

pub static INIT_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>

  <dim># Initialize current directory with interactive prompts</dim>
  <b>stencila init</>

  <dim># Initialize with all defaults (non-interactive)</dim>
  <b>stencila init --yes</>

  <dim># Initialize a specific directory</dim>
  <b>stencila init</> <g>./my-project</>

  <dim># Initialize with specific options</dim>
  <b>stencila init --root docs --home index.md</>

  <dim># Initialize with outputs for executable documents</dim>
  <b>stencila init --outputs docx,pdf</>

<bold><b>Note</b></bold>
  This creates a stencila.toml configuration file with site settings,
  routes, and output configurations based on repository analysis.
"
);

/// Configuration collected during init
#[derive(Debug, Default)]
struct InitConfig {
    workspace_id: Option<String>,
    site_root: Option<String>,
    home_page: Option<String>,
    exclude_patterns: Vec<String>,
    output_formats: Vec<String>,
    executable_docs: Vec<std::path::PathBuf>,
}

impl Init {
    #[tracing::instrument]
    pub async fn run(self) -> Result<()> {
        // Setup defaults provider for non-interactive mode
        if self.yes {
            setup_defaults().await?;
        }

        // Create directory if it doesn't exist
        if !self.dir.exists() {
            create_dir_all(&self.dir).await?;
        }

        let dir = self.dir.canonicalize()?;
        let config_path = dir.join("stencila.toml");

        // Check if config already exists
        if config_path.exists() {
            let answer =
                ask_with_default("stencila.toml already exists. Overwrite?", Answer::No).await?;
            if !answer.is_yes() {
                message!("🚫 Initialization cancelled");
                return Ok(());
            }
        }

        // Analyze repository
        message!("🔍 Analyzing repository...");
        let analyzer = RepoAnalyzer::new(&dir);
        let analysis = analyzer.analyze()?;

        // Report detected project types
        if !analysis.project_types.is_empty() {
            let types: Vec<String> = analysis
                .project_types
                .iter()
                .map(|t| format!("{t:?}"))
                .collect();
            message!("📦 Detected: {}", types.join(", "));
        }

        // Collect configuration interactively or from CLI args
        let config = self.collect_configuration(&dir, &analysis).await?;

        // Write configuration
        self.write_config(&dir, &config).await?;

        // Print summary
        self.print_summary(&config);

        Ok(())
    }

    #[allow(clippy::field_reassign_with_default)]
    async fn collect_configuration(
        &self,
        dir: &std::path::Path,
        analysis: &RepoAnalysis,
    ) -> Result<InitConfig> {
        let mut config = InitConfig::default();

        // Check for workspace ID from environment variable
        config.workspace_id = std::env::var("STENCILA_WORKSPACE_ID").ok();

        // Site root selection - allow (none) to skip site configuration
        let mut skip_site = false;
        config.site_root = if let Some(ref root) = self.root {
            if root == "none" {
                skip_site = true;
                None
            } else {
                Some(root.clone())
            }
        } else if !analysis.suggested_roots.is_empty() {
            let mut options = vec![];
            options.extend(analysis.suggested_roots.clone());
            options.push("none (skip site setup)".to_string());
            options.push("enter custom path".to_string());

            let idx = select("Select site root directory", &options).await?;

            if idx == options.len() - 2 {
                // User chose none
                skip_site = true;
                None
            } else if idx == options.len() - 1 {
                // User chose custom
                Some(input("Enter custom root path").await?)
            } else {
                let selected = &options[idx];
                if selected == "." {
                    None // Root directory, no need to set explicitly
                } else {
                    Some(selected.clone())
                }
            }
        } else {
            None
        };

        // Determine the actual site root path for exclusion filtering
        let site_root_path = config
            .site_root
            .as_ref()
            .map(|r| dir.join(r))
            .unwrap_or_else(|| dir.to_path_buf());

        // Home page selection - only if site setup wasn't skipped
        if !skip_site && (!analysis.home_page_candidates.is_empty() || self.home.is_some()) {
            config.home_page = if let Some(ref home) = self.home {
                if home == "none" {
                    None
                } else {
                    Some(home.clone())
                }
            } else {
                let mut options = vec!["(none)".to_string()];
                options.extend(
                    analysis
                        .home_page_candidates
                        .iter()
                        .map(|p| p.to_string_lossy().to_string()),
                );
                options.push("(enter custom path)".to_string());

                let idx = select("Select home page", &options).await?;

                if idx == 0 {
                    // User chose none
                    None
                } else if idx == options.len() - 1 {
                    // User chose custom
                    Some(input("Enter custom home page path").await?)
                } else {
                    Some(options[idx].clone())
                }
            };
        }

        // Filter exclusion patterns to only include those relevant to files in site root
        // Skip if site setup was skipped
        if !skip_site {
            let analyzer = RepoAnalyzer::new(dir);
            config.exclude_patterns =
                analyzer.suggest_excludes_for_root(&analysis.project_types, &site_root_path)?;
        }

        // Outputs for executable documents
        if !analysis.executable_docs.is_empty() {
            config.output_formats = if let Some(ref outputs) = self.outputs {
                // Handle --outputs none
                if outputs.len() == 1 && outputs[0] == "none" {
                    vec![]
                } else {
                    outputs.clone()
                }
            } else {
                // Offer output format options including none
                let format_options = vec![
                    "pdf".to_string(),
                    "docx".to_string(),
                    "pdf and docx".to_string(),
                    "none (skip outputs)".to_string(),
                ];

                let idx = select(
                    "Select output format(s) for executable documents",
                    &format_options,
                )
                .await?;

                match idx {
                    0 => vec!["pdf".to_string()],                     // pdf only
                    1 => vec!["docx".to_string()],                    // docx only
                    2 => vec!["pdf".to_string(), "docx".to_string()], // both
                    3 => vec![],                                      // none
                    _ => vec![],
                }
            };

            if !config.output_formats.is_empty() {
                config.executable_docs = analysis.executable_docs.clone();
            }
        }

        Ok(config)
    }

    async fn write_config(&self, dir: &std::path::Path, config: &InitConfig) -> Result<()> {
        use std::collections::HashSet;
        use std::fmt::Write;

        let config_path = dir.join("stencila.toml");
        let mut content = String::new();

        // [workspace] section - always first
        if let Some(ref id) = config.workspace_id {
            writeln!(content, "[workspace]")?;
            writeln!(content, "id = \"{}\"", id)?;
            writeln!(content)?;
        }

        // [site] section
        let has_site_config = config.site_root.is_some()
            || !config.exclude_patterns.is_empty()
            || config.home_page.is_some();

        if has_site_config {
            writeln!(content, "[site]")?;

            if let Some(ref root) = config.site_root {
                writeln!(content, "root = \"{}\"", root)?;
            }

            if !config.exclude_patterns.is_empty() {
                writeln!(content, "exclude = [")?;
                for pattern in &config.exclude_patterns {
                    writeln!(content, "  \"{}\",", pattern)?;
                }
                writeln!(content, "]")?;
            }

            writeln!(content)?;
        }

        // [site.routes] section
        if let Some(ref home) = config.home_page {
            writeln!(content, "[site.routes]")?;
            writeln!(content, "\"/\" = \"{}\"", home)?;
            writeln!(content)?;
        }

        // [outputs] section - limit to 10 outputs
        const MAX_OUTPUTS: usize = 10;
        if !config.output_formats.is_empty() && !config.executable_docs.is_empty() {
            writeln!(content, "[outputs]")?;

            // Track output keys to prevent duplicates
            let mut seen_keys = HashSet::new();
            let mut output_count = 0;

            'outer: for doc in &config.executable_docs {
                // Use full path without extension as the output key
                // e.g., "docs/report.smd" -> "docs/report.pdf"
                let path_without_ext = doc.with_extension("");
                let output_base = path_without_ext.to_string_lossy();
                let source = doc.to_string_lossy();

                for format in &config.output_formats {
                    if output_count >= MAX_OUTPUTS {
                        break 'outer;
                    }
                    let key = format!("{}.{}", output_base, format);
                    // Only write if we haven't seen this key before
                    if seen_keys.insert(key.clone()) {
                        writeln!(content, "\"{}\" = \"{}\"", key, source)?;
                        output_count += 1;
                    }
                }
            }
        }

        tokio::fs::write(&config_path, content).await?;

        Ok(())
    }

    fn print_summary(&self, config: &InitConfig) {
        message!("✅ Created stencila.toml");

        if let Some(ref id) = config.workspace_id {
            message!("   🔗 Workspace: {id}");
        }

        if let Some(ref root) = config.site_root {
            message!("   📁 Site root: {root}");
        }

        if let Some(ref home) = config.home_page {
            message!("   🏠 Home page: {home} -> /");
        }

        if !config.exclude_patterns.is_empty() {
            message!(
                "   🚫 Exclusions: {} patterns",
                config.exclude_patterns.len()
            );
        }

        if !config.output_formats.is_empty() && !config.executable_docs.is_empty() {
            let total_possible = config.executable_docs.len() * config.output_formats.len();
            let actual = total_possible.min(10);
            if total_possible > 10 {
                message!(
                    "   📄 Outputs: {} (limited from {} docs × {:?})",
                    actual,
                    config.executable_docs.len(),
                    config.output_formats
                );
            } else {
                message!(
                    "   📄 Outputs: {} docs -> {:?}",
                    config.executable_docs.len(),
                    config.output_formats
                );
            }
        }
    }
}

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

  <dim># Check config validity</dim>
  <b>stencila config check</b>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    Get(Get),
    Set(Set),
    Unset(Unset),
    Check(Check),
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
            Command::Check(check) => check.run().await,
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
            if cfg.site.is_none()
                && cfg.workspace.is_none()
                && cfg.remotes.is_none()
                && cfg.outputs.is_none()
            {
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

/// Check configuration validity
#[derive(Debug, Default, Args)]
#[command(after_long_help = CHECK_AFTER_LONG_HELP)]
struct Check {
    /// Directory to check configuration for
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    dir: PathBuf,
}

pub static CHECK_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>

  <dim># Check config in current directory</dim>
  <b>stencila config check</b>

  <dim># Check config in a specific directory</dim>
  <b>stencila config check</b> <g>./my-project</g>
"
);

impl Check {
    async fn run(self) -> Result<()> {
        let dir = self.dir.canonicalize()?;

        match config(&dir) {
            Ok(cfg) => {
                message!("✅ Configuration is valid");

                // Report what was found
                if let Some(workspace) = &cfg.workspace
                    && let Some(id) = &workspace.id
                {
                    message!("   🔗 Workspace: {id}");
                }

                if let Some(site) = &cfg.site {
                    if let Some(root) = &site.root {
                        message!("   📁 Site root: {}", root.as_str());
                    }
                    if let Some(domain) = &site.domain {
                        message!("   🌐 Site domain: {domain}");
                    }
                    if let Some(routes) = &site.routes {
                        message!("   🔀 Site routes: {} configured", routes.len());
                    }
                    if let Some(exclude) = &site.exclude {
                        message!("   🚫 Site exclusions: {} patterns", exclude.len());
                    }
                    if let Some(layout) = &site.layout {
                        let mut features = Vec::new();
                        if layout.has_left_sidebar() {
                            features.push("left-sidebar");
                        }
                        if layout.has_right_sidebar() {
                            features.push("right-sidebar");
                        }
                        if features.is_empty() {
                            message!("   📐 Site layout: (no features enabled)");
                        } else {
                            message!("   📐 Site layout: {}", features.join(", "));
                        }
                    }
                }

                if let Some(remotes) = &cfg.remotes {
                    message!("   📡 Remotes: {} configured", remotes.len());
                }

                if let Some(outputs) = &cfg.outputs {
                    message!("   📄 Outputs: {} configured", outputs.len());
                }

                Ok(())
            }
            Err(error) => {
                message!("❌ Configuration is invalid");
                Err(error)
            }
        }
    }
}
