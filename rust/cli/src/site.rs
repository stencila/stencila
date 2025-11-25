use std::env::current_dir;

use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail, eyre};

use stencila_ask::{Answer, AskLevel, AskOptions, ask_for_password, ask_with};
use stencila_cli_utils::{
    ToStdout,
    color_print::cstr,
    message, parse_domain,
    tabulated::{Cell, CellAlignment, Color, Tabulated},
};
use stencila_cloud::sites::{
    default_site_url, delete_site, delete_site_branch, delete_site_domain, ensure_site, get_site,
    get_site_domain_status, list_site_branches, set_site_domain, update_site_access,
};
use stencila_cloud::{AccessMode, WatchRequest, create_watch};
use stencila_codec_utils::git_info;
use stencila_config::{ConfigTarget, config, config_set, config_unset, config_update_site_watch};

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

  <dim># Create a site for the workspace</dim>
  <b>stencila site create</>

  <dim># Set site access to public</dim>
  <b>stencila site access public</>

  <dim># Set site access to password-protected</dim>
  <b>stencila site access password</>

  <dim># Set site access to team members only</dim>
  <b>stencila site access team</>

  <dim># Update the password (keeps current access mode)</dim>
  <b>stencila site password set</>

  <dim># Clear the password hash</dim>
  <b>stencila site password clear</>

  <dim># Delete the workspace site</dim>
  <b>stencila site delete</>
"
);

#[derive(Debug, Subcommand)]
enum SiteCommand {
    Show(Show),
    Create(Create),
    Delete(Delete),
    Access(Access),
    Password(Password),
    Domain(Domain),
    Branch(Branch),
}

impl Site {
    pub async fn run(self) -> Result<()> {
        let command = self
            .command
            .unwrap_or(SiteCommand::Show(Show { path: None }));

        match command {
            SiteCommand::Show(show) => show.run().await,
            SiteCommand::Create(create) => create.run().await,
            SiteCommand::Delete(delete) => delete.run().await,
            SiteCommand::Access(access) => access.run().await,
            SiteCommand::Password(password) => password.run().await,
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

        // Read site config to get site ID
        let cfg = config(&path)?;
        let site = match cfg.site {
            Some(site) => site,
            None => {
                message(cstr!(
                    "💡 No site is enabled for this workspace. To create one, run <b>stencila site create</>"
                ));
                return Ok(());
            }
        };
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("No site id in configuration"))?;

        // Fetch site details from API
        let details = get_site(site_id).await?;

        // Sync domain to config
        if let Some(domain) = &details.domain {
            config_set("site.domain", domain, ConfigTarget::Nearest)?;
        } else if site.domain.is_some() {
            // Domain was removed on cloud, clear it from config
            config_unset("site.domain", ConfigTarget::Nearest)?;
        }

        // Format ownership info
        let ownership = if details.org_id.is_some() {
            "Organization"
        } else {
            "User"
        };

        // Format access restriction
        let access = match details.access_restriction.as_str() {
            "public" => "Public".to_string(),
            "password" => {
                if details.access_restrict_main {
                    "Password protected".to_string()
                } else {
                    "Password protected (excluding main/master branches)".to_string()
                }
            }
            "auth" => "Team only".to_string(),
            other => format!("Unknown ({})", other),
        };

        let url = default_site_url(site_id, details.domain.as_deref());

        // Display site information
        let info = format!(
            "{}\n\
             \n\
             ID:             {}\n\
             Custom domain:  {}\n\
             Owned by:       {}\n\
             Created:        {}\n\
             Access:         {}\n\
             Access updated: {}",
            url,
            site_id,
            details.domain.as_deref().unwrap_or("None"),
            ownership,
            details.created_at,
            access,
            details.access_updated_at.as_deref().unwrap_or("Never")
        );

        message!("🌐 {info}");

        Ok(())
    }
}

/// Create a site for the workspace
#[derive(Debug, Args)]
#[command(after_long_help = CREATE_AFTER_LONG_HELP)]
pub struct Create {
    /// Root directory for site content
    ///
    /// If specified, sets the site.root config value. Files will be published
    /// from this directory, and routes calculated relative to it.
    /// Example: `stencila site create docs` publishes from ./docs/
    root: Option<std::path::PathBuf>,

    /// Path to the workspace directory where stencila.toml will be created
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,

    /// Set access restrictions for the site
    #[arg(long, short, value_enum)]
    access: Option<AccessMode>,

