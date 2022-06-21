use std::{
    env,
    fs::{create_dir_all, read_to_string, remove_file, write},
    path::{Path, PathBuf},
};

use common::{
    eyre::{self, bail, Result},
    serde_json::{self, json},
    serde_yaml, toml, tracing,
};
use http_utils::CLIENT;

use crate::{
    api,
    errors::Error,
    types::{ProjectLocal, ProjectMember, ProjectRemote},
    utils::token_read,
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

impl ProjectLocal {
    /// Convert a local project to a JSON payload suitable for a `PATCH /projects/<id>` request
    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(json!({
            "name": self.name,
            "title": self.title,
            "isPublic": self.public.unwrap_or(false)
        }))
    }
}

pub async fn project_list(
    search: Option<&str>,
    role: Option<&str>,
    org_id: Option<u64>,
    org_name: Option<&str>,
    all: bool,
) -> Result<Vec<ProjectRemote>> {
    let mut request = CLIENT.get(api!("projects")).bearer_auth(token_read()?);
    if let Some(search) = search {
        request = request.query(&[("search", search)]);
    }
    if let Some(role) = role {
        request = request.query(&[("role", role)]);
    }
    if let Some(org_id) = org_id {
        request = request.query(&[("orgId", org_id)]);
    }
    if let Some(org_name) = org_name {
        request = request.query(&[("orgName", org_name)]);
    }
    if all {
        request = request.query(&[("all", all)]);
    }

    let response = request.send().await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
    }
}

pub async fn project_create(
    org_id: Option<u64>,
    name: Option<&str>,
    title: Option<&str>,
    public: bool,
) -> Result<ProjectLocal> {
    let mut request = CLIENT
        .post(api!("projects"))
        .bearer_auth(token_read()?)
        .json(&json!({
            "name": name,
            "title": title,
            "is_public": public
        }));
    if let Some(org_id) = org_id {
        request = request.query(&[("orgId", org_id)]);
    }

    let response = request.send().await?;
    let project: ProjectLocal = if response.status().is_success() {
        response.json().await?
    } else {
        return Error::from_response(response).await;
    };

    if project_current().is_err() {
        let toml = toml::to_string_pretty(&project)?;
        write("stencila.toml", toml)?;
    }

    tracing::info!(
        "Successfully created project #{}",
        project.id.unwrap_or_default()
    );

    Ok(project)
}

pub async fn project_clone(project_id: u64, dir: Option<&Path>) -> Result<()> {
    let response = CLIENT
        .get(api!("projects/{}", project_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        let value: serde_json::Value = response.json().await?;

        let project: ProjectRemote = serde_json::from_value(value.clone())?;

        let dir = dir
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(project.name));
        if dir.exists() {
            bail!("Directory `{}` already exists", dir.display())
        }
        create_dir_all(&dir)?;

        let project: ProjectLocal = serde_json::from_value(value)?;
        let toml = toml::to_string_pretty(&project)?;
        write(dir.join("stencila.toml"), toml)?;

        tracing::info!(
            "Successfully cloned project #{} into `{}`",
            project_id,
            dir.display()
        );

        Ok(())
    } else {
        Error::from_response(response).await
    }
}

