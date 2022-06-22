use common::{eyre::Result, serde_json::json, tracing};
use http_utils::CLIENT;

use crate::{
    api,
    errors::Error,
    types::{Team, TeamMember},
    utils::token_read,
};

pub async fn team_list(org_id: &str) -> Result<Vec<Team>> {
    let request = CLIENT
        .get(api!("orgs/{}/teams", org_id))
        .bearer_auth(token_read()?);
    let response = request.send().await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
    }
}

pub async fn team_create(org_id: &str, name: &str, description: Option<&str>) -> Result<Team> {
    let request = CLIENT
        .post(api!("orgs/{}/teams", org_id))
        .bearer_auth(token_read()?)
        .json(&json!({
            "name": name,
            "description": description
        }));

    let response = request.send().await?;
    let team: Team = if response.status().is_success() {
        response.json().await?
    } else {
        return Error::from_response(response).await;
    };
    tracing::info!("Successfully created team #{}", team.id);
    Ok(team)
}

pub async fn team_delete(org_id: &str, team_id: u64) -> Result<()> {
    let response = CLIENT
        .delete(api!("orgs/{}/teams/{}", org_id, team_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if !response.status().is_success() {
        return Error::from_response(response).await;
    }
    tracing::info!("Successfully deleted team #{}", team_id);
    Ok(())
}

pub async fn members_list(org_id: &str, team_id: u64) -> Result<Vec<TeamMember>> {
    let response = CLIENT
        .get(api!("orgs/{}/teams/{}/members", org_id, team_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
    }
}

pub async fn members_create(
    org_id: &str,
    team_id: u64,
    user_id: Option<&str>,
    user_name: Option<&str>,
) -> Result<TeamMember> {
    let response = CLIENT
        .post(api!("orgs/{}/teams/{}/members", org_id, team_id))
        .bearer_auth(token_read()?)
        .json(&json!({
            "userId": user_id,
            "userName": user_name,
        }))
        .send()
        .await?;
    if response.status().is_success() {
        let member: TeamMember = response.json().await?;
        tracing::info!(
            "Successfully added user #{} to team #{}",
            member.user.id,
            team_id
        );
        Ok(member)
    } else {
        Error::from_response(response).await
    }
}

pub async fn members_delete(org_id: &str, team_id: u64, membership_id: u64) -> Result<()> {
    let response = CLIENT
        .delete(api!(
            "orgs/{}/teams/{}/members/{}",
            org_id,
            team_id,
            membership_id
        ))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        tracing::info!("Successfully removed user from team #{}", team_id);
        Ok(())
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

    use crate::orgs::cli::OrgArg;

    use super::*;

    /// Manage teams
    ///
    /// Use this command to list your Stencila teams, inspect and update details for individual
    /// teams, and to manage team sources, members, and deployments etc.
    #[derive(Parser)]
    #[clap(alias = "team")]
    pub struct Command {
        #[clap(subcommand)]
        action: Action,
    }

    #[derive(Parser)]
    enum Action {
        List(List),
        Create(Create),
        Delete(Delete),
        Members(members::Command),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::List(action) => action.run().await,
                Action::Create(action) => action.run().await,
                Action::Delete(action) => action.run().await,
                Action::Members(action) => action.run().await,
            }
        }
    }

    /// List teams
    ///
    /// Use this command to get a list of Stencila teams for an organization that you are a member of.
    ///
    /// Defaults to using you default organization. Use the `--org` option to list teams in
    /// another organization.
    #[derive(Default, Parser)]
    struct List {
        #[clap(flatten)]
        org: OrgArg,
    }

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let teams = team_list(&self.org.resolve()?).await?;
            result::table(teams, Team::title())
        }
    }

    /// Create a team
    ///
    /// Use this command to create a new Stencila team for an organization that you are an admin or
    /// owner of.
    ///
    /// Defaults to using you default organization. Use the `--org` option to create a team in
    /// another organization.
    #[derive(Parser)]
    #[clap(alias = "new")]
    struct Create {
        /// The name of the team
        ///
        /// Must be unique within the organization.
        name: String,

        /// A description for the team
        description: Option<String>,

        #[clap(flatten)]
        org: OrgArg,
    }

    #[async_trait]
    impl Run for Create {
        async fn run(&self) -> Result {
            let team = team_create(
                &self.org.resolve()?,
                &self.name,
                self.description.as_deref(),
            )
            .await?;
            result::invisible(team)
        }
    }

    /// Delete a team
    ///
    /// Use this command to delete a Stencila team, forever.
    ///
    /// Defaults to using you default organization. Use the `--org` option to delete a
    /// team in another organization.
    #[derive(Parser)]
    struct Delete {
        // The id of the team to delete
        team: u64,

        #[clap(flatten)]
        org: OrgArg,
    }

    #[async_trait]
    impl Run for Delete {
        async fn run(&self) -> Result {
            tracing::warn!("Deleting a team can not be undone. Are you sure? (y/n)");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                tracing::info!("Aborting team deletion.");
            } else {
                team_delete(&self.org.resolve()?, self.team).await?;
            }

            result::nothing()
        }
    }

    mod members {
        use crate::utils::UUID_REGEX;

        use super::*;

        /// Manage team members
        #[derive(Parser)]
        pub struct Command {
            #[clap(subcommand)]
            action: Action,
        }

        #[derive(Parser)]
        enum Action {
            List(List),
            Add(Add),
            Remove(Remove),
        }

        #[async_trait]
        impl Run for Command {
            async fn run(&self) -> Result {
                match &self.action {
                    Action::List(action) => action.run().await,
                    Action::Add(action) => action.run().await,
                    Action::Remove(action) => action.run().await,
                }
            }
        }

        /// List members of a team
        ///
        /// Use this command to retrieve a list of a teams members. Each item in this list has a
        /// membership id which you can use to remove a team member.
        ///
        /// Defaults to using your default organization. Use the `--org` option to list members in
        /// a team belonging to another organization.
        #[derive(Default, Parser)]
        struct List {
            // The id of the team
            team: u64,

            #[clap(flatten)]
            org: OrgArg,
        }

        #[async_trait]
        impl Run for List {
            async fn run(&self) -> Result {
                let members = members_list(&self.org.resolve()?, self.team).await?;
                result::table(members, TeamMember::title())
            }
        }

        /// Add a user to a team
        ///
        /// Use this command to add a user to a team.
        #[derive(Parser)]
        struct Add {
            /// The id of the team
            team: u64,

            /// The username or id of the user
            ///
            /// Use the user's username (e.g. "mike21") or their id (e.g. "b18beb15-af3a-4696-98ea-f89e0cf6149a").
            user: String,

            #[clap(flatten)]
            org: OrgArg,
        }

        #[async_trait]
        impl Run for Add {
            async fn run(&self) -> Result {
                let (user_id, user_name) = if UUID_REGEX.is_match(&self.user) {
                    (Some(self.user.as_str()), None)
                } else {
                    (None, Some(self.user.as_str()))
                };

                let member =
                    members_create(&self.org.resolve()?, self.team, user_id, user_name).await?;
                result::invisible(member)
            }
        }

        /// Remove a user from a team
        #[derive(Parser)]
        struct Remove {
            /// The id of the team
            team: u64,

            /// The id of the team membership
            ///
            /// Note: this is the id of the membership, as shown in the membership list of the team,
            /// not the id of the user.
            membership: u64,

            #[clap(flatten)]
            org: OrgArg,
        }

        #[async_trait]
        impl Run for Remove {
            async fn run(&self) -> Result {
                members_delete(&self.org.resolve()?, self.team, self.membership).await?;
                result::nothing()
            }
        }
    }
}
