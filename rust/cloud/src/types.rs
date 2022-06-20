//! Type definitions for Stencila Cloud API types
//!
//! To avoid drift, prefer to only only add properties that are needed here to these structs.

use cli_utils::table::{date_time_ago, option_date_time_ago, option_string, Table};
use common::{
    chrono::{DateTime, Utc},
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct ApiToken {
    #[table(title = "ID")]
    pub id: u64,

    #[table(title = "Token")]
    pub token: String,

    #[table(title = "Tag", display_fn = "option_string")]
    pub tag: Option<String>,

    #[table(title = "Note", display_fn = "option_string")]
    pub note: Option<String>,

    #[table(title = "Expires", display_fn = "option_date_time_ago")]
    pub expires_at: Option<DateTime<Utc>>,

    #[table(title = "Last used", display_fn = "option_date_time_ago")]
    pub last_used_at: Option<DateTime<Utc>>,

    #[table(title = "Created", display_fn = "date_time_ago")]
    pub created_at: DateTime<Utc>,
}

/// The authenticated user
///
/// This is not used for `users find` or `users invite`, only for `user.json`.
/// It has the properties that we need to show to the current user.
#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct User {
    pub id: String,

    pub short_name: String,

    pub long_name: Option<String>,

    pub email: Option<String>,

    pub default_org: Option<u64>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct Org {
    #[table(title = "ID")]
    pub id: u64,

    #[table(title = "Short name")]
    pub short_name: String,

    #[table(title = "Long name", display_fn = "option_string")]
    pub long_name: Option<String>,

    #[table(title = "Role", display_fn = "option_org_role_table_display")]
    pub user_role: Option<String>,
}

fn option_org_role_table_display(role: &Option<String>) -> &str {
    role.as_deref()
        .map(project_role_table_display)
        .unwrap_or("*None*")
}

fn org_role_table_display(role: &str) -> &str {
    match role {
        "owner" => "🧰 Owner",
        "admin" => "🛠  Admin",
        "member" => "🔨 Member",
        _ => "?",
    }
}

/// A user's personal organization
///
/// Used when finding and inviting users. Differs from [`Org`] in that it displays
/// `user_id` (a string) rather than `id` (an integer)
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct OrgPersonal {
    #[table(skip)]
    pub id: u64,

    #[table(title = "ID")]
    pub user_id: String,

    #[table(title = "Short name")]
    pub short_name: String,

    #[table(title = "Long name", display_fn = "option_string")]
    pub long_name: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct OrgMember {
    #[table(title = "ID")]
    pub id: u64,

    #[table(title = "User", display_fn = "org_member_table_display")]
    pub user: User,

    #[table(title = "Role", display_fn = "org_role_table_display")]
    pub role: String,
}

fn org_member_table_display(user: &User) -> String {
    format!("{} (#{})", user.short_name.as_str(), user.id)
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct Team {
    #[table(title = "ID")]
    pub id: u64,

    #[table(title = "Name", display_fn = "option_string")]
    pub name: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ProjectLocal {
    pub id: Option<u64>,

    pub name: Option<String>,

    pub title: Option<String>,

    /// Whether the project is public or not
    ///
    /// Uses `public` rather than `isPublic` as used on Stencila Cloud.
    #[serde(alias = "isPublic")]
    pub public: Option<bool>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct ProjectRemote {
    #[table(title = "ID")]
    pub id: u64,

    #[table(title = "Organization", display_fn = "project_org_table_display")]
    pub org: Org,

    #[table(title = "Name")]
    pub name: String,

    #[table(title = "Title", display_fn = "project_title_table_display")]
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

fn project_title_table_display(title: &Option<String>) -> &str {
    title.as_deref().unwrap_or("*Untitled*")
}

fn project_visibility_table_display(is_public: &bool) -> &str {
    match is_public {
        true => "🌍",
        false => "🔒",
    }
}

fn option_project_role_table_display(role: &Option<String>) -> &str {
    role.as_deref()
        .map(project_role_table_display)
        .unwrap_or("*None*")
}

fn project_role_table_display(role: &str) -> &str {
    match role {
        "owner" => "🧰 Owner",
        "admin" => "🛠  Admin",
        "member" => "🔨 Member",
        _ => "?",
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Table)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[table(crate = "cli_utils::cli_table")]
pub struct ProjectMember {
    #[table(title = "ID")]
    pub id: u64,

    #[table(title = "User", display_fn = "project_member_user_table_display")]
    pub user: Option<User>,

    #[table(title = "Team", display_fn = "project_member_team_table_display")]
    pub team: Option<Team>,

    #[table(title = "Role", display_fn = "project_role_table_display")]
    pub role: String,
}

fn project_member_user_table_display(user: &Option<User>) -> String {
    user.as_ref()
        .map(|user| format!("{} (#{})", user.short_name.as_str(), user.id))
        .unwrap_or_else(|| "-".to_string())
}

fn project_member_team_table_display(team: &Option<Team>) -> String {
    team.as_ref()
        .map(|team| {
            let name = team.name.clone().unwrap_or_else(|| "*Unnamed*".to_string());
            format!("{} (#{})", name, team.id)
        })
        .unwrap_or_else(|| "-".to_string())
}
