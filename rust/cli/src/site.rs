use std::collections::HashMap;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail, eyre};
use stencila_document::{Document, ExecuteOptions};
use url::Url;

use stencila_ask::{Answer, AskLevel, AskOptions, ask_for_password, ask_with};
use stencila_cli_utils::{
    ToStdout,
    color_print::cstr,
    message, parse_domain,
    tabulated::{Cell, CellAlignment, Color, Tabulated},
};
use stencila_cloud::ensure_workspace;
use stencila_cloud::sites::{
    default_site_url, delete_site_branch, delete_site_domain, get_site, get_site_domain_status,
    list_site_branches, set_site_domain, update_site_access,
};
use stencila_config::{
    ConfigTarget, RouteSpread, LayoutConfig, SpreadMode, config, config_add_redirect_route,
    config_add_route, config_remove_route, config_set, config_set_route_spread, config_unset,
    validate_placeholders,
};
use stencila_server::{ServeOptions, SiteMessage, get_server_token};
use tokio::sync::{broadcast, mpsc};

/// Manage the workspace site
#[derive(Debug, Parser)]
#[command(alias = "sites", after_long_help = AFTER_LONG_HELP)]
pub struct Site {
    #[command(subcommand)]
    command: Option<SiteCommand>,
}

pub static AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># View details of the workspace site</dim>
  <b>stencila site</>
  <b>stencila site show</>

  <dim># List configured routes</dim>
  <b>stencila site list</>

  <dim># Add a route</dim>
  <b>stencila site add / index.md</>
  <b>stencila site add /about/ README.md</>
  <b>stencila site add /old/ --redirect /new/ --status 301</>

  <dim># Remove a route</dim>
  <b>stencila site remove /about/</>

  <dim># Push site content to cloud</dim>
  <b>stencila site push</>

  <dim># Show current access restrictions</dim>
  <b>stencila site access</>

  <dim># Make site public (remove all restrictions)</dim>
  <b>stencila site access --public</>

  <dim># Enable team access restriction</dim>
  <b>stencila site access team</>

  <dim># Set a password for the site</dim>
  <b>stencila site access password</>

  <dim># Clear the password</dim>
  <b>stencila site access password --clear</>
"
);

#[derive(Debug, Subcommand)]
enum SiteCommand {
    Show(Show),
    List(List),
    Add(Add),
    Remove(Remove),
    Render(Render),
    Preview(Preview),
    Push(Push),
    Access(Access),
    Domain(Domain),
    Branch(Branch),
}

impl Site {
    pub async fn run(self) -> Result<()> {
        let command = self.command.unwrap_or(SiteCommand::List(List::default()));

        match command {
            SiteCommand::Show(show) => show.run().await,
            SiteCommand::List(list) => list.run().await,
            SiteCommand::Add(add) => add.run(),
            SiteCommand::Remove(remove) => remove.run(),
            SiteCommand::Render(render) => render.run().await,
            SiteCommand::Preview(preview) => preview.run().await,
            SiteCommand::Push(push) => push.run().await,
            SiteCommand::Access(access) => access.run().await,
            SiteCommand::Domain(domain) => domain.run().await,
            SiteCommand::Branch(branch) => branch.run().await,
        }
    }
}

/// Show details of the workspace site
#[derive(Debug, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
pub struct Show {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># View details of the current workspace's site</dim>
  <b>stencila site</>
  <b>stencila site show</>

  <dim># View details of another workspace's site</dim>
  <b>stencila site show --path /path/to/workspace</>
"
);

impl Show {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        // Read workspace config to get workspace ID (used as site ID)
        let cfg = config(&path)?;
        let workspace_id = match cfg.workspace.and_then(|w| w.id) {
            Some(id) => id,
            None => {
                message(cstr!(
                    "💡 No site is enabled for this workspace. Run <b>stencila site push</> to create one."
                ));
                return Ok(());
            }
        };

        // Fetch site details from API
        let details = get_site(&workspace_id).await?;

        // Sync domain to config
        // Re-read config to check current site settings (we consumed cfg.workspace earlier)
        let cfg = config(&path)?;
        if let Some(domain) = &details.domain {
            config_set("site.domain", domain, ConfigTarget::Nearest)?;
        } else if cfg.site.as_ref().and_then(|s| s.domain.as_ref()).is_some() {
            // Domain was removed on cloud, clear it from config
            config_unset("site.domain", ConfigTarget::Nearest)?;
        }

        // Format access based on boolean flags (matching dashboard logic)
        let access = format_access_label(
            details.team_access,
            details.password_set,
            details.access_restrict_main,
        );

        let url = default_site_url(&workspace_id, details.domain.as_deref());

        // Display site information
        let info = format!(
            "{}\n\
             \n\
             ID:            {}\n\
             Custom domain: {}\n\
             Access:        {}",
            url,
            workspace_id,
            details.domain.as_deref().unwrap_or("None"),
            access
        );

        message!("🌐 {info}");

        Ok(())
    }
}

/// Format access label based on team_access and password_set flags
fn format_access_label(
    team_access: bool,
    password_set: bool,
    access_restrict_main: bool,
) -> String {
    let base = if !team_access && !password_set {
        "Public"
    } else if team_access && password_set {
        "Collaborators or password"
    } else if team_access {
        "Collaborators only"
    } else {
        "Password protected"
    };

    // Add main branch exclusion note if applicable
    if (team_access || password_set) && !access_restrict_main {
        format!("{base} (excluding main/master branches)")
    } else {
        base.to_string()
    }
}

/// List all routes (configured and file-implied)
#[derive(Debug, Default, Args)]
#[command(alias = "ls", after_long_help = LIST_AFTER_LONG_HELP)]
pub struct List {
    /// Show expanded spread route variants
    ///
    /// When set, spread routes are expanded into their individual variants
    /// instead of showing the template with a variant count.
    #[arg(long, alias = "expand")]
    expanded: bool,

    /// Show routes for static files (e.g. images, CSS)
    #[arg(long)]
    statics: bool,

    /// Filter by route prefix
    ///
    /// Only show routes that start with this prefix (e.g., "/docs/")
    #[arg(long = "route")]
    route_filter: Option<String>,

    /// Filter by source file path prefix
    ///
    /// Only show routes whose source file starts with this prefix (e.g., "docs/")
    #[arg(long = "path")]
    path_filter: Option<String>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all routes (configured and file-implied)</dim>
  <b>stencila site</>
  <b>stencila site list</>

  <dim># Show expanded spread route variants</dim>
  <b>stencila site list --expanded</>

  <dim># Show routes for static files (e.g. images)</dim>
  <b>stencila site list --statics</>

