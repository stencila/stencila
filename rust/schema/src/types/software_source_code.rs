//! Generated file, do not edit

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
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct SoftwareSourceCode {
    /// The type of this item
    r#type: MustBe!("SoftwareSourceCode"),

    /// The identifier for this item
    id: String,

    /// Link to the repository where the un-compiled, human readable code and related code is located.
    code_repository: Option<String>,

    /// The computer programming language.
    programming_language: Option<String>,

    /// Target operating system or product to which the code applies.
    target_products: Option<Vec<SoftwareApplication>>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<SoftwareSourceCodeOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct SoftwareSourceCodeOptions {
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

    /// The subject matter of the content.
    about: Option<Vec<ThingType>>,

    /// The authors of this creative work.
    authors: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    content: Option<Vec<Block>>,

    /// Date/time of creation.
    date_created: Option<Date>,

    /// Date/time that work was received.
    date_received: Option<Date>,

    /// Date/time of acceptance.
    date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    date_modified: Option<Date>,

    /// Date of first publication.
    date_published: Option<Date>,

    /// People who edited the `CreativeWork`.
    editors: Option<Vec<Person>>,

    /// People or organizations that funded the `CreativeWork`.
    funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    genre: Option<Vec<String>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    keywords: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    is_part_of: Option<Box<CreativeWorkType>>,

    /// License documents that applies to this content, typically indicated by URL.
    licenses: Option<Vec<CreativeWorkTypeOrString>>,

    /// The people or organizations who maintain this CreativeWork.
    maintainers: Option<Vec<PersonOrOrganization>>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    references: Option<Vec<CreativeWorkTypeOrString>>,

    /// The textual content of this creative work.
    text: Option<String>,

    /// The title of the creative work.
    title: Option<Vec<Inline>>,

    /// The version of the creative work.
    version: Option<StringOrNumber>,

    /// What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
    code_sample_type: Option<String>,

    /// Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0).
    runtime_platform: Option<Vec<String>>,

    /// Dependency requirements for the software.
    software_requirements: Option<Vec<SoftwareSourceCodeOrSoftwareApplicationOrString>>,
}
