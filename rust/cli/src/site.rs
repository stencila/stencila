use std::env::current_dir;

use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail};

use stencila_ask::{Answer, AskLevel, AskOptions, ask_for_password, ask_with};
use stencila_cli_utils::{color_print::cstr, message, parse_domain};
use stencila_cloud::sites::{
    SiteConfig, delete_site, delete_site_domain, ensure_site, get_site, get_site_domain_status,
    set_site_domain, update_site_access,
};

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

        let url = details.domain.as_deref().map_or_else(
            || config.default_url(),
            |domain| format!("https://{domain}"),
        );

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

        // Read site config to get site ID
        let config = SiteConfig::read(&path).await?;

        // Set the domain via API
        let response = set_site_domain(&config.id, &self.domain).await?;

        // Display appropriate message and instructions based on status
        match response.status.as_str() {
            "pending_dns" => {
                let cname_instructions =
                    format_cname_instructions(&response.cname_record, &response.cname_target);

                message(
                    &format!(
                        "Custom domain `{}` set for site `{}`\n\n\
                        To complete setup:\n\n\
                        1. {}\n\n\
                        2. Wait for DNS propagation (usually 5-30 minutes)\n\n\
                        3. Check status with: `stencila site domain status`\n\n\
                        Once the CNAME is detected, SSL will be provisioned automatically and your site will go live.",
                        response.domain, config.id, cname_instructions
                    ),
                    Some("⏳"),
                );
            }
            "ssl_initializing"
            | "ssl_pending_validation"
            | "ssl_pending_issuance"
            | "ssl_pending_deployment" => {
                message(
                    &format!("🔄 SSL provisioning started for `{}`", response.domain),
                    None,
                );
                if let Some(true) = response.cname_configured {
                    message(
                        "\nCNAME record detected! SSL certificate is being provisioned...\n\n\
                        Check status with: `stencila site domain status`",
                        None,
                    );
                } else {
                    let cname_instructions =
                        format_cname_instructions(&response.cname_record, &response.cname_target);

                    message(
                        &format!(
                            "\nTo complete setup:\n\n\
                            1. {}\n\n\
                            2. Monitor progress with: `stencila site domain status`",
                            cname_instructions
                        ),
                        None,
                    );
                }
            }
            "active" => {
                message(
                    &format!("Your site is now live at https://{}", response.domain),
                    Some("🎉"),
                );
            }
            "failed" => {
                bail!(
                    "Domain setup failed for `{}`. Run `stencila site domain status` for details.",
                    response.domain
                );
            }
            _ => {
                message(&format!("Status: {}", response.status), Some("🔄"));
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

        // Read site config to get site ID
        let config = SiteConfig::read(&path).await?;

        // Get domain status
        let status = get_site_domain_status(&config.id).await?;

        if !status.configured {
            message("No custom domain is configured for this site", Some("ℹ️"));
        } else if let Some("active") = status.status.as_deref()
            && let Some(domain) = &status.domain
        {
            message(
                &format!("Your site is live at https://{domain}"),
                Some("🎉"),
            );
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

            message(&parts.join("\n "), Some(emoji));
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

        // Read site config to get site ID
        let config = SiteConfig::read(&path).await?;

        // Check if a domain is configured before prompting
        let status = get_site_domain_status(&config.id).await?;
        if !status.configured {
            message("No custom domain is configured for this site", Some("ℹ️"));
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
            message("Domain removal cancelled", Some("ℹ️"));
            return Ok(());
        }

        // Call API to clear domain
        delete_site_domain(&config.id).await?;

        message(
            &format!("Custom domain removed from site {}", config.default_url()),
            Some("✅"),
        );

        Ok(())
    }
}