  <dim># Filter routes by route prefix</dim>
  <b>stencila site list --route /docs</>

  <dim># Filter routes by source file path prefix</dim>
  <b>stencila site list --path docs/</>
"
);

impl List {
    pub async fn run(self) -> Result<()> {
        use stencila_site::{RouteType, list};

        let routes = list(
            &current_dir()?,
            self.expanded,
            self.statics,
            self.route_filter.as_deref(),
            self.path_filter.as_deref(),
        )
        .await?;

        if routes.is_empty() {
            message(cstr!(
                "💡 No routes found. To add a route, run <b>stencila site add ROUTE FILE</>"
            ));
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Route", "Type", "Target"]);

        for entry in routes {
            let type_str = match entry.route_type {
                RouteType::File => "file".to_string(),
                RouteType::Redirect => "redirect".to_string(),
                RouteType::Spread => {
                    if let Some(count) = entry.spread_count {
                        format!("spread x{count}")
                    } else {
                        "spread".to_string()
                    }
                }
                RouteType::Implied => "implied".to_string(),
                RouteType::Static => "static".to_string(),
            };

            let type_cell = match entry.route_type {
                RouteType::File => Cell::new(&type_str).fg(Color::Green),
                RouteType::Redirect => Cell::new(&type_str).fg(Color::Yellow),
                RouteType::Spread => Cell::new(&type_str).fg(Color::Magenta),
                RouteType::Implied => Cell::new(&type_str).fg(Color::Grey),
                RouteType::Static => Cell::new(&type_str).fg(Color::Blue),
            };

            // Format target with spread arguments if present
            let target_str = if let Some(args) = &entry.spread_arguments {
                let args_str: Vec<String> = args.iter().map(|(k, v)| format!("{k}={v}")).collect();
                format!("{} ({})", entry.target, args_str.join(", "))
            } else {
                entry.target.clone()
            };

            table.add_row([
                Cell::new(&entry.route).fg(Color::Cyan),
                type_cell,
                Cell::new(&target_str),
            ]);
        }

        table.to_stdout();
        Ok(())
    }
}

/// Add a route
#[derive(Debug, Args)]
#[command(after_long_help = ADD_AFTER_LONG_HELP)]
pub struct Add {
    /// Route path (e.g., "/", "/about/", "/{region}/report/")
    route: String,

    /// File to serve at this route
    file: Option<String>,

    /// Redirect URL (instead of a file)
    #[arg(long, short)]
    redirect: Option<String>,

    /// HTTP status code for redirect (301, 302, 303, 307, 308)
    #[arg(long, short)]
    status: Option<u16>,

    /// Spread mode for multi-variant routes (grid or zip)
    ///
    /// Use with routes containing placeholders like "/{region}/report/".
    /// - grid: Cartesian product of all argument values (default)
    /// - zip: Positional pairing (all arguments must have same length)
    #[arg(long, value_enum)]
    spread: Option<SpreadMode>,

    /// Arguments for spread routes (comma-delimited key=val1,val2 pairs)
    ///
    /// Example: stencila site add "/{region}/" report.smd -- region=north,south
    #[arg(last = true, allow_hyphen_values = true)]
    arguments: Vec<String>,
}

pub static ADD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Add a file route</dim>
  <b>stencila site add / index.md</>
  <b>stencila site add /about/ README.md</>

  <dim># Add a redirect</dim>
  <b>stencila site add /old/ --redirect /new/</>
  <b>stencila site add /old/ --redirect /new/ --status 301</>

  <dim># Add external redirect</dim>
  <b>stencila site add /github/ --redirect https://github.com/stencila/stencila</>

  <dim># Add a spread route (generates multiple variants)</dim>
  <b>stencila site add \"/{region}/\" report.smd -- region=north,south</>
  <b>stencila site add \"/{region}/{year}/\" report.smd -- region=north,south year=2024,2025</>
  <b>stencila site add \"/{q}-report/\" quarterly.smd --spread zip -- q=q1,q2,q3,q4</>
"
);

impl Add {
    pub fn run(self) -> Result<()> {
        // Auto-add leading / if missing
        let route = if self.route.starts_with('/') {
            self.route
        } else {
            format!("/{}", self.route)
        };

        // Must have either file or redirect
        if self.file.is_none() && self.redirect.is_none() {
            bail!("Must specify either a file or use --redirect");
        }

        if self.file.is_some() && self.redirect.is_some() {
            bail!("Cannot specify both a file and use --redirect");
        }

        if self.status.is_some() && self.redirect.is_none() {
            bail!("--status can only be used with --redirect");
        }

        // Check for spread-related options with redirect
        if self.redirect.is_some() && (self.spread.is_some() || !self.arguments.is_empty()) {
            bail!("--spread and arguments cannot be used with --redirect");
        }

        // Check if this is a spread route (has placeholders like {region})
        let has_placeholders = route.contains('{') && route.contains('}');

        if let Some(file) = &self.file {
            let file_path = std::path::Path::new(file);
            if !file_path.exists() {
                message!("⚠️  Warning: File '{}' does not exist", file);
            }

            // Check if we have spread arguments
            if !self.arguments.is_empty() {
                // Parse arguments into HashMap
                let arguments = Self::parse_arguments(&self.arguments)?;

                if arguments.is_empty() {
                    bail!("Arguments provided but no valid key=value pairs found");
                }

                if !has_placeholders {
                    bail!(
                        "Route '{}' has no placeholders but arguments were provided. \
                         Use placeholders like /{{region}}/ for spread routes.",
                        route
                    );
                }

                // Validate that each placeholder has a corresponding argument
                validate_placeholders(&route, Some(&arguments), "Route")?;

                // Create spread config (config_set_route_spread handles path resolution)
                let spread = RouteSpread {
                    file: file.clone(),
                    spread: self.spread,
                    arguments,
                };

                config_set_route_spread(&route, &spread)?;

                let mode = self.spread.unwrap_or_default();
                message!(
                    "✅ Added spread route {} → {} (mode: {:?})",
                    route,
                    file,
                    mode
                );
            } else {
                // Simple file route
                if has_placeholders {
                    bail!(
                        "Route '{}' contains placeholders but no arguments provided. \
                         Either remove placeholders or add arguments after --.",
                        route
                    );
                }

                // config_add_route handles path resolution (relative to site.root if configured)
                let file_path = file_path
                    .canonicalize()
                    .unwrap_or_else(|_| file_path.to_path_buf());
                config_add_route(&file_path, &route)?;
                message!("✅ Added route {} → {}", route, file);
            }
        } else if let Some(redirect) = &self.redirect {
            // Add redirect route
            config_add_redirect_route(&route, redirect, self.status)?;
            let status_str = self.status.map(|s| format!(" ({})", s)).unwrap_or_default();
            message!("✅ Added redirect {} → {}{}", route, redirect, status_str);
        }

        Ok(())
    }

