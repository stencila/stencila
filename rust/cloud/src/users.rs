use std::fs::read_to_string;

use common::{
    eyre::{bail, Result},
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    tokio::time::{sleep, Duration},
    tracing,
};
use fs_utils::remove_if_exists;
use http_utils::CLIENT;

use crate::{
    api,
    errors::*,
    types::*,
    utils::{token_path, token_read, token_write, user_path, user_write},
};

/// Get the currently authenticated user, if any
pub fn me() -> Result<Option<User>> {
    let path = user_path();
    if path.exists() {
        let json = read_to_string(path)?;
        let user: User = serde_json::from_str(&json)?;
        Ok(Some(user))
    } else {
        Ok(None)
    }
}

/// Login and return the authenticated user
///
/// Creates a voucher, opens the browser at the `/api/v1/vouchers?create=XXX` page,
/// and then starts polling `/api/v1/vouchers?redeem=XXX` to get the generated
/// API tokens.
pub async fn login() -> Result<User> {
    if let Some(user) = me()? {
        tracing::info!("Already logged in as @{}; use `stencila logout` first if you want to login as a different user", user.short_name);
        return Ok(user);
    }

    let voucher = key_utils::generate("svk");

    let create_url = api!("vouchers?create={}&tag=cli&note=Automatically%20generated%20for%20logins%20from%20Stencila%20CLI", voucher);
    tracing::info!("Opening login URL in browser: {}", create_url);
    webbrowser::open(&create_url)?;

    tracing::info!("Waiting for you to login in via browser");
    let redeem_url = api!("vouchers?redeem={}", voucher);
    loop {
        sleep(Duration::from_millis(1000)).await;

        let response = CLIENT.get(&redeem_url).send().await?;
        if response.status() == 200 {
            let token: ApiToken = response.json().await?;
            token_write(&token)?;

            let response = CLIENT
                .get(api!("me"))
                .bearer_auth(token.token)
                .send()
                .await?;
            let user: User = response.json().await?;
            user_write(&user)?;

            tracing::info!(
                "Welcome @{}, you successfully logged in to your Stencila account",
                user.short_name
            );

            return Ok(user);
        } else if response.status() == 202 {
            // Voucher exists but not ready to be redeemed
            eprint!(".");
        } else {
            bail!(
                "While redeeming voucher: {}",
                Error::response_to_string(response).await
            );
        }
    }
}

/// Logout the currently logged in user, if any
///
/// Deletes `token.json`, `user.json`, etc.
pub async fn logout() -> Result<()> {
    let user = me();

    remove_if_exists(token_path())?;
    remove_if_exists(user_path())?;

    match user {
        Ok(Some(user)) => {
            tracing::info!(
                "Goodbye @{}, you successfully logged out of your Stencila account",
                user.short_name
            );
        }
        Ok(None) => {
            tracing::info!("No user currently logged in");
        }
        _ => {
            // It doesn't matter if there was an error with `me()` call cause we deleted the files
            tracing::info!("Successfully logged out");
        }
    }
    Ok(())
}

pub async fn user_list(search: &str) -> Result<Vec<OrgPersonal>> {
    let response = CLIENT
        .get(api!("orgs"))
        .bearer_auth(token_read()?)
        .query(&[("type", "personal")])
        .query(&[("search", search)])
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct UserInvite {
    user: OrgPersonal,
    emailed: bool,
    link: String,
}

pub async fn user_invite(email: &str, no_send: bool) -> Result<UserInvite> {
    let response = CLIENT
        .post(api!("invite"))
        .bearer_auth(token_read()?)
        .json(&json!({
            "email": email,
            "send": !no_send
        }))
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub mod cli {
    use cli_utils::{
        clap::{self, Parser},
        cli_table::Title,
        common::async_trait::async_trait,
        result, Result, Run,
    };

    use super::*;

    /// Find and invite users
    #[derive(Parser)]
    #[clap(alias = "user")]
    pub struct Command {
        #[clap(subcommand)]
        action: Action,
    }

    #[derive(Parser)]
    enum Action {
        Login(Login),
        Logout(Logout),
        Me(Me),
        Find(Find),
        Invite(Invite),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::Me(action) => action.run().await,
                Action::Login(action) => action.run().await,
                Action::Logout(action) => action.run().await,
                Action::Find(action) => action.run().await,
                Action::Invite(action) => action.run().await,
            }
        }
    }

    /// Show the currently authenticated user
    #[derive(Parser)]
    #[clap(alias = "current")]
    pub struct Me;

    #[async_trait]
    impl Run for Me {
        async fn run(&self) -> Result {
            let user = me()?;
            if user.is_none() {
                tracing::info!("No user currently logged in");
            }
            result::value(user)
        }
    }

    /// Login to your Stencila account
    ///
    /// Use this command to link the Stencila CLI to your Stencila account.
    /// A browser window will be opened allowing you to sign in to Stencila, or
    /// create a Stencila account if you do not have one already. Once you have
    /// done that an access token will be stored on your machine allowing to access the
    /// Stencila API without having to sign in again.
    #[derive(Parser)]
    #[clap(alias = "signin")]
    pub struct Login;

    #[async_trait]
    impl Run for Login {
        async fn run(&self) -> Result {
            let user = login().await?;
            result::value(user)
        }
    }

    /// Logout from your Stencila account
    ///
    /// Use this command to unlink the Stencila CLI from your Stencila account.
    /// This will not affect your login status in the browser. i.e. if you are logged in
    /// to Stencila in your browser, this will not log you out there.
    #[derive(Parser)]
    #[clap(alias = "signin")]
    pub struct Logout;

    #[async_trait]
    impl Run for Logout {
        async fn run(&self) -> Result {
            logout().await?;
            result::nothing()
        }
    }

    /// Find users by name
    ///
    /// Use this command to search for Stencila users, for example, when you need their
    /// id to add them as a member on a project or organization.
    #[derive(Parser)]
    #[clap(alias = "search")]
    pub struct Find {
        /// A search string to filter users by
        search: String,
    }

    #[async_trait]
    impl Run for Find {
        async fn run(&self) -> Result {
            let users = user_list(&self.search).await?;
            result::table(users, OrgPersonal::title())
        }
    }

    /// Invite users by email or link
    ///
    /// Use this command when you want to invite someone who is not yet a Stencila user
    /// to join your project or organization.
    ///
    /// Stencila will send an invitation message to the user and generate a link that you can also personally
    /// send to them (this is advisable if you are concerned about spam filters catching the email).
    #[derive(Parser)]
    pub struct Invite {
        /// The email address of the user you wish to invite
        email: String,

        /// Do not send an email, just generate a link
        ///
        /// Use this option if you do not want Stencila to send an email and will
        /// send the invitee the link yourself.
        #[clap(long)]
        no_send: bool,
    }

    #[async_trait]
    impl Run for Invite {
        async fn run(&self) -> Result {
            let user_invite = user_invite(&self.email, self.no_send).await?;
            result::value(user_invite)
        }
    }
}
