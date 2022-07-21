use common::{
    eyre::Result,
    serde::{Deserialize, Serialize},
    serde_json::json,
};
use http_utils::CLIENT;

use crate::{api, errors::*, types::*, utils::token_read};

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
        Error::from_response(response).await
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
        Error::from_response(response).await
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
        Find(Find),
        Invite(Invite),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::Find(action) => action.run().await,
                Action::Invite(action) => action.run().await,
            }
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