    /// Parse arguments from CLI format "key=val1,val2" into HashMap
    fn parse_arguments(args: &[String]) -> Result<std::collections::HashMap<String, Vec<String>>> {
        let mut result = std::collections::HashMap::new();

        for arg in args {
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                bail!(
                    "Invalid argument format '{}'. Expected 'key=val1,val2'",
                    arg
                );
            }

            let key = parts[0].trim().to_string();
            let values: Vec<String> = parts[1].split(',').map(|s| s.trim().to_string()).collect();

            if key.is_empty() {
                bail!("Argument key cannot be empty in '{}'", arg);
            }
            if values.is_empty() || values.iter().all(|v| v.is_empty()) {
                bail!("Argument '{}' must have at least one value", key);
            }

            result.insert(key, values);
        }

        Ok(result)
    }
}

/// Remove a route
#[derive(Debug, Args)]
#[command(alias = "rm", after_long_help = REMOVE_AFTER_LONG_HELP)]
pub struct Remove {
    /// Route path to remove (e.g., "/about/")
    route: String,
}

pub static REMOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove a route</dim>
  <b>stencila site remove /about/</>
  <b>stencila site remove /old/</>
"
);

impl Remove {
    pub fn run(self) -> Result<()> {
        config_remove_route(&self.route)?;
        message!("✅ Removed route {}", self.route);
        Ok(())
    }
}

/// Render site content to a directory
#[derive(Debug, Args)]
#[command(after_long_help = RENDER_AFTER_LONG_HELP)]
pub struct Render {
    /// Output directory for rendered files
    #[arg()]
    pub output: PathBuf,

    /// Source directory (uses site.root if configured, otherwise current directory)
    #[arg(long, short)]
    pub source: Option<PathBuf>,

    /// Filter by route prefix (only render matching routes)
    #[arg(long = "route")]
    pub route_filter: Option<String>,

    /// Filter by source file path prefix
    #[arg(long = "path")]
    pub path_filter: Option<String>,
}

pub static RENDER_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Render site to a directory</dim>
  <b>stencila site render ./dist</>

  <dim># Render specific routes</dim>
  <b>stencila site render ./dist --route /docs/</>

  <dim># Render from a specific source</dim>
  <b>stencila site render ./dist --source ./content</>
"
);

impl Render {
    pub async fn run(self) -> Result<()> {
        use stencila_dirs::{closest_stencila_dir, workspace_dir};
        use stencila_site::RenderProgress;

        // Resolve source path
        let source = self.source.map_or_else(current_dir, Ok)?;

        // If using default path, check if site.root is configured
        let source = {
            let cfg = stencila_config::config(&source)?;
            if let Some(site) = &cfg.site
                && let Some(root) = &site.root
            {
                let stencila_dir = closest_stencila_dir(&source, true).await?;
                let ws_dir = workspace_dir(&stencila_dir)?;
                root.resolve(&ws_dir)
            } else {
                source
            }
        };

        // Get base_url from config or use localhost
        let cfg = stencila_config::config(&source)?;
        let base_url = cfg
            .site
            .as_ref()
            .and_then(|s| s.domain.as_ref())
            .map(|domain| format!("https://{domain}"))
            .unwrap_or_else(|| "https://localhost".to_string());

        // Set up progress channel
        let (tx, mut rx) = mpsc::channel::<RenderProgress>(100);

        // Spawn progress handler
        let progress_handle = tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                match progress {
                    RenderProgress::WalkingDirectory => {
                        message("📁 Walking directory");
                    }
                    RenderProgress::FilesFound {
                        documents,
                        static_files,
                    } => {
                        message!("📊 Found {documents} documents, {static_files} static files");
                    }
                    RenderProgress::EncodingDocument {
                        relative_path,
                        index,
                        total,
                        ..
                    } => {
                        message!("📃 Rendering {}/{}: {}", index + 1, total, relative_path);
                    }
                    RenderProgress::DocumentFailed { path, error } => {
                        message!("❌ Failed: {}: {}", path.display(), error);
                    }
                    RenderProgress::CopyingStatic { copied, total } => {
                        if copied == total {
                            message!("📦 Copied {total} static files");
                        }
                    }
                    _ => {}
                }
            }
        });

        message!("🔨 Rendering site to {}", self.output.display());

        // Call render
        let result = stencila_site::render(
            &source,
            &self.output,
            &base_url,
            self.route_filter.as_deref(),
            self.path_filter.as_deref(),
            Some(tx),
            |doc_path, arguments: HashMap<String, String>| async move {
                let doc = Document::open(&doc_path, None).await?;
                let arguments: Vec<(&str, &str)> = arguments
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_str()))
                    .collect();
                doc.call(&arguments, ExecuteOptions::default()).await?;
                Ok(doc.root().await)
            },
        )
        .await;

        // Wait for progress handler
        let _ = progress_handle.await;

        let result = result?;

        message!(
            "✅ Rendered {} documents, {} static files, {} media files to {}",
            result.documents_ok.len(),
            result.static_files.len(),
            result.media_files_count,
            self.output.display()
        );

        if result.media_duplicates_eliminated > 0 {
            message!(
                "♻️ {} media duplicates eliminated",
                result.media_duplicates_eliminated
            );
        }

        if !result.documents_failed.is_empty() {
            message!("⚠️ {} documents failed:", result.documents_failed.len());
            for (doc_path, error) in &result.documents_failed {
                message!("     - {}: {}", doc_path.display(), error);
            }
        }

        Ok(())
    }
}

/// Preview the workspace site locally with live reload
#[derive(Debug, Args)]
#[command(after_long_help = PREVIEW_AFTER_LONG_HELP)]
pub struct Preview {
    /// Route to open in browser (default: /)
    #[arg(default_value = "/")]
    route: String,

    /// Port to serve on
    #[arg(long, short, default_value_t = 9000)]
    port: u16,

    /// Do not open browser automatically
    #[arg(long)]
    no_open: bool,

    /// Do not watch for file changes
    #[arg(long)]
    no_watch: bool,
}

pub static PREVIEW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Preview site at root</dim>
  <b>stencila site preview</>

  <dim># Preview a specific route</dim>
  <b>stencila site preview /docs/guide/</>

  <dim># Preview without opening browser</dim>
  <b>stencila site preview --no-open</>

  <dim># Preview on different port</dim>
  <b>stencila site preview --port 8080</>

  <dim># Preview without file watching</dim>
  <b>stencila site preview --no-watch</>
