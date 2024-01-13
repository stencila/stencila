// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author_role_name::AuthorRoleName;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::string::String;

/// An author and their role.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "AuthorRole")]
pub struct AuthorRole {
    /// The type of this item.
    pub r#type: MustBe!("AuthorRole"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The author.
    pub author: PersonOrOrganizationOrSoftwareApplication,

    /// A role played by the author.
    #[serde(alias = "role-name", alias = "role_name")]
    pub role_name: AuthorRoleName,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl AuthorRole {
    const NICK: &'static str = "aut";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::AuthorRole
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(author: PersonOrOrganizationOrSoftwareApplication, role_name: AuthorRoleName) -> Self {
        Self {
            author,
            role_name,
            ..Default::default()
        }
    }
}
