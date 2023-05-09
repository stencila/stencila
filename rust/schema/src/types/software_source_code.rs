// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::comment::Comment;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_type_or_string::CreativeWorkTypeOrString;
use super::date::Date;
use super::grant_or_monetary_grant::GrantOrMonetaryGrant;
use super::image_object_or_string::ImageObjectOrString;
use super::inline::Inline;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::software_application::SoftwareApplication;
use super::software_source_code_or_software_application_or_string::SoftwareSourceCodeOrSoftwareApplicationOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::thing_type::ThingType;

/// Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct SoftwareSourceCode {
    /// The type of this item
    pub r#type: MustBe!("SoftwareSourceCode"),

    /// The identifier for this item
    pub id: Option<String>,

    /// Link to the repository where the un-compiled, human readable code and related
    /// code is located.
    pub code_repository: Option<String>,

    /// The computer programming language.
    pub programming_language: Option<String>,

    /// Target operating system or product to which the code applies.
    pub target_products: Option<Vec<SoftwareApplication>>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<SoftwareSourceCodeOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct SoftwareSourceCodeOptions {
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

    /// The subject matter of the content.
    pub about: Option<Vec<ThingType>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Block>>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// Keywords or tags used to describe this content.
    /// Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Box<CreativeWorkType>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypeOrString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrOrganization>>,

    /// Elements of the collection which can be a variety of different elements,
    /// such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication,
    /// web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypeOrString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<Vec<Inline>>,

    /// The version of the creative work.
    pub version: Option<StringOrNumber>,

    /// What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
    pub code_sample_type: Option<String>,

    /// Runtime platform or script interpreter dependencies (Example - Java v1,
    /// Python2.3, .Net Framework 3.0).
    pub runtime_platform: Option<Vec<String>>,

    /// Dependency requirements for the software.
    pub software_requirements: Option<Vec<SoftwareSourceCodeOrSoftwareApplicationOrString>>,
}

impl SoftwareSourceCode {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