"
);

impl Preview {
    pub async fn run(self) -> Result<()> {
        use stencila_dirs::{closest_stencila_dir, workspace_dir};

        let cwd = current_dir()?;
        let cfg = config(&cwd)?;

        // Get layout from config
        let layout = cfg.site.as_ref().and_then(|s| s.layout.clone());

        // Resolve site root
        let stencila_dir = closest_stencila_dir(&cwd, true).await?;
        let ws_dir = workspace_dir(&stencila_dir)?;
        let site_root = cfg
            .site
            .as_ref()
            .and_then(|s| s.root.as_ref())
            .map(|r| r.resolve(&ws_dir))
            .unwrap_or_else(|| cwd.clone());

        // Create temp directory (auto-cleans on drop)
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path().to_path_buf();

        // Initial render (render() outputs uncompressed HTML with flat structure)
        message!("📁 Rendering site to temporary directory...");
        Self::render_site(&site_root, &temp_path, layout.as_ref()).await?;

        // Serve directly from temp_path (render uses flat structure, no decompression needed)
        let serve_dir = temp_path.clone();

        // Generate server token
        let server_token = get_server_token();

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        // Start server
        let server_port = self.port;
        let server_token_clone = server_token.clone();

        // Create broadcast channel for site notifications
        // The CLI sends messages after re-rendering, server broadcasts to WebSocket clients
        let (site_notify_tx, _) = broadcast::channel::<SiteMessage>(16);
        let site_notify_tx_clone = site_notify_tx.clone();

        let server_handle = tokio::spawn(async move {
            let options = ServeOptions {
                dir: serve_dir.clone(),
                port: server_port,
                server_token: Some(server_token_clone),
                no_startup_message: true,
                shutdown_receiver: Some(shutdown_rx),
                // Serve pre-rendered HTML files directly without document processing
                static_dir: Some(serve_dir),
                // Use broadcast channel for notifications (not file watching)
                site_notify: Some(site_notify_tx_clone),
                ..Default::default()
            };
            stencila_server::serve(options).await
        });

        message!("🌐 Preview at http://localhost:{}", self.port);

        // Open browser
        if !self.no_open {
            let url = format!(
                "http://localhost:{}/~login?sst={}&next={}",
                self.port, server_token, self.route
            );
            if let Err(error) = webbrowser::open(&url) {
                tracing::warn!("Failed to open browser: {error}");
            }
        }

        // Watch loop or wait for Ctrl+C
        if self.no_watch {
            message!("Press Ctrl+C to stop");
            tokio::signal::ctrl_c().await?;
        } else {
            message!("👁️ Watching for changes (Ctrl+C to stop)");
            Self::watch_and_rerender(&cwd, &site_root, &temp_path, layout, site_notify_tx).await?;
        }

        // Graceful shutdown
        message!("Shutting down...");
        let _ = shutdown_tx.send(());
        let _ = server_handle.await;

        // temp_dir drops here, cleaning up rendered files
        Ok(())
    }

    /// Render the site to the output directory
    async fn render_site(source: &Path, output: &Path, _layout: Option<&LayoutConfig>) -> Result<()> {
        use stencila_site::RenderProgress;

        // Use render() to render to the output directory
        let (tx, mut rx) = tokio::sync::mpsc::channel::<RenderProgress>(100);

        // Base URL for local preview
        let base_url = "http://localhost:9000".to_string();

        // Spawn progress handler
        let progress_handle = tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                match progress {
                    RenderProgress::EncodingDocument {
                        relative_path,
                        index,
                        total,
                        ..
                    } => {
                        message!("📃 Rendering {}/{}: {}", index + 1, total, relative_path);
                    }
                    RenderProgress::DocumentFailed { path, error } => {
                        message!("❌ Failed: {}: {}", path.display(), error);
                    }
                    _ => {}
                }
            }
        });

        // Call render directly
        let result = stencila_site::render(
            source,
            output,
            &base_url,
            None,
            None,
            Some(tx),
            |doc_path, arguments: HashMap<String, String>| async move {
                let doc = Document::open(&doc_path, None).await?;
                let arguments: Vec<(&str, &str)> = arguments
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_str()))
                    .collect();
                doc.call(&arguments, ExecuteOptions::default()).await?;
                Ok(doc.root().await)
            },
        )
        .await;

        // Wait for progress handler
        let _ = progress_handle.await;

        result?;
        message!("✅ Site rendered");
        Ok(())
    }

    /// Watch for changes and re-render
    async fn watch_and_rerender(
        workspace_root: &Path,
        site_root: &Path,
        output: &Path,
        mut layout: Option<LayoutConfig>,
        site_notify: broadcast::Sender<SiteMessage>,
    ) -> Result<()> {
        // Watch config file for layout changes
        let mut config_receiver = stencila_config::watch(workspace_root).await?;

        // Watch site root for file changes
        let mut site_receiver = stencila_site::watch(site_root, Some(output)).await?;

        loop {
            tokio::select! {
                // Ctrl+C to exit
                _ = tokio::signal::ctrl_c() => {
                    break;
                }

                // Config changed - update layout and re-render
                Some(result) = async {
                    match config_receiver.as_mut() {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    match result {
                        Ok(new_config) => {
                            layout = new_config.site.and_then(|s| s.layout);
                            message!("🔄 Config changed, re-rendering...");
                            if let Err(e) = Self::render_site(site_root, output, layout.as_ref()).await {
                                message!("❌ Render error: {e}");
                                // Continue watching - don't exit on render errors
                            } else {
                                // Notify browser to reload after successful re-render
                                let _ = site_notify.send(SiteMessage::ConfigChange);
                            }
                        }
                        Err(e) => {
                            message!("⚠️ Config error: {e}");
                        }
                    }
                }

                // Site files changed - re-render
                Some(event) = site_receiver.recv() => {
                    let changed: Vec<_> = event.paths.iter()
                        .filter_map(|p| p.file_name())
                        .filter_map(|n| n.to_str())
                        .take(3) // Limit display to 3 files
                        .collect();
                    let suffix = if event.paths.len() > 3 {
                        format!(" (+{} more)", event.paths.len() - 3)
                    } else {
                        String::new()
                    };
                    message!("🔄 Files changed: {}{}, re-rendering...", changed.join(", "), suffix);

                    if let Err(e) = Self::render_site(site_root, output, layout.as_ref()).await {
                        message!("❌ Render error: {e}");
                        // Continue watching - don't exit on render errors
                    } else {
                        // Notify browser to reload after successful re-render
                        let paths: Vec<String> = event.paths.iter()
                            .filter_map(|p| p.to_str())
                            .map(String::from)
                            .collect();
                        let _ = site_notify.send(SiteMessage::SiteChange { paths });
                    }
                }

                else => break,
            }
        }

        Ok(())
    }
}