    /// Create a watch for automatic deployment
    ///
    /// When changes are pushed to the repository, the site is automatically
    /// updated. Requires a git repository with an origin remote.
    #[arg(long, short)]
    watch: bool,

    /// Set a custom domain for the site
    ///
    /// Example: --domain example.com
    #[arg(long, short, value_parser = parse_domain)]
    domain: Option<String>,
}

pub static CREATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create site for the current workspace</dim>
  <b>stencila site create</>

  <dim># Create site with docs/ as the root</dim>
  <b>stencila site create docs</>

  <dim># Create site with public access</dim>
  <b>stencila site create --access public</>

  <dim># Create site with password protection</dim>
  <b>stencila site create --access password</>

  <dim># Create site with team-only access</dim>
  <b>stencila site create --access team</>

  <dim># Create site with automatic deployment on git push</dim>
  <b>stencila site create --watch</>

  <dim># Create site with a custom domain</dim>
  <b>stencila site create --domain example.com</>

  <dim># Combine options</dim>
  <b>stencila site create docs --access public --watch --domain docs.example.com</>
"
);

impl Create {
    pub async fn run(self) -> Result<()> {
        let workspace_path = self.path.map_or_else(current_dir, Ok)?;

        // 1. Create the site
        let (site_id, already_existed) = ensure_site(&workspace_path).await?;

        // 2. Set site.root if provided
        if let Some(root) = &self.root {
            let root_str = root.to_string_lossy();
            config_set("site.root", root_str.as_ref(), ConfigTarget::Nearest)?;
        }

        // 3. Set access mode if provided
        if let Some(access) = self.access {
            match access {
                AccessMode::Public | AccessMode::Team => {
                    update_site_access(&site_id, Some(access), None, None).await?;
                }
                AccessMode::Password => {
                    let password = ask_for_password("Enter password for site access").await?;
                    update_site_access(&site_id, Some(access), Some(Some(password.as_str())), None)
                        .await?;
                }
            }
        }

        // 4. Set domain if provided
        let mut domain_instructions: Option<String> = None;
        if let Some(domain) = &self.domain {
            let response = set_site_domain(&site_id, domain).await?;
            config_set("site.domain", domain, ConfigTarget::Nearest)?;

            // Prepare CNAME instructions if DNS not yet configured
            if response.status == "pending_dns" {
                domain_instructions = Some(format_cname_instructions(
                    &response.cname_record,
                    &response.cname_target,
                ));
            }
        }

        // 5. Create watch if requested
        if self.watch {
            // Check git remote exists
            let site_path = self
                .root
                .as_ref()
                .map(|r| workspace_path.join(r))
                .unwrap_or_else(|| workspace_path.clone());

            let git_info = git_info(&site_path)?;
            let repo_url = git_info
                .origin
                .ok_or_else(|| eyre!("--watch requires a git repository with an origin remote"))?;

            // Check not already watched
            let cfg = config(&workspace_path)?;
            if cfg.site.as_ref().and_then(|s| s.watch.as_ref()).is_none() {
                // Get directory path relative to repo root (must end with / for API)
                let dir_path = git_info.path.unwrap_or_else(|| ".".to_string());
                let dir_path = if dir_path.ends_with('/') {
                    dir_path
                } else {
                    format!("{dir_path}/")
                };

                // Build the site URL
                let site_url = format!("https://{site_id}.stencila.site");

                let request = WatchRequest {
                    remote_url: site_url,
                    repo_url,
                    file_path: dir_path,
                    direction: Some("to-remote".to_string()),
                    pr_mode: None,
                    debounce_seconds: None,
                };

                let response = create_watch(request).await?;
                config_update_site_watch(&workspace_path, Some(response.id))?;
            }
        }

        // 6. Display success message
        let cfg = config(&workspace_path)?;
        let domain = cfg.site.and_then(|s| s.domain);
        let url = default_site_url(&site_id, domain.as_deref());

        if already_existed {
            message!("ℹ️ Site already exists: {url}");
        } else {
            message!("✅ Site created: {url}");
        }

        // Show additional status for new options
        if let Some(access) = &self.access {
            message!("   Access: {access}");
        }
        if self.watch {
            message!("   Watch: enabled");
        }
        if let Some(domain) = &self.domain {
            message!("   Domain: {domain}");
        }

        // Show CNAME instructions if domain was set and needs DNS configuration
        if let Some(instructions) = domain_instructions {
            message!("\n{instructions}");
        }

        Ok(())
    }
}

