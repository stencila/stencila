// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::brand::Brand;
use super::contact_point::ContactPoint;
use super::image_object_or_string::ImageObjectOrString;
use super::organization_or_person::OrganizationOrPerson;
use super::postal_address_or_string::PostalAddressOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// An organization such as a school, NGO, corporation, club, etc.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Organization {
    /// The type of this item
    pub r#type: MustBe!("Organization"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<OrganizationOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct OrganizationOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,

    /// Postal address for the organization.
    pub address: Option<PostalAddressOrString>,

    /// Brands that the organization is connected with.
    pub brands: Option<Vec<Brand>>,

    /// Correspondence/Contact points for the organization.
    pub contact_points: Option<Vec<ContactPoint>>,

    /// Departments within the organization. For example, Department of Computer Science, Research & Development etc.
    pub departments: Option<Vec<Organization>>,

    /// Organization(s) or person(s) funding the organization.
    pub funders: Option<Vec<OrganizationOrPerson>>,

    /// The official name of the organization, e.g. the registered company name.
    pub legal_name: Option<String>,

    /// The logo of the organization.
    pub logo: Option<Box<ImageObjectOrString>>,

    /// Person(s) or organization(s) who are members of this organization.
    pub members: Option<Vec<OrganizationOrPerson>>,

    /// Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
    pub parent_organization: Option<Box<Organization>>,
}

impl Organization {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