/// Push site content to Stencila Cloud
#[derive(Debug, Args)]
#[command(after_long_help = PUSH_AFTER_LONG_HELP)]
pub struct Push {
    /// Path to push (file or directory)
    ///
    /// If not specified, uses site.root if configured, otherwise current directory
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Force push without checking etags
    #[arg(long, short)]
    pub force: bool,
}

pub static PUSH_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Push site content to cloud (uses site.root if configured)</dim>
  <b>stencila site push</>

  <dim># Push a specific directory</dim>
  <b>stencila site push ./site/docs</>

  <dim># Push a specific file</dim>
  <b>stencila site push ./site/report.md</>

  <dim># Force push (ignore unchanged files)</dim>
  <b>stencila site push --force</>
"
);

impl Push {
    pub async fn run(self) -> Result<()> {
        use stencila_dirs::{closest_stencila_dir, workspace_dir};
        use stencila_site::PushProgress;

        // Resolve the provided path
        let is_default_path = self.path == PathBuf::from(".");
        let mut path = if is_default_path {
            current_dir()?
        } else {
            self.path.clone()
        };

        // If using default path ("."), check if site.root is configured
        if is_default_path {
            let cfg = stencila_config::config(&path)?;
            if let Some(site) = &cfg.site
                && let Some(root) = &site.root
            {
                let stencila_dir = closest_stencila_dir(&path, true).await?;
                let ws_dir = workspace_dir(&stencila_dir)?;
                path = root.resolve(&ws_dir);
            }
        }

        let path_display = path.display();

        // Ensure workspace exists, creating it if needed
        let (workspace_id, already_existed) = ensure_workspace(&path).await?;
        if !already_existed {
            message!("✨ Workspace registered: https://{workspace_id}.stencila.site");
        }

        // Set up progress channel
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PushProgress>(100);

        // Spawn a task to handle progress updates
        let progress_handle = tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                match progress {
                    PushProgress::WalkingDirectory => {
                        message("📁 Walking directory");
                    }
                    PushProgress::FilesFound {
                        documents,
                        static_files,
                    } => {
                        message!("📊 Found {documents} documents, {static_files} static files");
                    }
                    PushProgress::EncodingDocument {
                        relative_path,
                        index,
                        total,
                        ..
                    } => {
                        message!("📃 Rendering {}/{}: {}", index + 1, total, relative_path);
                    }
                    PushProgress::DocumentEncoded { .. } => {}
                    PushProgress::DocumentFailed { path, error } => {
                        message!("❌ Failed: {}: {}", path.display(), error);
                    }
                    PushProgress::UploadStarting { total } => {
                        message!("☁️ Uploading {total} files");
                    }
                    PushProgress::Processing {
                        processed,
                        uploaded,
                        total,
                    } => {
                        if processed == total {
                            let unchanged = total - uploaded;
                            message!(
                                "⚙️ Uploaded {uploaded}/{total} files ({unchanged} unchanged)"
                            );
                        }
                    }
                    PushProgress::Reconciling => {
                        message("🔄 Reconciling files");
                    }
                    PushProgress::Complete(_) => {}
                }
            }
        });

        message!("☁️ Pushing directory `{path_display}` to workspace site");

        // Call push with a decoder function
        let result = stencila_site::push(
            &path,
            &workspace_id,
            None, // Use current branch
            None,
            None,
            self.force,
            Some(tx),
            |doc_path, arguments: HashMap<String, String>| async move {
                let doc = Document::open(&doc_path, None).await?;
                let arguments: Vec<(&str, &str)> = arguments
                    .iter()
                    .map(|(name, value)| (name.as_str(), value.as_str()))
                    .collect();
                doc.call(&arguments, ExecuteOptions::default()).await?;
                Ok(doc.root().await)
            },
        )
        .await;

        // Wait for progress handler to finish
        let _ = progress_handle.await;

        // Handle result
        let result = result?;

        message!(
            "✅ Push complete: {} documents, {} redirects, {} static files, {} media files",
            result.render.documents_ok.len(),
            result.render.redirects.len(),
            result.render.static_files.len(),
            result.render.media_files_count
        );

        if result.render.media_duplicates_eliminated > 0 {
            message!(
                "♻️ {} media duplicates eliminated",
                result.render.media_duplicates_eliminated
            );
        }

        if result.upload.files_skipped > 0 {
            message!(
                "⏭️ {} unchanged files skipped (use --force to upload all)",
                result.upload.files_skipped
            );
        }

        if !result.render.documents_failed.is_empty() {
            message!(
                "⚠️ {} documents failed:",
                result.render.documents_failed.len()
            );
            for (doc_path, error) in &result.render.documents_failed {
                message!("     - {}: {}", doc_path.display(), error);
            }
        }

        let url = format!("https://{workspace_id}.stencila.site");
        let url = Url::parse(&url)?;
        let url = stencila_site::browseable_url(&url, Some(&path))?;
        message!("🔗 Site available at: {url}");

        Ok(())
    }
}

/// Manage access restrictions for the workspace site
#[derive(Debug, Parser)]
#[command(after_long_help = ACCESS_AFTER_LONG_HELP)]
pub struct Access {
    /// Make the site public (remove all access restrictions)
    #[arg(long)]
    public: bool,

    /// Path to the workspace directory
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Option<AccessCommand>,
}

pub static ACCESS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show current access restrictions</dim>
  <b>stencila site access</>

  <dim># Make site public (remove all restrictions)</dim>
  <b>stencila site access --public</>

  <dim># Enable team access restriction</dim>
  <b>stencila site access team</>

  <dim># Disable team access restriction</dim>
  <b>stencila site access team --off</>

  <dim># Set a password for the site</dim>
  <b>stencila site access password</>

  <dim># Clear the password</dim>
  <b>stencila site access password --clear</>
"
);

#[derive(Debug, Subcommand)]
enum AccessCommand {
    /// Manage team access restriction
    Team(AccessTeam),
    /// Manage password protection
    Password(AccessPassword),
}

impl Access {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let workspace_id = match cfg.workspace.and_then(|w| w.id) {
            Some(id) => id,
            None => {
                message(cstr!(
                    "💡 No site is enabled for this workspace. Run <b>stencila site push</> to create one."
                ));
                return Ok(());
            }
        };
        let domain = cfg.site.and_then(|s| s.domain);