pub async fn project_retrieve(project_id: &str) -> Result<ProjectRemote> {
    let response = CLIENT
        .get(api!("projects/{}", project_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
    }
}

pub async fn project_pull() -> Result<()> {
    let (path, project) = project_current()?;

    let id = match project.id {
        Some(id) => id,
        None => bail!(
            "Project file is missing an `id`. To see available projects and their ids use `stencila projects list`",
        ),
    };

    let response = CLIENT
        .get(api!("projects/{}", id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        let project: ProjectLocal = response.json().await?;
        let toml = toml::to_string_pretty(&project)?;
        write(&path, toml)?;

        tracing::info!(
            "Successfully pulled project #{} to file `{}`",
            id,
            path.display()
        );

        Ok(())
    } else {
        Error::from_response(response).await
    }
}

pub async fn project_push() -> Result<()> {
    let (.., project) = project_current()?;

    let id = match project.id {
        Some(id) => id,
        None => bail!(
            "Project file is missing an `id`. To create a new project from the file use `stencila projects create --from <file>`",
        ),
    };

    let data = project.to_json()?;

    let response = CLIENT
        .patch(api!("projects/{}", id))
        .bearer_auth(token_read()?)
        .json(&data)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Error::from_response(response).await
    }
}

pub async fn project_delete(project_id: &str) -> Result<()> {
    let response = CLIENT
        .delete(api!("projects/{}", project_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if !response.status().is_success() {
        return Error::from_response(response).await;
    }

    if let Ok((path, project)) = project_current() {
        if let Some(id) = project.id {
            if id.to_string() == project_id {
                remove_file(path)?;
            }
        }
    }

    tracing::info!("Successfully deleted project #{}", project_id);

    Ok(())
}

pub async fn members_list(project_id: &str) -> Result<Vec<ProjectMember>> {
    let response = CLIENT
        .get(api!("projects/{}/members", project_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
        let mut members: Vec<ProjectMember> = response.json().await?;
        members
            .iter_mut()
            .for_each(|member| member.desc = member.generate_desc());
        Ok(members)
    } else {
        Error::from_response(response).await
    }
}

pub async fn members_create(
    project_id: &str,
    user_name: Option<&str>,
    user_id: Option<&str>,
    team_id: Option<&str>,
    role: &str,
) -> Result<ProjectMember> {
    let response = CLIENT
        .post(api!("projects/{}/members", project_id))
        .bearer_auth(token_read()?)
        .json(&json!({
            "userName": user_name,
            "userId": user_id,
            "teamId": team_id,
            "role": role
        }))
        .send()
        .await?;
    if response.status().is_success() {
        let member: ProjectMember = response.json().await?;
        if let Some(user) = &member.user {
            tracing::info!(
                "Successfully added user #{} as {} of project #{}",
                user.id,
                member.role,
                project_id
            )
        } else if let Some(team) = &member.team {
            tracing::info!(
                "Successfully added team #{} as {} of project #{}",
                team.id,
                member.role,
                project_id
            )
        }
        Ok(member)
    } else {
        Error::from_response(response).await
    }
}

pub async fn members_update(
    project_id: &str,
    membership_id: &str,
    role: &str,
) -> Result<ProjectMember> {
    let json = serde_json::to_string(&serde_json::json!({ "role": role }))?;
    let response = CLIENT
        .patch(api!("projects/{}/members/{}", project_id, membership_id))
        .bearer_auth(token_read()?)
        .body(json)
        .send()
        .await?;
    if response.status().is_success() {
        let member: ProjectMember = response.json().await?;
        if let Some(user) = &member.user {
            tracing::info!(
                "Successfully changed role of user #{} to {} on project #{}",
                user.id,
                member.role,
                project_id
            )
        }
        if let Some(team) = &member.team {
            tracing::info!(
                "Successfully changed role of team #{} to {} on project #{}",
                team.id,
                member.role,
                project_id
            )
        }
        Ok(member)
    } else {
        Error::from_response(response).await
    }
}

pub async fn members_delete(project_id: &str, membership_id: &str) -> Result<()> {
    let response = CLIENT
        .delete(api!("projects/{}/members/{}", project_id, membership_id))
        .bearer_auth(token_read()?)
        .send()
        .await?;
    if response.status().is_success() {
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
        Create(Create),
        Clone(Clone),
        Pull(Pull),
        Push(Push),
        Delete(Delete),
        Members(members::Command),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Create(action) => action.run().await,
                Action::Clone(action) => action.run().await,
                Action::Pull(action) => action.run().await,
                Action::Push(action) => action.run().await,
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
        /// Use a numeric id or organization's short name.
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
            let (org_id, org_name) = match &self.org {
                Some(org) => match org.parse::<u64>() {
                    Ok(id) => (Some(id), None),
                    Err(..) => (None, Some(org.as_str())),
                },
                None => (None, None),
            };
            let projects = project_list(
                self.search.as_deref(),
                self.role.as_deref(),
                org_id,
                org_name,
                self.all,
            )
            .await?;
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

    /// Create a project
    ///
    /// Use this command to create a new Stencila project. A new project will be created on Stencila Cloud
    /// and a `stencila.toml` file will be created, with the new project's id, in the current folder.
    ///
    /// Use the `--org` option to select the organization for the project.
    #[derive(Parser)]
    #[clap(alias = "init")]
    struct Create {
        /// The name of the project
        ///
        /// Must be unique within the organization. Defaults to a randomly generated name.
        #[clap(short, long)]
        name: Option<String>,

        /// The title of the project
        #[clap(short, long)]
        title: Option<String>,

        /// Whether the project should be public
        ///
        /// New projects default to being private. Use the this flag to make
        /// the new project public.
        #[clap(short, long)]
        public: bool,

        /// The organization under which to create the project
        ///
        /// Use the organization's numeric id.
        #[clap(short, long)]
        org: Option<u64>,
    }

    #[async_trait]
    impl Run for Create {
        async fn run(&self) -> Result {
            let project = project_create(
                self.org,
                self.name.as_deref(),
                self.title.as_deref(),
                self.public,
            )
            .await?;
            result::invisible(project)
        }
    }

    /// Clone a project
    ///
    /// Use this command to create a local clone of a project on Stencila Cloud.
    #[derive(Parser)]
    struct Clone {
        /// The id of the project to clone
        project: u64,

        /// The directory to clone the project into
        ///
        /// Defaults to the name of the project
        into: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Clone {
        async fn run(&self) -> Result {
            project_clone(self.project, self.into.as_deref()).await?;
            result::nothing()
        }
    }

    /// Pull the current project
    ///
    /// Updates the local project configuration file (e.g. `stencila.toml`) from Stencila Cloud.
    /// The file must have a project id.
    #[derive(Parser)]
    struct Pull;

    #[async_trait]
    impl Run for Pull {
        async fn run(&self) -> Result {
            project_pull().await?;
            result::nothing()
        }
    }

    /// Push the current project
    ///
    /// Updates the project on Stencila Cloud based on the local configuration file (e.g. `stencila.toml`).
    /// The file must have a project id. You can create a new project from a file with no id using `stencila projects create --from <file>`.
    #[derive(Parser)]
    struct Push;

    #[async_trait]
    impl Run for Push {
        async fn run(&self) -> Result {
            project_push().await?;
            result::nothing()
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

            tracing::warn!("Deleting a project can not be undone. Please confirm you want to proceed by typing the name of the project: {}", project.name);

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim() != project.name {
                tracing::error!("Inputted name is not the same as the project name. Cancelling project deletion.")
            }

            project_delete(project_id).await?;
            result::nothing()
        }
    }

    mod members {
        use crate::utils::UUID_REGEX;

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
            /// The type of member to add
            #[clap(possible_values = ["user", "team"])]
            type_: String,

            /// The id of the user or team
            ///
            /// To add a user use their username or id (e.g. "b18beb15-af3a-4696-98ea-f89e0cf6149a").
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
                let id = Some(self.id.as_str());
                let (user_name, user_id, team_id) = match self.type_.as_str() {
                    "user" => match UUID_REGEX.is_match(&self.id) {
                        true => (None, id, None),
                        false => (id, None, None),
                    },
                    "team" => (None, None, id),
                    _ => unreachable!(),
                };

                let member = members_create(
                    &self.project.resolve()?,
                    user_name,
                    user_id,
                    team_id,
                    &self.role,
                )
                .await?;
                result::invisible(member)
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
                let member = members_update(&self.project.resolve()?, &self.id, &self.role).await?;
                result::invisible(member)
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
                members_delete(&self.project.resolve()?, &self.id).await?;
                result::nothing()
            }
        }
    }
}
