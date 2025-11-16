use std::env::current_dir;

use clap::{Args, Parser, Subcommand};
use eyre::Result;

use stencila_ask::{Answer, AskLevel, AskOptions, ask_for_password, ask_with};
use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::sites::{SiteConfig, delete_site, ensure_site, get_site, update_site_access};

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
        let config = match SiteConfig::read(&path).await {
            Ok(config) => config,
            Err(_) => {
                message(
                    cstr!(
                        "No site is enabled for this workspace. To create one, run <b>stencila site create</>"
                    ),
                    Some("💡"),
                );
                return Ok(());
            }
        };

        // Fetch site details from API
        let details = get_site(&config.id).await?;

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
            config.default_url(),
            config.id,
            details.domain.as_deref().unwrap_or("None"),
            ownership,
            details.created_at,
            access,
            details.access_updated_at
        );

        message(&info, Some("🌐"));

        Ok(())
    }
}

/// Create a site for the workspace
#[derive(Debug, Args)]
#[command(after_long_help = CREATE_AFTER_LONG_HELP)]
pub struct Create {
    /// Path to the workspace directory where .stencila/site.yaml will be created
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static CREATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create site for the current workspace</dim>
  <b>stencila site create</>

  <dim># Create site for another workspace</dim>
  <b>stencila site create --path /path/to/workspace</>
"
);

impl Create {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(current_dir, Ok)?;

        let (config, already_existed) = ensure_site(&path).await?;
        let url = config.default_url();

        if already_existed {
            message(
                &format!("Site already exists for workspace: {url}"),
                Some("ℹ️"),
            );
        } else {
            message(
                &format!("Site successfully created for workspace: {url}"),
                Some("✅"),
            );
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
            message("Site deletion cancelled", Some("ℹ️"));
            return Ok(());
        }

        delete_site(&path).await?;

        message("Site deleted successfully", Some("✅"));

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
            let config = match SiteConfig::read(&path).await {
                Ok(config) => config,
                Err(_) => {
                    message(
                        cstr!(
                            "No site is enabled for this workspace. To create one, run <b>stencila site create</>"
                        ),
                        Some("💡"),
                    );
                    return Ok(());
                }
            };

            let details = get_site(&config.id).await?;

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

            message(&format!("Access mode: {}", access), Some("ℹ️"));
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

        let config = SiteConfig::read(&path).await?;

        update_site_access(&config.id, Some("public"), None, None).await?;

        message(
            &format!("Site {} switched to public access", config.default_url()),
            Some("✅"),
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

        let config = SiteConfig::read(&path).await?;

        // Set password_for_main based on the flag
        let access_restrict_main = if self.not_main {
            Some(false)
        } else {
            Some(true)
        };

        // First, try to switch to password mode without prompting for password
        // This will succeed if a password hash already exists in the database
        let result =
            update_site_access(&config.id, Some("password"), None, access_restrict_main).await;

        match result {
            Ok(()) => {
                // Successfully switched to password mode using existing password hash
                message(
                    &format!(
                        "Site {} switched to password-protected access{}",
                        config.default_url(),
                        if self.not_main {
                            " (excluding main/master branches)"
                        } else {
                            ""
                        }
                    ),
                    Some("✅"),
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
                        &config.id,
                        Some("password"),
                        Some(Some(&password)),
                        access_restrict_main,
                    )
                    .await?;

                    message(
                        &format!(
                            "Site {} switched to password-protected access{}",
                            config.default_url(),
                            if self.not_main {
                                " (excluding main/master branches)"
                            } else {
                                ""
                            }
                        ),
                        Some("✅"),
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

        let config = SiteConfig::read(&path).await?;

        // Determine accessRestrictMain value if flags are provided
        let access_restrict_main = if self.main {
            Some(true)
        } else if self.not_main {
            Some(false)
        } else {
            None
        };

        update_site_access(&config.id, Some("auth"), None, access_restrict_main).await?;

        message(
            &format!(
                "Site {} switched to team-only access{}",
                config.default_url(),
                if self.not_main {
                    " (excluding main/master branches)"
                } else if self.main {
                    " (including main/master branches)"
                } else {
                    ""
                }
            ),
            Some("✅"),
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

        // Read site config to get site ID
        let config = SiteConfig::read(&path).await?;

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
            &config.id,
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

        message(
            &format!("Password updated for {}{}", config.default_url(), mode_msg),
            Some("✅"),
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

        // Read site config to get site ID
        let config = SiteConfig::read(&path).await?;

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
            message("Password clear cancelled", Some("ℹ️"));
            return Ok(());
        }

        // Call API to clear password (pass Some(None) to explicitly set password to null)
        update_site_access(&config.id, None, Some(None), None).await?;

        message(
            &format!("Password cleared from {}", config.default_url()),
            Some("✅"),
        );

        Ok(())
    }
}
