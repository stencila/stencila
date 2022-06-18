use std::{env, fs::read_to_string, path::PathBuf};

use common::{
    eyre::{self, bail, Result},
    serde_json, serde_yaml, toml, tracing,
};
use http_utils::CLIENT;

use crate::{
    errors::Error,
    types::{ProjectLocal, ProjectMember, ProjectRemote},
    utils::{token_read, BASE_URL},
};

/// Resolve the current project
pub fn project_current() -> Result<(PathBuf, ProjectLocal)> {
    let mut dir = env::current_dir()?;
    loop {
        for filename in ["stencila.toml", "stencila.yaml", "stencila.json"] {
            let path = dir.join(filename);
            if path.exists() {
                let content = read_to_string(&path)?;
                let path_str = path.to_string_lossy();
                let project = if path_str.ends_with(".toml") {
                    toml::from_str(&content)?
                } else if path_str.ends_with(".yaml") {
                    serde_yaml::from_str(&content)?
                } else if path_str.ends_with(".json") {
                    serde_json::from_str(&content)?
                } else {
                    unreachable!()
                };
                return Ok((path, project));
            }
        }
        if let Some(parent) = dir.parent() {
            dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    bail!("Unable to determine the current project. Could not find a `stencila.toml`, `stencila.yaml` or `stencila.json` file in the current directory or any of its parents")
}

pub async fn project_list(
    search: &Option<String>,
    role: &Option<String>,
    org: &Option<String>,
    all: bool,
) -> Result<Vec<ProjectRemote>> {
    let mut request = CLIENT
        .get(format!("{}/projects", BASE_URL))
        .bearer_auth(token_read()?);
    if let Some(search) = search {
        request = request.query(&[("search", search)]);
    }
    if let Some(role) = role {
        request = request.query(&[("role", role)]);
    }
    if let Some(org) = org {
        request = request.query(&[("org", org)]);
    }
    request = request.query(&[("all", all)]);

    let response = request.send().await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub async fn project_retrieve(project_id: &str) -> Result<ProjectRemote> {
    let response = CLIENT
        .get(format!("{}/projects/{}", BASE_URL, project_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub async fn project_delete(project_id: &str) -> Result<()> {
    let response = CLIENT
        .delete(format!("{}/projects/{}", BASE_URL, project_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if !response.status().is_success() {
        bail!("{}", Error::response_to_string(response).await)
    }

    if let Ok((path, project)) = project_current() {
        if let Some(id) = project.id {
            if id.to_string() == project_id {
                std::fs::remove_file(path)?;
            }
        }
    }

    Ok(())
}

pub async fn members_list(project_id: &str) -> Result<Vec<ProjectMember>> {
    let response = CLIENT
        .get(format!("{}/projects/{}/members", BASE_URL, project_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub async fn members_add(
    project_id: &str,
    user_id: Option<&str>,
    team_id: Option<&str>,
    role: &str,
) -> Result<ProjectMember> {
    let json = serde_json::to_string(&serde_json::json!({
        "userId": user_id,
        "teamId": team_id,
        "role": role
    }))?;
    let response = CLIENT
        .post(format!("{}/projects/{}/members", BASE_URL, project_id))
        .bearer_auth(token_read()?)
        .body(json)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub async fn members_change(project_id: &str, membership_id: &str, role: &str) -> Result<()> {
    let json = serde_json::to_string(&serde_json::json!({ "role": role }))?;
    let response = CLIENT
        .patch(format!(
            "{}/projects/{}/members/{}",
            BASE_URL, project_id, membership_id
        ))
        .bearer_auth(token_read()?)
        .body(json)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(())
    } else {
        bail!("{}", Error::response_to_string(response).await)
    }
}

pub async fn members_remove(project_id: &str, membership_id: &str) -> Result<()> {
    let response = CLIENT
        .delete(format!(
            "{}/projects/{}/members/{}",
            BASE_URL, project_id, membership_id
        ))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
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

    // An option that is reused in several subcommands below to
    // allow the user to specify the target project for the command.
    #[derive(Default, Parser)]
    pub struct ProjectArg {
        /// The id of the project
        ///
        /// If this option is not supplied, Stencila will use the current project.
        /// The current project is determined by searching upwards, from the current directory,
        /// for a `stencila.toml`, `stencila.yaml`, or `stencila.json` file.
        #[clap(long, short)]
        project: Option<String>,
    }

    impl ProjectArg {
        pub fn resolve(&self) -> eyre::Result<String> {
            match &self.project {
                Some(id) => Ok(id.to_string()),
                None => match project_current()?.1.id {
                    Some(id) => Ok(id.to_string()),
                    None => bail!("Stencila project file does not have an id"),
                },
            }
        }
    }

    /// Manage projects
    ///
    /// Use this command to list your Stencila projects, inspect and update details for individual
    /// projects, and to manage project sources, members, and deployments etc.
    #[derive(Parser)]
    #[clap(alias = "project")]
    pub struct Command {
        #[clap(subcommand)]
        action: Action,
    }

    #[derive(Parser)]
    enum Action {
        List(List),
        Show(Show),
        Delete(Delete),
        Members(members::Command),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Delete(action) => action.run().await,
                Action::Members(action) => action.run().await,
            }
        }
    }

    /// List projects
    ///
    /// Use this command to get a list of Stencila projects that you are a member of, or that are public.
    /// For more details on a particular project use the `show` sibling command.
    ///
    /// Use the optional search string to filter projects using the name and title
    /// properties of projects.
    ///
    /// By default, only shows projects that you are a member of, use the `--all` flag to include
    /// projects that you are not a member of but which are public. Use the `--role` flag to only
    /// include projects for which you have a particular role.
    #[derive(Default, Parser)]
    struct List {
        /// A search string to filter projects by
        #[clap(short, long)]
        search: Option<String>,

        /// Only list projects for which you have a specific role
        ///
        /// The role may be granted directly to you, or via a team.
        #[clap(short, long, possible_values = ["owner", "admin", "member"])]
        role: Option<String>,

        /// Only list projects which belong to a particular organization
        ///
        /// Provide a numeric organization id.
        #[clap(short, long)]
        org: Option<String>,

        /// List all projects, including public project that you are not a member of
        ///
        /// To avoid getting a long list of projects, you generally only want to use
        /// this flag in conjunction with a search string.
        #[clap(short, long)]
        all: bool,
    }

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let projects = project_list(&self.search, &self.role, &self.org, self.all).await?;
            result::table(projects, ProjectRemote::title())
        }
    }

    /// Show details of a project
    ///
    /// Use this command to get details on a Stencila project such as when it was last updated.
    /// By default, this command shows details for the current project. Use the `--project` option
    /// to target another project.
    #[derive(Parser)]
    struct Show {
        #[clap(flatten)]
        project: ProjectArg,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let project = project_retrieve(&self.project.resolve()?).await?;
            result::value(project)
        }
    }

    /// Delete a project
    ///
    /// Use this command to delete a Stencila project, forever. If the current project is being deleted
    /// then its local `stencila.{toml,yaml,json}` file will also be deleted. No other directories
    /// or files will be deleted.
    ///
    /// Only project owners can delete a project. Because a project can not be un-deleted,
    /// this command asks you to confirm by typing the name of the project.
    #[derive(Parser)]
    struct Delete {
        #[clap(flatten)]
        project: ProjectArg,
    }

    #[async_trait]
    impl Run for Delete {
        async fn run(&self) -> Result {
            let project_id = &self.project.resolve()?;
            let project = project_retrieve(project_id).await?;

            let name = project.name.unwrap_or_else(|| "unnamed".to_string());
            tracing::warn!("Deleting a project can not be undone. Please confirm you want to proceed by typing the name of the project: {}", name);

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() != name {
                tracing::error!("Inputted name is not the same as the project name. Cancelling project deletion.")
            }

            project_delete(project_id).await?;
            result::nothing()
        }
    }

    mod members {
        use super::*;

        /// Manage project members
        #[derive(Parser)]
        pub struct Command {
            #[clap(subcommand)]
            action: Action,
        }

        #[derive(Parser)]
        enum Action {
            List(List),
            Add(Add),
            Change(Change),
            Remove(Remove),
        }

        #[async_trait]
        impl Run for Command {
            async fn run(&self) -> Result {
                match &self.action {
                    Action::List(action) => action.run().await,
                    Action::Add(action) => action.run().await,
                    Action::Change(action) => action.run().await,
                    Action::Remove(action) => action.run().await,
                }
            }
        }

        /// List members of a project
        ///
        /// Use this command to retrieve a list of a projects members and their role
        /// on the project. Each item in this list has a membership id which you can
        /// use to remove, or modify, a membership.
        #[derive(Default, Parser)]
        struct List {
            #[clap(flatten)]
            project: cli::ProjectArg,
        }

        #[async_trait]
        impl Run for List {
            async fn run(&self) -> Result {
                let members = members_list(&self.project.resolve()?).await?;
                result::table(members, ProjectMember::title())
            }
        }

        /// Add a user or team as a member of a project
        ///
        /// Use this command to a add a user or team to a project. When you add a team to a
        /// project, all the users that are members of that team get the same role on the project.
        /// The default role is "member". Specify "owner" or "admin" roles for greater permissions
        /// on the project.
        #[derive(Parser)]
        struct Add {
            /// The id of the user or team
            ///
            /// To add a user use their UUID (e.g. "b18beb15-af3a-4696-98ea-f89e0cf6149a").
            /// To add a team use its numeric id (e.g. 123).
            id: String,

            /// The role to give the user or team
            ///
            /// Defaults to "member". Use "admin" or "owner" to give the user or team greater
            /// permissions on the project.
            #[clap(default_value = "member", possible_values = ["owner", "admin", "member"])]
            role: String,

            #[clap(flatten)]
            project: cli::ProjectArg,
        }

        #[async_trait]
        impl Run for Add {
            async fn run(&self) -> Result {
                let project_id = self.project.resolve()?;

                // Resolve if id is for a user or team
                let (user_id, team_id) = match self.id.parse::<u64>() {
                    Ok(..) => (None, Some(self.id.as_str())),
                    Err(..) => (Some(self.id.as_str()), None),
                };

                let member = members_add(&project_id, user_id, team_id, &self.role).await?;

                if let Some(user_id) = user_id {
                    tracing::info!(
                        "Successfully added user #{} as {} of project #{}",
                        user_id,
                        member.role,
                        project_id
                    )
                } else if let Some(team_id) = team_id {
                    tracing::info!(
                        "Successfully added team #{} as {} of project #{}",
                        team_id,
                        member.role,
                        project_id
                    )
                }
                result::value(member)
            }
        }

        /// Change the role of user or team on a project
        #[derive(Parser)]
        struct Change {
            /// The id of the membership
            ///
            /// Note: this is the id of the membership, as shown in the membership list of the project,
            /// not the id of the user or team.
            id: String,

            /// The role to give the user or team
            ///
            /// Defaults to "member". Use "admin" or "owner" to give the user or team greater
            /// permissions on the project.
            #[clap(default_value = "member", possible_values = ["owner", "admin", "member"])]
            role: String,

            #[clap(flatten)]
            project: cli::ProjectArg,
        }

        #[async_trait]
        impl Run for Change {
            async fn run(&self) -> Result {
                members_change(&self.project.resolve()?, &self.id, &self.role).await?;
                result::nothing()
            }
        }

        /// Remove a user or team membership of a project
        #[derive(Parser)]
        struct Remove {
            /// The id of the membership
            ///
            /// Note: this is the id of the membership, as shown in the membership list of the project,
            /// not the id of the user or team.
            id: String,

            #[clap(flatten)]
            project: cli::ProjectArg,
        }

        #[async_trait]
        impl Run for Remove {
            async fn run(&self) -> Result {
                members_remove(&self.project.resolve()?, &self.id).await?;
                result::nothing()
            }
        }
    }
}
