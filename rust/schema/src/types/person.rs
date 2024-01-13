// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::image_object::ImageObject;
use super::organization::Organization;
use super::person_or_organization::PersonOrOrganization;
use super::postal_address_or_string::PostalAddressOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;

/// A person (alive, dead, undead, or fictional).
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Person")]
pub struct Person {
    /// The type of this item.
    pub r#type: MustBe!("Person"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Organizations that the person is affiliated with.
    #[serde(alias = "affiliation")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub affiliations: Option<Vec<Organization>>,

    /// Family name. In the U.S., the last name of a person.
    #[serde(alias = "familyName", alias = "surname", alias = "surnames", alias = "lastName", alias = "lastNames", alias = "family-names", alias = "family_names", alias = "family-name", alias = "family_name")]
    #[serde(default, deserialize_with = "option_ssv_or_array")]
    pub family_names: Option<Vec<String>>,

    /// Given name. In the U.S., the first name of a person.
    #[serde(alias = "firstName", alias = "firstNames", alias = "given-names", alias = "given_names", alias = "givenName", alias = "given-name", alias = "given_name")]
    #[serde(default, deserialize_with = "option_ssv_or_array")]
    pub given_names: Option<Vec<String>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<PersonOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PersonOptions {
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

    /// Postal address for the person.
    pub address: Option<PostalAddressOrString>,

    /// Email addresses for the person.
    #[serde(alias = "email")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    pub emails: Option<Vec<String>>,

    /// A person or organization that supports (sponsors) something through some kind of financial contribution.
    #[serde(alias = "funder")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
    #[serde(alias = "prefix", alias = "honorific-prefix", alias = "honorific_prefix")]
    pub honorific_prefix: Option<String>,

    /// An honorific suffix after a person's name such as MD/PhD/MSCSW.
    #[serde(alias = "suffix", alias = "honorific-suffix", alias = "honorific_suffix")]
    pub honorific_suffix: Option<String>,

    /// The job title of the person (for example, Financial Manager).
    #[serde(alias = "job-title", alias = "job_title")]
    pub job_title: Option<String>,

    /// An organization (or program membership) to which this person belongs.
    #[serde(alias = "member-of", alias = "member_of")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub member_of: Option<Vec<Organization>>,

    /// Telephone numbers for the person.
    #[serde(alias = "telephone", alias = "telephone-numbers", alias = "telephone_numbers", alias = "telephoneNumber", alias = "telephone-number", alias = "telephone_number")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    pub telephone_numbers: Option<Vec<String>>,
}

impl Person {
    const NICK: &'static str = "per";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Person
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