        // Handle --public flag
        if self.public {
            // Clear both teamAccess and password
            update_site_access(&workspace_id, Some(false), Some(None), None).await?;
            message!(
                "✅ Site {} is now public",
                default_site_url(&workspace_id, domain.as_deref())
            );
            return Ok(());
        }

        // If no subcommand, show current access state
        let Some(command) = self.command else {
            let details = get_site(&workspace_id).await?;

            let access = format_access_label(
                details.team_access,
                details.password_set,
                details.access_restrict_main,
            );

            message!(
                "Access: {}\n  Team access:   {}\n  Password:      {}\n  Restrict main: {}",
                access,
                if details.team_access {
                    "enabled"
                } else {
                    "disabled"
                },
                if details.password_set {
                    "set"
                } else {
                    "not set"
                },
                if details.access_restrict_main {
                    "yes"
                } else {
                    "no"
                }
            );
            return Ok(());
        };

        match command {
            AccessCommand::Team(team) => {
                team.run_with_context(&workspace_id, domain.as_deref())
                    .await
            }
            AccessCommand::Password(password) => {
                password
                    .run_with_context(&workspace_id, domain.as_deref())
                    .await
            }
        }
    }
}

/// Manage team access restriction
#[derive(Debug, Args)]
#[command(after_long_help = ACCESS_TEAM_AFTER_LONG_HELP)]
pub struct AccessTeam {
    /// Disable team access restriction
    #[arg(long)]
    off: bool,

    /// Do not apply restriction to main or master branches
    #[arg(long)]
    not_main: bool,

    /// Apply restriction to main or master branches
    #[arg(long, conflicts_with = "not_main")]
    main: bool,
}

pub static ACCESS_TEAM_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Enable team access restriction</dim>
  <b>stencila site access team</>

  <dim># Disable team access restriction</dim>
  <b>stencila site access team --off</>

  <dim># Enable but exclude main/master branches</dim>
  <b>stencila site access team --not-main</>
"
);

impl AccessTeam {
    pub async fn run_with_context(self, workspace_id: &str, domain: Option<&str>) -> Result<()> {
        let team_access = !self.off;

        // Determine accessRestrictMain value if flags are provided
        let access_restrict_main = if self.main {
            Some(true)
        } else if self.not_main {
            Some(false)
        } else {
            None
        };

        update_site_access(workspace_id, Some(team_access), None, access_restrict_main).await?;

        let status = if self.off { "disabled" } else { "enabled" };
        let main_note = if self.not_main {
            " (excluding main/master branches)"
        } else if self.main {
            " (including main/master branches)"
        } else {
            ""
        };

        message!(
            "✅ Team access {} for {}{}",
            status,
            default_site_url(workspace_id, domain),
            main_note
        );

        Ok(())
    }
}

/// Manage password protection
#[derive(Debug, Args)]
#[command(after_long_help = ACCESS_PASSWORD_AFTER_LONG_HELP)]
pub struct AccessPassword {
    /// Clear the password
    #[arg(long)]
    clear: bool,

    /// Do not apply password protection to main or master branches
    #[arg(long)]
    not_main: bool,

    /// Apply password protection to main or master branches
    #[arg(long, conflicts_with = "not_main")]
    main: bool,
}

pub static ACCESS_PASSWORD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Set a password for the site</dim>
  <b>stencila site access password</>

  <dim># Clear the password</dim>
  <b>stencila site access password --clear</>

  <dim># Set password but exclude main/master branches</dim>
  <b>stencila site access password --not-main</>
"
);

impl AccessPassword {
    pub async fn run_with_context(self, workspace_id: &str, domain: Option<&str>) -> Result<()> {
        // Determine accessRestrictMain value if flags are provided
        let access_restrict_main = if self.main {
            Some(true)
        } else if self.not_main {
            Some(false)
        } else {
            None
        };

        if self.clear {
            // Ask for confirmation
            let answer = ask_with(
                "This will clear the password from your site.",
                AskOptions {
                    level: AskLevel::Warning,
                    default: Some(Answer::No),
                    title: Some("Clear Password".into()),
                    yes_text: Some("Yes, clear password".into()),
                    no_text: Some("Cancel".into()),
                    ..Default::default()
                },
            )
            .await?;

            if !answer.is_yes() {
                message("ℹ️ Password clear cancelled");
                return Ok(());
            }

            update_site_access(workspace_id, None, Some(None), access_restrict_main).await?;
            message!(
                "✅ Password cleared from {}",
                default_site_url(workspace_id, domain)
            );
        } else {
            // Prompt for password
            let password = ask_for_password(cstr!(
                "Enter password for your site (will not be displayed)"
            ))
            .await?;

            update_site_access(
                workspace_id,
                None,
                Some(Some(&password)),
                access_restrict_main,
            )
            .await?;

            let main_note = if self.not_main {
                " (excluding main/master branches)"
            } else if self.main {
                " (including main/master branches)"
            } else {
                ""
            };

            message!(
                "✅ Password set for {}{}",
                default_site_url(workspace_id, domain),
                main_note
            );
        }

        Ok(())
    }
}

/// Manage custom domain for the workspace site
#[derive(Debug, Parser)]
#[command(after_long_help = DOMAIN_AFTER_LONG_HELP)]
pub struct Domain {
    #[command(subcommand)]
    command: DomainCommand,
}

pub static DOMAIN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Set a custom domain for the site</dim>
  <b>stencila site domain set example.com</>

  <dim># Check domain status</dim>
  <b>stencila site domain status</>

  <dim># Remove the custom domain</dim>
  <b>stencila site domain clear</>
"
);

#[derive(Debug, Subcommand)]
enum DomainCommand {
    Set(DomainSet),
    Status(DomainStatus),
    Clear(DomainClear),
}

impl Domain {
    pub async fn run(self) -> Result<()> {
        match self.command {
            DomainCommand::Set(set) => set.run().await,
            DomainCommand::Status(status) => status.run().await,
            DomainCommand::Clear(clear) => clear.run().await,
        }
    }
}

/// Format CNAME record setup instructions
fn format_cname_instructions(cname_record: &str, cname_target: &str) -> String {
    format!(
        "Add this CNAME record to your DNS:\n   \
        Name:   {} (or @ if configuring apex domain)\n   \
        Target: {}\n\n\
        ⚠️ If using Cloudflare DNS, set the CNAME to \"DNS only\" (gray cloud icon).\n   \
        Do not use \"Proxied\" mode (orange cloud) as this will prevent verification.",
        cname_record, cname_target
    )
}

