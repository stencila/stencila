//! Type definitions for Stencila Cloud API types
//!
//! To avoid drift, prefer to only only add properties that are needed here to these structs.

use cli_utils::table::{date_time_ago, option_string, option_u64, Table};
use common::{
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
};

pub fn id_table_display(id: &u64) -> String {
    format!("#{}", id)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ApiToken {
    pub token: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct User {
    #[table(title = "User ID")]
    pub id: String,

    #[table(title = "Username")]
    pub short_name: String,

    #[table(title = "Full name", display_fn = "option_string")]
    pub long_name: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct Org {
    #[table(title = "Org.")]
    pub id: u64,

    #[table(title = "Short name")]
    pub short_name: String,

    #[table(title = "Long name", display_fn = "option_string")]
    pub long_name: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct Team {
    #[table(title = "Project", display_fn = "id_table_display")]
    pub id: u64,

    #[table(title = "Org", display_fn = "id_table_display")]
    pub org_id: u64,

    #[table(title = "Name", display_fn = "option_string")]
    pub name: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ProjectLocal {
    pub id: Option<u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct ProjectRemote {
    #[table(title = "Project", display_fn = "id_table_display")]
    pub id: u64,

    #[table(title = "Org.", display_fn = "project_org_table_display")]
    pub org: Org,

    #[table(title = "Name", display_fn = "option_string")]
    pub name: Option<String>,

    #[table(title = "Title", display_fn = "option_string")]
    pub title: Option<String>,

    #[table(title = "Visibility", display_fn = "project_visibility_table_display")]
    pub is_public: bool,

    #[table(title = "Role", display_fn = "option_project_role_table_display")]
    pub user_role: Option<String>,

    #[table(title = "Updated", display_fn = "date_time_ago")]
    pub updated_at: DateTime<Utc>,

    #[table(skip, title = "Created", display_fn = "date_time_ago")]
    pub created_at: DateTime<Utc>,
}

fn project_org_table_display(org: &Org) -> String {
    format!("{} (#{})", org.short_name.as_str(), org.id)
}

pub fn project_visibility_table_display(is_public: &bool) -> &str {
    match is_public {
        true => "🌍",
        false => "🔒",
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct ProjectMember {
    #[table(title = "Membership", display_fn = "id_table_display")]
    pub id: u64,

    #[table(title = "User", display_fn = "member_user_table_display")]
    pub user: Option<User>,

    #[table(title = "Team", display_fn = "member_team_table_display")]
    pub team: Option<Team>,

    #[table(title = "Role", display_fn = "project_role_table_display")]
    pub role: String,
}

fn member_user_table_display(user: &Option<User>) -> String {
    user.as_ref()
        .map(|user| format!("{} (#{})", user.short_name.as_str(), user.id))
        .unwrap_or_default()
}

fn member_team_table_display(team: &Option<Team>) -> String {
    team.as_ref()
        .map(|team| {
            let name = team.name.clone().unwrap_or_else(|| "*unnamed*".to_string());
            format!("{} (#{})", name, team.id)
        })
        .unwrap_or_else(|| "-".to_string())
}

fn option_project_role_table_display(role: &Option<String>) -> &str {
    role.as_deref()
        .map(project_role_table_display)
        .unwrap_or_default()
}

fn project_role_table_display(role: &str) -> &str {
    match role {
        "owner" => "**Owner**",
        "admin" => "**Admin**",
        "member" => "Member",
        _ => role,
    }
}
