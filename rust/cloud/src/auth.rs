use std::{env, fs::read_to_string};

use common::{
    chrono::{self, Utc},
    eyre::{bail, Result},
    serde_json::{self, json},
    tokio::time::{sleep, Duration, Instant},
    tracing,
};
use fs_utils::remove_if_exists;
use http_utils::{reqwest, CLIENT};

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

pub async fn token_list() -> Result<Vec<ApiToken>> {
    let response = CLIENT
        .get(api!("tokens"))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
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
        .post(api!("tokens"))
        .bearer_auth(token_read()?)
        .json(&data)
        .send()
        .await?;
    if response.status().is_success() {
        tracing::info!("Successfully created token");
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
    }
}

pub async fn token_delete(id: u64) -> Result<()> {
    let response = CLIENT
        .delete(api!("tokens/{}", id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        tracing::info!("Successfully deleted token");
        Ok(())
    } else {
        Error::from_response(response).await
    }
}

pub async fn provider_list() -> Result<Vec<Provider>> {
    let response = CLIENT
        .get(api!("oauth/providers"))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
    }
}

pub async fn provider_create(provider: &str) -> Result<()> {
    let client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let response = client
        .post(api!("oauth/providers?provider={}", provider.to_lowercase()))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status() == 302 {
        let url = response.text().await?;
        tracing::info!(
            "Opening authorization page for provider `{}`: {}",
            provider,
            url
        );
        webbrowser::open(&url)?;
        Ok(())
    } else {
        Error::from_response(response).await
    }
}

pub async fn provider_delete(provider: &str) -> Result<()> {
    let response = CLIENT
        .delete(api!("oauth/providers?provider={}", provider.to_lowercase()))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        tracing::info!(
            "Successfully disconnected account for provider `{}`",
            provider
        );
        Ok(())
    } else {
        Error::from_response(response).await
    }
}

/// Get a token for a provider
///
/// Attempts to get a token for the provider, in order of preference
///
/// - an environment variable e.g. `GITHUB_TOKEN`
/// - an existing connection `api/oauth/token?provider=<provider>` (will be refreshed if necessary)
/// - a new connection: calls the `provider_create` function and polls for up to a minute for
///   a token
pub async fn provider_token(provider: &str) -> Result<ProviderToken> {
    // Check for env var
    if let Ok(token) = env::var([provider, "_TOKEN"].concat().to_uppercase()) {
        return Ok(ProviderToken {
            provider: provider.to_lowercase(),
            access_token: token,
            expires_at: None,
        });
    }

    // Attempt to fetch token
    async fn fetch(provider: &str) -> Result<ProviderToken> {
        let response = CLIENT
            .get(api!("oauth/token?provider={}", provider.to_lowercase()))
            .bearer_auth(token_read()?)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Error::from_response(response).await
        }
    }
    match fetch(provider).await {
        Ok(token) => return Ok(token),
        Err(error) => {
            tracing::warn!("{}", error)
        }
    }

    // Prompt user to add provider and poll for a token
    provider_create(provider).await?;
    tracing::info!("Waiting for you to connect your account to `{}`", provider);
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        sleep(Duration::from_millis(1500)).await;
        if let Ok(token) = fetch(provider).await {
            eprint!("\n\n");
            return Ok(token);
        }

        eprint!(".");

        if Instant::now() >= deadline {
            tracing::error!("Giving up waiting for access token for `{}`", provider)
        }
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

    /// Login/out and manage tokens and authentication providers
    #[derive(Parser)]
    pub struct Command {
        #[clap(subcommand)]
        action: Action,
    }

    #[derive(Parser)]
    enum Action {
        Login(Login),
        Logout(Logout),
        Me(Me),
        Tokens(tokens::Command),
        Providers(providers::Command),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::Me(action) => action.run().await,
                Action::Login(action) => action.run().await,
                Action::Logout(action) => action.run().await,
                Action::Tokens(action) => action.run().await,
                Action::Providers(action) => action.run().await,
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

    pub mod tokens {
        use super::*;

        /// Manage personal access tokens
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

        /// List personal access tokens
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

    mod providers {
        use crate::{page, utils::WebArg};

        use super::*;

        /// Manage authentication providers
        #[derive(Parser)]
        #[clap(alias = "provider")]
        pub struct Command {
            #[clap(subcommand)]
            action: Action,
        }

        #[derive(Parser)]
        enum Action {
            List(List),
            Connect(Connect),
            Disconnect(Disconnect),
            Token(Token),
        }

        #[async_trait]
        impl Run for Command {
            async fn run(&self) -> Result {
                match &self.action {
                    Action::List(action) => action.run().await,
                    Action::Connect(action) => action.run().await,
                    Action::Disconnect(action) => action.run().await,
                    Action::Token(action) => action.run().await,
                }
            }
        }

        /// List external accounts connected to your Stencila account
        #[derive(Parser)]
        struct List {
            #[clap(flatten)]
            web: WebArg,
        }

        #[async_trait]
        impl Run for List {
            async fn run(&self) -> Result {
                if self.web.yes {
                    self.web.open(page!("settings/integrations"))
                } else {
                    let providers = provider_list().await?;
                    result::table(providers, Provider::title())
                }
            }
        }

        #[derive(Parser)]
        pub(crate) struct ProviderArg {
            /// The name of the authentication provider
            #[clap(possible_values = ["github", "gitlab","google"], ignore_case = true)]
            pub provider: String,
        }

        /// Connect an external account to your Stencila account
        #[derive(Parser)]
        #[clap(alias = "add")]
        struct Connect {
            #[clap(flatten)]
            provider: ProviderArg,

            #[clap(flatten)]
            web: WebArg,
        }

        #[async_trait]
        impl Run for Connect {
            async fn run(&self) -> Result {
                if self.web.yes {
                    self.web.open(page!("settings/integrations"))
                } else {
                    provider_create(&self.provider.provider).await?;
                    result::nothing()
                }
            }
        }

        /// Disconnect an external account from your Stencila account
        #[derive(Parser)]
        #[clap(alias = "remove")]
        struct Disconnect {
            #[clap(flatten)]
            provider: ProviderArg,

            #[clap(flatten)]
            web: WebArg,
        }

        #[async_trait]
        impl Run for Disconnect {
            async fn run(&self) -> Result {
                if self.web.yes {
                    self.web.open(page!("settings/integrations"))
                } else {
                    provider_delete(&self.provider.provider).await?;
                    result::nothing()
                }
            }
        }

        /// Obtain an access token for a provider
        ///
        /// This command is generally only used during testing or debugging.
        #[derive(Parser)]
        #[clap(hide = true)]
        struct Token {
            #[clap(flatten)]
            provider: ProviderArg,
        }

        #[async_trait]
        impl Run for Token {
            async fn run(&self) -> Result {
                let token = provider_token(&self.provider.provider).await?;
                result::value(token)
            }
        }
    }
}
