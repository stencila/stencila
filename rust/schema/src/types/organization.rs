// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::brand::Brand;
use super::contact_point::ContactPoint;
use super::image_object::ImageObject;
use super::person_or_organization::PersonOrOrganization;
use super::postal_address_or_string::PostalAddressOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;

/// An organization such as a school, NGO, corporation, club, etc.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Organization")]
#[jats(elem = "institution")]
pub struct Organization {
    /// The type of this item.
    pub r#type: MustBe!("Organization"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<OrganizationOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct OrganizationOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    #[serde(alias = "identifier")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[serde(alias = "image")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    #[strip(metadata)]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    pub url: Option<String>,

    /// Postal address for the organization.
    pub address: Option<PostalAddressOrString>,

    /// Brands that the organization is connected with.
    #[serde(alias = "brand")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub brands: Option<Vec<Brand>>,

    /// Correspondence/Contact points for the organization.
    #[serde(alias = "contact-points", alias = "contact_points", alias = "contactPoint", alias = "contact-point", alias = "contact_point")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub contact_points: Option<Vec<ContactPoint>>,

    /// Departments within the organization. For example, Department of Computer Science, Research & Development etc.
    #[serde(alias = "department")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub departments: Option<Vec<Organization>>,

    /// Organization(s) or person(s) funding the organization.
    #[serde(alias = "funder")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// The official name of the organization, e.g. the registered company name.
    #[serde(alias = "legal-name", alias = "legal_name")]
    pub legal_name: Option<String>,

    /// The logo of the organization.
    pub logo: Option<ImageObject>,

    /// Person(s) or organization(s) who are members of this organization.
    #[serde(alias = "member")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    pub members: Option<Vec<PersonOrOrganization>>,

    /// Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
    #[serde(alias = "parent-organization", alias = "parent_organization")]
    pub parent_organization: Option<Organization>,
}

impl Organization {
    const NICK: &'static str = "org";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Organization
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
