//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::organization::Organization;
use super::organization_or_person::OrganizationOrPerson;
use super::postal_address_or_string::PostalAddressOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A person (alive, dead, undead, or fictional).
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Person {
    /// The type of this item
    r#type: MustBe!("Person"),

    /// The identifier for this item
    id: String,

    /// Organizations that the person is affiliated with.
    affiliations: Option<Vec<Organization>>,

    /// Family name. In the U.S., the last name of a person.
    family_names: Option<Vec<String>>,

    /// Given name. In the U.S., the first name of a person.
    given_names: Option<Vec<String>>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<PersonOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PersonOptions {
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

    /// Postal address for the person.
    address: Option<PostalAddressOrString>,

    /// Email addresses for the person.
    emails: Option<Vec<String>>,

    /// A person or organization that supports (sponsors) something through some kind of financial contribution.
    funders: Option<Vec<OrganizationOrPerson>>,

    /// An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
    honorific_prefix: Option<String>,

    /// An honorific suffix after a person's name such as MD/PhD/MSCSW.
    honorific_suffix: Option<String>,

    /// The job title of the person (for example, Financial Manager).
    job_title: Option<String>,

    /// An organization (or program membership) to which this person belongs.
    member_of: Option<Vec<Organization>>,

    /// Telephone numbers for the person.
    telephone_numbers: Option<Vec<String>>,
}
