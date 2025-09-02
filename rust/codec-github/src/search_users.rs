use std::str::FromStr;

use serde::Deserialize;
use stencila_codec::stencila_schema::{
    ContactPoint, ImageObject, Node, Organization, OrganizationOptions, Person, PersonOptions,
};

use crate::search_code::TextMatch;

/// User search result item from GitHub search API
#[derive(Deserialize)]
pub struct UserSearchItem {
    /// Username/login
    pub login: String,
    /// User ID
    pub id: i64,
    /// Node ID for GraphQL API
    pub node_id: String,
    /// Avatar image URL
    pub avatar_url: String,
    /// Gravatar ID
    pub gravatar_id: Option<String>,
    /// API URL for the user
    pub url: String,
    /// HTML URL for the user's profile
    pub html_url: String,
    /// Followers URL
    pub followers_url: String,
    /// Subscriptions URL
    pub subscriptions_url: String,
    /// Organizations URL
    pub organizations_url: String,
    /// Repositories URL
    pub repos_url: String,
    /// Received events URL
    pub received_events_url: String,
    /// User type ("User" or "Organization")
    pub r#type: String,
    /// Search relevance score
    pub score: f64,
    /// Following URL template
    pub following_url: String,
    /// Gists URL template
    pub gists_url: String,
    /// Starred URL template
    pub starred_url: String,
    /// Events URL template
    pub events_url: String,
    /// Whether user is a site admin
    pub site_admin: bool,
    /// Display name
    pub name: Option<String>,
    /// Biography
    pub bio: Option<String>,
    /// Email address
    pub email: Option<String>,
    /// Location
    pub location: Option<String>,
    /// Whether user is hireable
    pub hireable: Option<bool>,
    /// Blog URL
    pub blog: Option<String>,
    /// Company
    pub company: Option<String>,
    /// Number of public repositories
    pub public_repos: Option<i64>,
    /// Number of public gists
    pub public_gists: Option<i64>,
    /// Number of followers
    pub followers: Option<i64>,
    /// Number of users being followed
    pub following: Option<i64>,
    /// Created timestamp
    pub created_at: Option<String>,
    /// Last updated timestamp
    pub updated_at: Option<String>,
    /// Suspended timestamp
    pub suspended_at: Option<String>,
    /// User view type
    pub user_view_type: Option<String>,
    /// Text match highlighting information
    pub text_matches: Option<Vec<TextMatch>>,
}

impl From<UserSearchItem> for Person {
    fn from(user: UserSearchItem) -> Self {
        let (parsed, name) = if let Some(name) = user.name {
            (Person::from_str(&name).ok(), Some(name))
        } else {
            (None, Some(user.login))
        };

        Person {
            options: Box::new(PersonOptions {
                name,
                description: user.bio,
                emails: user.email.map(|email| vec![email]),
                url: Some(user.html_url),
                images: Some(vec![ImageObject::new(user.avatar_url)]),
                ..Default::default()
            }),
            // Parsed names may include given names, family name, honorifics etc
            ..parsed.unwrap_or_default()
        }
    }
}

impl From<UserSearchItem> for Organization {
    fn from(user: UserSearchItem) -> Self {
        Organization {
            name: user.name.or(Some(user.login)),
            options: Box::new(OrganizationOptions {
                description: user.bio,
                url: Some(user.html_url),
                contact_points: user.email.map(|email| {
                    vec![ContactPoint {
                        emails: Some(vec![email]),
                        ..Default::default()
                    }]
                }),
                images: Some(vec![ImageObject::new(user.avatar_url)]),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

impl From<UserSearchItem> for Node {
    fn from(user: UserSearchItem) -> Self {
        match user.r#type.as_str() {
            "Organization" => Node::Organization(user.into()),
            _ => Node::Person(user.into()), // Default to Person for "User" and unknown types
        }
    }
}
