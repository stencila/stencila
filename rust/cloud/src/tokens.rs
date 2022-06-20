use common::{
    chrono::{self, Utc},
    eyre::{bail, Result},
    serde_json::json,
    tracing,
};
use http_utils::CLIENT;

use crate::{api, errors::*, types::*, utils::token_read};

pub async fn token_list() -> Result<Vec<ApiToken>> {
    let response = CLIENT
        .get(api!("tokens"))
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
        .post(api!("tokens"))
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
        .delete(api!("tokens/{}", id))
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
                token_create(self.tag.as_deref(), self.note.as_deref(), self.expires_in).await?;
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
