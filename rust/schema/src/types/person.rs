// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::organization::Organization;
use super::organization_or_person::OrganizationOrPerson;
use super::postal_address_or_string::PostalAddressOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A person (alive, dead, undead, or fictional).
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Person {
    /// The type of this item
    pub r#type: MustBe!("Person"),

    /// The identifier for this item
    pub id: Option<String>,

    /// Organizations that the person is affiliated with.
    pub affiliations: Option<Vec<Organization>>,

    /// Family name. In the U.S., the last name of a person.
    pub family_names: Option<Vec<String>>,

    /// Given name. In the U.S., the first name of a person.
    pub given_names: Option<Vec<String>>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<PersonOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PersonOptions {
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

    /// Postal address for the person.
    pub address: Option<PostalAddressOrString>,

    /// Email addresses for the person.
    pub emails: Option<Vec<String>>,

    /// A person or organization that supports (sponsors) something through
    /// some kind of financial contribution.
    pub funders: Option<Vec<OrganizationOrPerson>>,

    /// An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
    pub honorific_prefix: Option<String>,

    /// An honorific suffix after a person's name such as MD/PhD/MSCSW.
    pub honorific_suffix: Option<String>,

    /// The job title of the person (for example, Financial Manager).
    pub job_title: Option<String>,

    /// An organization (or program membership) to which this person belongs.
    pub member_of: Option<Vec<Organization>>,

    /// Telephone numbers for the person.
    pub telephone_numbers: Option<Vec<String>>,
}

impl Person {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