/// Delete the site for the workspace
#[derive(Debug, Args)]
#[command(after_long_help = DELETE_AFTER_LONG_HELP)]
pub struct Delete {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static DELETE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Delete site for current workspace</dim>
  <b>stencila site delete</>

  <dim># Delete site for another workspace</dim>
  <b>stencila site delete --path /path/to/workspace</>
"
);

impl Delete {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        // Ask for confirmation with warning level
        let answer = ask_with(
            "This will permanently delete the site on Stencila Cloud including all content. This cannot be undone.",
            AskOptions {
                level: AskLevel::Warning,
                default: Some(Answer::No),
                title: Some("Delete Stencila Site".into()),
                yes_text: Some("Yes, delete".into()),
                no_text: Some("Cancel".into()),
                ..Default::default()
            },
        )
        .await?;

        if !answer.is_yes() {
            message("ℹ️ Site deletion cancelled");
            return Ok(());
        }

        let site_id = delete_site(&path).await?;

        // Clean up implicit remotes from remotes.json
        let stencila_dir = stencila_dirs::closest_stencila_dir(&path, false).await?;
        if let Ok(removed_count) =
            stencila_remotes::remove_site_remotes(&stencila_dir, &site_id).await
            && removed_count > 0
        {
            tracing::debug!("Removed {removed_count} remote tracking entries");
        }

        message("✅ Site deleted successfully");

        Ok(())
    }
}

/// Manage access restrictions for the workspace site
#[derive(Debug, Parser)]
#[command(after_long_help = ACCESS_AFTER_LONG_HELP)]
pub struct Access {
    #[command(subcommand)]
    command: Option<AccessCommand>,
}

pub static ACCESS_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Show current access mode</dim>
  <b>stencila site access</>

  <dim># Switch to public access</dim>
  <b>stencila site access public</>

  <dim># Switch to password-protected access</dim>
  <b>stencila site access password</>

  <dim># Switch to team-only access</dim>
  <b>stencila site access team</>
"
);

#[derive(Debug, Subcommand)]
enum AccessCommand {
    Public(AccessPublic),
    Password(AccessPassword),
    Team(AccessTeam),
}

impl Access {
    pub async fn run(self) -> Result<()> {
        let path = current_dir()?;

        // If no subcommand, show current access mode
        let Some(command) = self.command else {
            let cfg = config(&path)?;
            let site = match cfg.site {
                Some(site) => site,
                None => {
                    message(cstr!(
                        "💡 No site is enabled for this workspace. To create one, run <b>stencila site create</>"
                    ));
                    return Ok(());
                }
            };
            let site_id = site
                .id
                .as_ref()
                .ok_or_else(|| eyre!("Site ID not set in configuration"))?;

            let details = get_site(site_id).await?;

            let access = match details.access_restriction.as_str() {
                "public" => "Public",
                "password" => {
                    if details.access_restrict_main {
                        "Password protected"
                    } else {
                        "Password protected (excluding main/master branches)"
                    }
                }
                "auth" => "Team only",
                other => other,
            };

            message!("ℹ️ Access mode: {}", access);
            return Ok(());
        };

        match command {
            AccessCommand::Public(public) => public.run().await,
            AccessCommand::Password(password) => password.run().await,
            AccessCommand::Team(team) => team.run().await,
        }
    }
}

/// Switch to public access
#[derive(Debug, Args)]
#[command(after_long_help = ACCESS_PUBLIC_AFTER_LONG_HELP)]
pub struct AccessPublic {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static ACCESS_PUBLIC_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Switch to public access</dim>
  <b>stencila site access public</>

  <dim># Switch for another workspace</dim>
  <b>stencila site access public --path /path/to/workspace</>
"
);

impl AccessPublic {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;
        let domain = site.domain.as_deref();

        update_site_access(site_id, Some(AccessMode::Public), None, None).await?;

        message!(
            "✅ Site {} switched to public access",
            default_site_url(site_id, domain)
        );

        Ok(())
    }
}

/// Switch to password-protected access
#[derive(Debug, Args)]
#[command(after_long_help = ACCESS_PASSWORD_AFTER_LONG_HELP)]
pub struct AccessPassword {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,

    /// Do not apply password protection to main or master branches
    ///
    /// By default, the password applies to all branches including main and master.
    /// Use this flag to exclude main and master branches from password protection.
    #[arg(long)]
    not_main: bool,
}