/// Set a custom domain for the site
#[derive(Debug, Args)]
#[command(after_long_help = DOMAIN_SET_AFTER_LONG_HELP)]
pub struct DomainSet {
    /// The custom domain to use for the site
    ///
    /// Must be a valid domain name (IP addresses and ports are not allowed)
    #[arg(value_parser = parse_domain)]
    domain: String,

    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static DOMAIN_SET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Set custom domain for the current workspace's site</dim>
  <b>stencila site domain set example.com</>

  <dim># Set custom domain for another workspace's site</dim>
  <b>stencila site domain set example.com --path /path/to/workspace</>

<bold><b>Setup Process</b></bold>

  After running this command, you'll need to complete the following steps:

  1. <bold>Add the CNAME record to your DNS</bold>
     The command will provide the exact record details (name and target)

  2. <bold>Wait for DNS propagation</bold> (usually 5-30 minutes)
     DNS changes can take time to propagate globally

  3. <bold>Check status:</bold> <dim>stencila site domain status</dim>
     Monitor the verification and SSL provisioning progress

  Once the CNAME is detected, SSL will be provisioned automatically and
  your site will go live.

<bold><b>Troubleshooting</b></bold>

  <bold>Domain status stuck on \"Waiting for CNAME record to be configured\":</bold>

  1. <bold>Verify CNAME is configured correctly:</bold>
     <dim>dig example.com CNAME</dim>
     <dim>nslookup -type=CNAME example.com</dim>
     Should show your domain pointing to the CNAME target provided

  2. <bold>Cloudflare DNS users:</bold>
     - Ensure CNAME is set to \"DNS only\" (gray cloud), NOT \"Proxied\" (orange cloud)
     - Proxied mode prevents domain verification and SSL provisioning
     - This setting must remain \"DNS only\" permanently, not just during setup

  3. <bold>Check for conflicting DNS records:</bold>
     - Remove any A or AAAA records for the same hostname
     - Ensure no NS records delegating to a different DNS provider

  4. <bold>Wait for DNS propagation:</bold>
     - DNS changes typically take 5-30 minutes (sometimes up to 48 hours)
     - Check propagation: <dim>https://dnschecker.org</dim>

  5. <bold>Apex domain issues:</bold>
     - Some DNS providers don't support CNAME on apex/root domains
     - Consider using a subdomain (e.g., www.example.com) instead
"
);

impl DomainSet {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let workspace_id = cfg
            .workspace
            .and_then(|w| w.id)
            .ok_or_else(|| eyre!("No workspace configured for this directory"))?;

        // Set the domain via API
        let response = set_site_domain(&workspace_id, &self.domain).await?;

        // Sync domain to config
        config_set("site.domain", &response.domain, ConfigTarget::Nearest)?;

        // Display appropriate message and instructions based on status
        match response.status.as_str() {
            "pending_dns" => {
                let cname_instructions =
                    format_cname_instructions(&response.cname_record, &response.cname_target);

                message!(
                    "⏳ Custom domain `{}` set for site `{}`\n\n\
                    To complete setup:\n\n\
                    1. {}\n\n\
                    2. Wait for DNS propagation (usually 5-30 minutes)\n\n\
                    3. Check status with: *stencila site domain status*\n\n\
                    Once the CNAME is detected, SSL will be provisioned automatically and your site will go live.",
                    response.domain,
                    workspace_id,
                    cname_instructions
                );
            }
            "ssl_initializing"
            | "ssl_pending_validation"
            | "ssl_pending_issuance"
            | "ssl_pending_deployment" => {
                message!("🔄 SSL provisioning started for `{}`", response.domain);
                if let Some(true) = response.cname_configured {
                    message(
                        "\nCNAME record detected! SSL certificate is being provisioned...\n\n\
                        Check status with: *stencila site domain status*",
                    );
                } else {
                    let cname_instructions =
                        format_cname_instructions(&response.cname_record, &response.cname_target);

                    message!(
                        "\nTo complete setup:\n\n\
                        1. {}\n\n\
                        2. Monitor progress with: *stencila site domain status*",
                        cname_instructions
                    );
                }
            }
            "active" => {
                message!("🎉 Your site is now live at https://{}", response.domain);
            }
            "failed" => {
                bail!(
                    "Domain setup failed for `{}`. Run *stencila site domain status* for details.",
                    response.domain
                );
            }
            _ => {
                message!("🔄 Status: {}", response.status);
            }
        }

        Ok(())
    }
}

/// Check the status of the custom domain
#[derive(Debug, Args)]
#[command(after_long_help = DOMAIN_STATUS_AFTER_LONG_HELP)]
pub struct DomainStatus {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static DOMAIN_STATUS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Check domain status</dim>
  <b>stencila site domain status</>

  <dim># Check status for another workspace</dim>
  <b>stencila site domain status --path /path/to/workspace</>
"
);

impl DomainStatus {
    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let workspace_id = cfg
            .workspace
            .and_then(|w| w.id)
            .ok_or_else(|| eyre!("No workspace configured for this directory"))?;

        // Get domain status
        let status = get_site_domain_status(&workspace_id).await?;

        if !status.configured {
            message("ℹ️ No custom domain is configured for this site");
        } else if let Some("active") = status.status.as_deref()
            && let Some(domain) = &status.domain
        {
            message!("🎉 Your site is live at https://{domain}");
        } else {
            let emoji = match status.status.as_deref() {
                Some("active") => "✅",
                Some("failed") => "❌",
                _ => "⏳",
            };

            let mut parts = Vec::new();

            if let Some(domain) = &status.domain {
                parts.push(format!("Status of custom domain setup for `{domain}`:"));
            }

            parts.push(status.message.clone());

            message!("{emoji} {}", parts.join("\n "));
        }

        Ok(())
    }
}

/// Remove the custom domain from the site
#[derive(Debug, Args)]
#[command(after_long_help = DOMAIN_CLEAR_AFTER_LONG_HELP)]
pub struct DomainClear {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static DOMAIN_CLEAR_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove custom domain from the current workspace's site</dim>
  <b>stencila site domain clear</>

  <dim># Remove custom domain from another workspace's site</dim>
  <b>stencila site domain clear --path /path/to/workspace</>
"
);

impl DomainClear {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let workspace_id = cfg
            .workspace
            .and_then(|w| w.id)
            .ok_or_else(|| eyre!("No workspace configured for this directory"))?;

        // Check if a domain is configured before prompting
        let status = get_site_domain_status(&workspace_id).await?;
        if !status.configured {
            message("ℹ️ No custom domain is configured for this site");
            return Ok(());
        }

