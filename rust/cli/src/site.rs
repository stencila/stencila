use std::env::current_dir;

use clap::{Args, Parser, Subcommand};
use eyre::Result;

use stencila_ask::{Answer, AskLevel, AskOptions, ask_for_password, ask_with};
use stencila_cli_utils::{color_print::cstr, message};

/// Manage the workspace site
#[derive(Debug, Parser)]
#[command(alias = "sites", after_long_help = AFTER_LONG_HELP)]
pub struct Site {
    #[command(subcommand)]
    command: SiteCommand,
}

pub static AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Create a site for the workspace</dim>
  <b>stencila site create</>

  <dim># Set password protection for the site</dim>
  <b>stencila site password set</>

  <dim># Remove password protection from the site</dim>
  <b>stencila site password remove</>

  <dim># Delete the workspace site</dim>
  <b>stencila site delete</>
"
);

#[derive(Debug, Subcommand)]
enum SiteCommand {
    Create(Create),
    Delete(Delete),
    Password(Password),
}

impl Site {
    pub async fn run(self) -> Result<()> {
        match self.command {
            SiteCommand::Create(create) => create.run().await,
            SiteCommand::Delete(delete) => delete.run().await,
            SiteCommand::Password(password) => password.run().await,
        }
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
        let path = self.path.map_or_else(|| current_dir(), Ok)?;

        let (config, already_existed) = stencila_cloud::sites::ensure_site(&path).await?;
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
        let path = self.path.map_or_else(|| current_dir(), Ok)?;

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

        stencila_cloud::sites::delete_site(&path).await?;

        message("Site deleted successfully", Some("✅"));

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
  <dim># Set password for the current workspace site</dim>
  <b>stencila site password set</>

  <dim># Set password but not for main or master branches</dim>
  <b>stencila site password set --not-main</>

  <dim># Remove password protection</dim>
  <b>stencila site password remove</>
"
);

#[derive(Debug, Subcommand)]
enum PasswordCommand {
    #[command(alias = "add")]
    Set(PasswordSet),

    #[command(alias = "rm")]
    Remove(PasswordRemove),
}

impl Password {
    pub async fn run(self) -> Result<()> {
        match self.command {
            PasswordCommand::Set(set) => set.run().await,
            PasswordCommand::Remove(remove) => remove.run().await,
        }
    }
}

/// Set password protection for a Stencila Site
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
    /// By default, the password applies to all branches including main and master.
    /// Use this flag to exclude main and master branches from password protection,
    /// allowing them to remain publicly accessible while protecting other branches.
    #[arg(long)]
    not_main: bool,
}

pub static PASSWORD_SET_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Set password for the current workspace's site</dim>
  <b>stencila site password set</>

  <dim># Set password for another workspace's site</dim>
  <b>stencila site password set --path /path/to/workspace</>

  <dim># Set password but not for main or master branches</dim>
  <b>stencila site password set --not-main</>
"
);

impl PasswordSet {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(|| current_dir(), Ok)?;

        // Read site config to get site ID
        let config = stencila_cloud::sites::SiteConfig::read(&path).await?;

        // Prompt for password securely
        let password = ask_for_password(cstr!(
            "Enter password to protect your site (will not be displayed)"
        ))
        .await?;

        // Set password_for_main based on the flag (true by default, false if --not-main)
        let password_for_main = !self.not_main;

        // Call API to set password
        stencila_cloud::sites::set_site_password(&config.id, &password, password_for_main).await?;

        message(
            &format!(
                "Password protection enabled for {} {}",
                config.default_url(),
                if !password_for_main {
                    "(excluding main and master branches)"
                } else {
                    ""
                }
            ),
            Some("✅"),
        );

        Ok(())
    }
}

/// Remove password protection from a Stencila Site
#[derive(Debug, Args)]
#[command(after_long_help = PASSWORD_REMOVE_AFTER_LONG_HELP)]
pub struct PasswordRemove {
    /// Path to the workspace directory containing .stencila/site.yaml
    ///
    /// If not specified, uses the current directory
    #[arg(long, short)]
    path: Option<std::path::PathBuf>,
}

pub static PASSWORD_REMOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove password for the current workspace's site</dim>
  <b>stencila site password remove</>

  <dim># Remove password for another workspace's site</dim>
  <b>stencila site password remove --path /path/to/workspace</>
"
);

impl PasswordRemove {
    pub async fn run(self) -> Result<()> {
        let path = self.path.map_or_else(|| current_dir(), Ok)?;

        // Read site config to get site ID
        let config = stencila_cloud::sites::SiteConfig::read(&path).await?;

        // Ask for confirmation
        let answer = ask_with(
            "This will remove password protection from your site, making it publicly accessible.",
            AskOptions {
                level: AskLevel::Warning,
                default: Some(Answer::No),
                title: Some("Remove Password Protection".into()),
                yes_text: Some("Yes, remove password".into()),
                no_text: Some("Cancel".into()),
                ..Default::default()
            },
        )
        .await?;

        if !answer.is_yes() {
            message("Password removal cancelled", Some("ℹ️"));
            return Ok(());
        }

        // Call API to remove password
        stencila_cloud::sites::remove_site_password(&config.id).await?;

        message(
            &format!("Password protection removed from {}", config.default_url()),
            Some("✅"),
        );

        Ok(())
    }
}