pub static ACCESS_PASSWORD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Switch to password-protected access</dim>
  <b>stencila site access password</>

  <dim># Exclude main/master branches from password protection</dim>
  <b>stencila site access password --not-main</>
"
);

impl AccessPassword {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;
        let domain = site.domain.as_deref();

        // Set password_for_main based on the flag
        let access_restrict_main = if self.not_main {
            Some(false)
        } else {
            Some(true)
        };

        // First, try to switch to password mode without prompting for password
        // This will succeed if a password hash already exists in the database
        let result = update_site_access(
            site_id,
            Some(AccessMode::Password),
            None,
            access_restrict_main,
        )
        .await;

        match result {
            Ok(()) => {
                // Successfully switched to password mode using existing password hash
                message!(
                    "✅ Site {} switched to password-protected access{}",
                    default_site_url(site_id, domain),
                    if self.not_main {
                        " (excluding main/master branches)"
                    } else {
                        ""
                    }
                );
                Ok(())
            }
            Err(err) => {
                // Check if error is about missing password (400 error)
                let err_msg = err.to_string();
                if err_msg.contains("400") || err_msg.contains("password") {
                    // Password is required - prompt user for it
                    let password = ask_for_password(cstr!(
                        "Enter password to protect your site (will not be displayed)"
                    ))
                    .await?;

                    // Retry with password
                    update_site_access(
                        site_id,
                        Some(AccessMode::Password),
                        Some(Some(&password)),
                        access_restrict_main,
                    )
                    .await?;

                    message!(
                        "✅ Site {} switched to password-protected access{}",
                        default_site_url(site_id, domain),
                        if self.not_main {
                            " (excluding main/master branches)"
                        } else {
                            ""
                        }
                    );

                    Ok(())
                } else {
                    // Some other error - return it
                    Err(err)
                }
            }
        }
    }
}

/// Switch to team-only access
#[derive(Debug, Args)]
#[command(after_long_help = ACCESS_TEAM_AFTER_LONG_HELP)]
pub struct AccessTeam {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,

    /// Do not apply team restriction to main or master branches
    ///
    /// By default, team restriction applies to all branches including main and master.
    /// Use this flag to exclude main and master branches from team restriction.
    #[arg(long)]
    not_main: bool,

    /// Apply team restriction to main or master branches
    ///
    /// Updates the accessRestrictMain flag. Use this to re-enable restriction
    /// for main and master branches.
    #[arg(long, conflicts_with = "not_main")]
    main: bool,
}

pub static ACCESS_TEAM_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Switch to team-only access</dim>
  <b>stencila site access team</>

  <dim># Exclude main/master branches from team restriction</dim>
  <b>stencila site access team --not-main</>

  <dim># Switch for another workspace</dim>
  <b>stencila site access team --path /path/to/workspace</>
"
);

impl AccessTeam {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;
        let domain = site.domain.as_deref();

        // Determine accessRestrictMain value if flags are provided
        let access_restrict_main = if self.main {
            Some(true)
        } else if self.not_main {
            Some(false)
        } else {
            None
        };

        update_site_access(site_id, Some(AccessMode::Team), None, access_restrict_main).await?;

        message!(
            "✅ Site {} switched to team-only access{}",
            default_site_url(site_id, domain),
            if self.not_main {
                " (excluding main/master branches)"
            } else if self.main {
                " (including main/master branches)"
            } else {
                ""
            }
        );

        Ok(())
    }
}

/// Manage password protection for the workspace site
#[derive(Debug, Parser)]
#[command(after_long_help = PASSWORD_AFTER_LONG_HELP)]
pub struct Password {
    #[command(subcommand)]
    command: PasswordCommand,
}

pub static PASSWORD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Set or update the password (keeps current access mode)</dim>
  <b>stencila site password set</>

  <dim># Set password and update main branch restriction</dim>
  <b>stencila site password set --not-main</>

  <dim># Clear the password hash</dim>
  <b>stencila site password clear</>
"
);

#[derive(Debug, Subcommand)]
enum PasswordCommand {
    #[command(alias = "add")]
    Set(PasswordSet),

    #[command(alias = "remove", alias = "rm")]
    Clear(PasswordClear),
}

impl Password {
    pub async fn run(self) -> Result<()> {
        match self.command {
            PasswordCommand::Set(set) => set.run().await,
            PasswordCommand::Clear(clear) => clear.run().await,
        }
    }
}

/// Set or update the password (without changing access mode)
#[derive(Debug, Args)]
#[command(after_long_help = PASSWORD_SET_AFTER_LONG_HELP)]
pub struct PasswordSet {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,