        // Ask for confirmation
        let answer = ask_with(
            "This will remove the custom domain from your site. The site will continue to be accessible at its default URL.",
            AskOptions {
                level: AskLevel::Warning,
                default: Some(Answer::No),
                title: Some("Remove Custom Domain".into()),
                yes_text: Some("Yes, remove domain".into()),
                no_text: Some("Cancel".into()),
                ..Default::default()
            },
        )
        .await?;

        if !answer.is_yes() {
            message("ℹ️ Domain removal cancelled");
            return Ok(());
        }

        // Call API to clear domain
        delete_site_domain(&workspace_id).await?;

        // Clear domain from config
        config_unset("site.domain", ConfigTarget::Nearest)?;

        message!(
            "✅ Custom domain removed from site {}",
            default_site_url(&workspace_id, None)
        );

        Ok(())
    }
}

/// Manage branches for the workspace site
#[derive(Debug, Parser)]
#[command(after_long_help = BRANCH_AFTER_LONG_HELP)]
pub struct Branch {
    #[command(subcommand)]
    command: BranchCommand,
}

pub static BRANCH_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all deployed branches</dim>
  <b>stencila site branch list</>

  <dim># Delete a feature branch</dim>
  <b>stencila site branch delete feature-xyz</>

  <dim># Delete a branch without confirmation</dim>
  <b>stencila site branch delete feature-xyz --force</>
"
);

#[derive(Debug, Subcommand)]
enum BranchCommand {
    List(BranchList),
    Delete(BranchDelete),
}

impl Branch {
    pub async fn run(self) -> Result<()> {
        match self.command {
            BranchCommand::List(list) => list.run().await,
            BranchCommand::Delete(delete) => delete.run().await,
        }
    }
}

/// List all deployed branches
#[derive(Debug, Args)]
#[command(after_long_help = BRANCH_LIST_AFTER_LONG_HELP)]
pub struct BranchList {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static BRANCH_LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List branches for the current workspace's site</dim>
  <b>stencila site branch list</>

  <dim># List branches for another workspace's site</dim>
  <b>stencila site branch list --path /path/to/workspace</>
"
);

impl BranchList {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let workspace_id = cfg
            .workspace
            .and_then(|w| w.id)
            .ok_or_else(|| eyre!("No workspace configured for this directory"))?;
        let domain = cfg.site.and_then(|s| s.domain);

        // Fetch branch list from API
        let branches = list_site_branches(&workspace_id).await?;

        if branches.is_empty() {
            message("ℹ️ No branches have been deployed to this site yet");
            return Ok(());
        }

        // Display header message
        message!(
            "Deployed branches for site {}:\n",
            default_site_url(&workspace_id, domain.as_deref())
        );

        // Create and populate table
        let mut table = Tabulated::new();
        table.set_header(["Branch", "Files", "Size", "Last Updated"]);

        for branch in &branches {
            let size = format_size(branch.total_size);
            let modified = branch
                .last_modified
                .as_ref()
                .map(|s| format_timestamp(s))
                .unwrap_or_else(|| "Never".to_string());

            // Highlight main/master branches in green
            let branch_cell = if branch.name == "main" || branch.name == "master" {
                Cell::new(&branch.name).fg(Color::Green)
            } else {
                Cell::new(&branch.name)
            };

            table.add_row([
                branch_cell,
                Cell::new(branch.file_count).set_alignment(CellAlignment::Right),
                Cell::new(size).set_alignment(CellAlignment::Right),
                Cell::new(modified)
                    .fg(Color::Grey)
                    .set_alignment(CellAlignment::Right),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Delete a branch from the site
#[derive(Debug, Args)]
#[command(after_long_help = BRANCH_DELETE_AFTER_LONG_HELP)]
pub struct BranchDelete {
    /// The branch name to delete
    #[arg(value_name = "BRANCH_NAME")]
    branch_name: String,

    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,

    /// Skip confirmation prompt
    #[arg(long, short)]
    force: bool,
}

pub static BRANCH_DELETE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Delete a feature branch (with confirmation)</dim>
  <b>stencila site branch delete feature-xyz</>

  <dim># Delete a branch without confirmation</dim>
  <b>stencila site branch delete feature-xyz --force</>

  <dim># Delete a branch from another workspace</dim>
  <b>stencila site branch delete feature-xyz --path /path/to/workspace</>

<bold><b>Notes</b></bold>
  - Protected branches (main, master) cannot be deleted
  - Deletion is asynchronous and happens in the background
  - Cache will be purged automatically for the deleted branch
"
);

impl BranchDelete {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        // Check if trying to delete protected branches
        if self.branch_name == "main" || self.branch_name == "master" {
            bail!(
                "Cannot delete protected branch: {}. The main and master branches are protected.",
                self.branch_name
            );
        }

        let cfg = config(&path)?;
        let workspace_id = cfg
            .workspace
            .and_then(|w| w.id)
            .ok_or_else(|| eyre!("No workspace configured for this directory"))?;

        // Ask for confirmation unless --force is used
        if !self.force {
            let answer = ask_with(
                &format!(
                    "This will permanently delete all files for branch '{}' from your site. This cannot be undone.",
                    self.branch_name
                ),
                AskOptions {
                    level: AskLevel::Warning,
                    default: Some(Answer::No),
                    title: Some("Delete Branch".into()),
                    yes_text: Some("Yes, delete branch".into()),
                    no_text: Some("Cancel".into()),
                    ..Default::default()
                },
            )
            .await?;

            if !answer.is_yes() {
                message("ℹ️ Branch deletion cancelled");
                return Ok(());
            }
        }

        // Call API to delete branch
        delete_site_branch(&workspace_id, &self.branch_name).await?;

        message!(
            "✅ Branch '{}' deletion started. Files will be removed in the background.",
            self.branch_name
        );

        Ok(())
    }
}

/// Format bytes as human-readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format ISO 8601 timestamp as relative time or local date
fn format_timestamp(iso: &str) -> String {
    use chrono::{DateTime, Utc};

    if let Ok(dt) = iso.parse::<DateTime<Utc>>() {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt);

        if duration.num_seconds() < 60 {
            "Just now".to_string()
        } else if duration.num_minutes() < 60 {
            let mins = duration.num_minutes();
            format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
        } else if duration.num_hours() < 24 {
            let hours = duration.num_hours();
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        } else if duration.num_days() < 7 {
            let days = duration.num_days();
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        } else {
            dt.format("%Y-%m-%d %H:%M UTC").to_string()
        }
    } else {
        iso.to_string()
    }
}
