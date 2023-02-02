//! Generated file, do not edit

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
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Organization {
    /// The type of this item
    r#type: MustBe!("Organization"),

    /// The identifier for this item
    id: String,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<OrganizationOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct OrganizationOptions {
    /// Alternate names (aliases) for the item.
    alternate_names: Option<Vec<String>>,

    /// A description of the item.
    description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    name: Option<String>,

    /// The URL of the item.
    url: Option<String>,

    /// Postal address for the organization.
    address: Option<PostalAddressOrString>,

    /// Brands that the organization is connected with.
    brands: Option<Vec<Brand>>,

    /// Correspondence/Contact points for the organization.
    contact_points: Option<Vec<ContactPoint>>,

    /// Departments within the organization. For example, Department of Computer Science, Research & Development etc.
    departments: Option<Vec<Organization>>,

    /// Organization(s) or person(s) funding the organization.
    funders: Option<Vec<OrganizationOrPerson>>,

    /// The official name of the organization, e.g. the registered company name.
    legal_name: Option<String>,

    /// The logo of the organization.
    logo: Option<Box<ImageObjectOrString>>,

    /// Person(s) or organization(s) who are members of this organization.
    members: Option<Vec<OrganizationOrPerson>>,

    /// Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
    parent_organization: Option<Box<Organization>>,
}
