use std::{fs::read_to_string, io::Write};

use common::{
    chrono::{self, Utc},
    eyre::{bail, Result},
    serde_json::{self, json},
    tokio::time::{sleep, Duration},
    tracing,
};
use fs_utils::{open_file_600, remove_if_exists};
use http_utils::CLIENT;

use crate::{
    errors::*,
    types::*,
    utils::{token_path, token_read, user_path, BASE_URL},
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

    let create_url = format!("{}/vouchers?create={}&tag=cli&note=Automatically%20generated%20for%20logins%20from%20Stencila%20CLI", BASE_URL, voucher);
    tracing::info!("Opening login URL in browser: {}", create_url);
    webbrowser::open(&create_url)?;

    tracing::info!("Waiting for you to login in via browser");
    let redeem_url = format!("{}/vouchers?redeem={}", BASE_URL, voucher);
    loop {
        sleep(Duration::from_millis(1000)).await;

        let response = CLIENT.get(&redeem_url).send().await?;
        if response.status() == 200 {
            let token: ApiToken = response.json().await?;
            let json = serde_json::to_string_pretty(&token)?;
            let mut file = open_file_600(token_path())?;
            file.write_all(json.as_bytes())?;

            let response = CLIENT
                .get(format!("{}/me", BASE_URL))
                .bearer_auth(token.token)
                .send()
                .await?;
            let user: User = response.json().await?;
            let json = serde_json::to_string_pretty(&user)?;
            let mut file = open_file_600(user_path())?;
            file.write_all(json.as_bytes())?;

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
    match me()? {
        Some(user) => {
            remove_if_exists(token_path())?;
            remove_if_exists(user_path())?;

            tracing::info!(
                "Goodbye @{}, you successfully logged out of your Stencila account",
                user.short_name
            );
        }
        None => {
            tracing::info!("No user currently logged in");
        }
    }
    Ok(())
}

pub async fn token_list() -> Result<Vec<ApiToken>> {
    let response = CLIENT
        .get(format!("{}/tokens", BASE_URL))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub async fn token_create(
    tag: Option<&str>,
    note: Option<&str>,
    expires_in: Option<u64>,
) -> Result<ApiToken> {
    let data = json!({
        "tag": tag,
        "note": note,
        "expires_at": expires_in.map(|minutes| Utc::now() + chrono::Duration::minutes(minutes as i64))
    });
    let response = CLIENT
        .post(format!("{}/tokens", BASE_URL))
        .bearer_auth(token_read()?)
        .json(&data)
        .send()
        .await?;
    if response.status().is_success() {
        tracing::info!("Successfully created token");
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub async fn token_delete(id: u64) -> Result<()> {
    let response = CLIENT
        .delete(format!("{}/tokens/{}", BASE_URL, id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        tracing::info!("Successfully deleted token");
        Ok(())
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

    /// Show the currently authenticated user
    #[derive(Parser)]
    #[clap(alias = "user")]
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

    pub mod tokens {
        use super::*;

        /// Manage your personal access tokens
        #[derive(Parser)]
        #[clap(alias = "token")]
        pub struct Command {
            #[clap(subcommand)]
            action: Action,
        }

        #[derive(Parser)]
        enum Action {
            List(List),
            Create(Create),
            Delete(Delete),
        }

        #[async_trait]
        impl Run for Command {
            async fn run(&self) -> Result {
                match &self.action {
                    Action::List(action) => action.run().await,
                    Action::Create(action) => action.run().await,
                    Action::Delete(action) => action.run().await,
                }
            }
        }

        /// List your personal access tokens
        ///
        /// Use this command to retrieve the details of the tokens created by you
        /// or on your behalf when signing in using Stencila API clients. Note that
        /// you can not retrieve the actual token itself (that is only available when
        /// you create it).
        #[derive(Default, Parser)]
        struct List;

        #[async_trait]
        impl Run for List {
            async fn run(&self) -> Result {
                let members = token_list().await?;
                result::table(members, ApiToken::title())
            }
        }

        /// Create a new personal access token
        ///
        /// Use this command to create a token for accessing the Stencila API on your behalf.
        /// Store tokens securely.
        #[derive(Parser)]
        struct Create {
            /// A note for the token
            ///
            /// This option is useful for remembering why you created a token and whether
            /// you can safely delete it in the future.
            #[clap(long, short)]
            note: Option<String>,

            /// The number of minutes until the token should expire
            ///
            /// Use this option if you want the new token to expire after a certain amount
            /// of time.
            #[clap(long, short)]
            expires_in: Option<u64>,

            /// A tag for the token
            ///
            /// Tags are used to identify a token created for a specific client or purpose.
            /// They avoid the generation of multiple, redundant tokens. You probably do not
            /// need to set a tag when manually creating a token.
            #[clap(long, short)]
            tag: Option<String>,
        }

        #[async_trait]
        impl Run for Create {
            async fn run(&self) -> Result {
                let token =
                    token_create(self.tag.as_deref(), self.note.as_deref(), self.expires_in)
                        .await?;
                result::value(token)
            }
        }

        /// Delete a personal access token
        ///
        /// Use this command to permanently delete an access token. Take care as any clients or services still
        /// relying on the token (including this CLI!) may be interrupted.
        #[derive(Parser)]
        #[clap(alias = "revoke")]
        struct Delete {
            /// The id of the token
            id: u64,
        }

        #[async_trait]
        impl Run for Delete {
            async fn run(&self) -> Result {
                token_delete(self.id).await?;
                result::nothing()
            }
        }
    }
}