    /// Do not apply password protection to main or master branches
    ///
    /// Updates the accessRestrictMain flag. By default, password protection applies
    /// to all branches including main and master. Use this flag to exclude them.
    #[arg(long)]
    not_main: bool,

    /// Apply password protection to main or master branches
    ///
    /// Updates the accessRestrictMain flag. Use this to re-enable protection
    /// for main and master branches.
    #[arg(long, conflicts_with = "not_main")]
    main: bool,
}

pub static PASSWORD_SET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Update the password (keeps current access mode)</dim>
  <b>stencila site password set</>

  <dim># Set password for another workspace</dim>
  <b>stencila site password set --path /path/to/workspace</>

  <dim># Update password and exclude main/master branches</dim>
  <b>stencila site password set --not-main</>

  <dim># Update password and include main/master branches</dim>
  <b>stencila site password set --main</>
"
);

impl PasswordSet {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;
        let domain = site.domain.as_deref();

        // Prompt for password securely
        let password = ask_for_password(cstr!(
            "Enter password for your site (will not be displayed)"
        ))
        .await?;

        // Determine if we should update accessRestrictMain
        let access_restrict_main = if self.main {
            Some(true)
        } else if self.not_main {
            Some(false)
        } else {
            None
        };

        // Update password only (preserve current access mode)
        update_site_access(
            site_id,
            None,                  // Don't change access mode
            Some(Some(&password)), // Update password
            access_restrict_main,  // Update main flag if specified
        )
        .await?;

        let mode_msg = if self.main {
            " (now protecting main/master branches)"
        } else if self.not_main {
            " (now excluding main/master branches)"
        } else {
            ""
        };

        message!(
            "✅ Password updated for {}{}",
            default_site_url(site_id, domain),
            mode_msg
        );

        Ok(())
    }
}

/// Clear the password (keeps access mode unchanged)
#[derive(Debug, Args)]
#[command(after_long_help = PASSWORD_CLEAR_AFTER_LONG_HELP)]
pub struct PasswordClear {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static PASSWORD_CLEAR_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Clear password for the current workspace's site</dim>
  <b>stencila site password clear</>

  <dim># Clear password for another workspace's site</dim>
  <b>stencila site password clear --path /path/to/workspace</>
"
);

impl PasswordClear {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let cfg = config(&path)?;
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;
        let domain = site.domain.as_deref();

        // Ask for confirmation
        let answer = ask_with(
            "This will clear the password from your site. The access mode will remain unchanged.",
            AskOptions {
                level: AskLevel::Warning,
                default: Some(Answer::No),
                title: Some("Clear Password Hash".into()),
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

        // Call API to clear password (pass Some(None) to explicitly set password to null)
        update_site_access(site_id, None, Some(None), None).await?;

        message!(
            "✅ Password cleared from {}",
            default_site_url(site_id, domain)
        );

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
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;

        // Set the domain via API
        let response = set_site_domain(site_id, &self.domain).await?;

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
                    site_id,
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
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;

        // Get domain status
        let status = get_site_domain_status(site_id).await?;

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

            // Add CNAME instructions for pending_dns status
            if let Some("pending_dns") = status.status.as_deref()
                && let Some(cname_record) = &status.cname_record
                && let Some(cname_target) = &status.cname_target
            {
                parts.push(String::new()); // Empty line
                parts.push(format_cname_instructions(cname_record, cname_target));
            }

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
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;

        // Check if a domain is configured before prompting
        let status = get_site_domain_status(site_id).await?;
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
        delete_site_domain(site_id).await?;

        // Clear domain from config
        config_unset("site.domain", ConfigTarget::Nearest)?;

        message!(
            "✅ Custom domain removed from site {}",
            default_site_url(site_id, None)
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
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;
        let domain = site.domain.as_deref();

        // Fetch branch list from API
        let branches = list_site_branches(site_id).await?;

        if branches.is_empty() {
            message("ℹ️ No branches have been deployed to this site yet");
            return Ok(());
        }

        // Display header message
        message!(
            "Deployed branches for site {}:\n",
            default_site_url(site_id, domain)
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
        let site = cfg
            .site
            .ok_or_else(|| eyre!("No site configured for this workspace"))?;
        let site_id = site
            .id
            .as_ref()
            .ok_or_else(|| eyre!("Site ID not set in configuration"))?;

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
        delete_site_branch(site_id, &self.branch_name).await?;

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
