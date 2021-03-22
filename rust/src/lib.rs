
#![allow(clippy::large_enum_variant)]

use std::sync::Arc;

type Null = serde_json::Value;
type Bool = bool;
type Integer = i32;
type Number = f32;
type Array = Vec<serde_json::Value>;
type Object = std::collections::HashMap<String, serde_json::Value>;

// Structs for each type


/// Entity
///
/// The most simple compound (ie. non-atomic like `number`, `string` etc) type.
#[derive(Debug, Default)]
pub struct Entity {
    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// ArrayValidator
///
/// A validator specifying constraints on an array node.
#[derive(Debug, Default)]
pub struct ArrayValidator {
    /// An array node is valid if at least one of its items is valid against the `contains` schema.
    pub contains: Option<Arc<ValidatorTypes>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Another validator node specifying the constraints on all items in the array.
    pub items_validator: Option<Arc<ValidatorTypes>>,

    /// An array node is valid if its size is less than, or equal to, this value.
    pub max_items: Option<Number>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// An array node is valid if its size is greater than, or equal to, this value.
    pub min_items: Option<Number>,

    /// A flag to indicate that each value in the array should be unique.
    pub unique_items: Option<Bool>,
}

/// BooleanValidator
///
/// A schema specifying that a node must be a boolean value.
#[derive(Debug, Default)]
pub struct BooleanValidator {
    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Cite
///
/// A reference to a CreativeWork that is cited in another CreativeWork.
#[derive(Debug, Default)]
pub struct Cite {
    /// The target of the citation (URL or reference ID).
    pub target: String,

    /// Determines how the citation is shown within the surrounding text.
    pub citation_mode: Option<CiteCitationMode>,

    /// Text to show before the citation.
    pub citation_prefix: Option<String>,

    /// Text to show after the citation.
    pub citation_suffix: Option<String>,

    /// Optional structured content/text of this citation.
    pub content: Option<Vec<InlineContent>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The page on which the work ends; for example "138" or "xvi".
    pub page_end: Option<IntegerString>,

    /// The page on which the work starts; for example "135" or "xiii".
    pub page_start: Option<IntegerString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pub pagination: Option<String>,
}

/// CiteGroup
///
/// A group of `Cite` nodes
#[derive(Debug, Default)]
pub struct CiteGroup {
    /// One or more `Cite`s to be referenced in the same surrounding text.
    pub items: Vec<Cite>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Code
///
/// Base type for code nodes e.g. `CodeBlock`, `CodeExpression`.
#[derive(Debug, Default)]
pub struct Code {
    /// The text of the code.
    pub text: String,

    /// Media type, typically expressed using a MIME format, of the code.
    pub format: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The programming language of the code.
    pub programming_language: Option<String>,
}

/// CodeBlock
///
/// A code block.
#[derive(Debug, Default)]
pub struct CodeBlock {
    /// The text of the code.
    pub text: String,

    /// A compilation directive giving the name of the variable to export into the content of the code block.
    pub export_from: Option<String>,

    /// Media type, typically expressed using a MIME format, of the code.
    pub format: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// A compilation directive giving the name of the variable to import the content of the code block as.
    pub import_to: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The programming language of the code.
    pub programming_language: Option<String>,
}

/// CodeChunk
///
/// A executable chunk of code.
#[derive(Debug, Default)]
pub struct CodeChunk {
    /// The text of the code.
    pub text: String,

    /// Names of variables that the code chunk alters.
    pub alters: Option<Vec<String>>,

    /// Variables that the code chunk assigns to.
    pub assigns: Option<Vec<StringVariable>>,

    /// A caption for the CodeChunk.
    pub caption: Option<StringVecNode>,

    /// Variables that the code chunk declares.
    pub declares: Option<Vec<StringVariableFunction>>,

    /// Duration in seconds of the last execution of the chunk.
    pub duration: Option<Number>,

    /// Errors when compiling or executing the chunk.
    pub errors: Option<Vec<CodeError>>,

    /// A compilation directive giving the name of the variable to export into the content of the code block.
    pub export_from: Option<String>,

    /// Media type, typically expressed using a MIME format, of the code.
    pub format: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// A compilation directive giving the name of the variable to import the content of the code block as.
    pub import_to: Option<String>,

    /// Software packages that the code chunk imports
    pub imports: Option<Vec<StringSoftwareSourceCodeSoftwareApplication>>,

    /// A short label for the CodeChunk.
    pub label: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// Outputs from executing the chunk.
    pub outputs: Option<Vec<Node>>,

    /// The programming language of the code.
    pub programming_language: Option<String>,

    /// Filesystem paths that this code chunk reads from.
    pub reads: Option<Vec<String>>,

    /// Names of variables that the code chunk uses (but does not alter).
    pub uses: Option<Vec<StringVariable>>,
}

/// CodeFragment
///
/// Inline code.
#[derive(Debug, Default)]
pub struct CodeFragment {
    /// The text of the code.
    pub text: String,

    /// Media type, typically expressed using a MIME format, of the code.
    pub format: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The programming language of the code.
    pub programming_language: Option<String>,
}

/// CodeExpression
///
/// An expression defined in programming language source code.
#[derive(Debug, Default)]
pub struct CodeExpression {
    /// The text of the code.
    pub text: String,

    /// Errors when compiling or executing the chunk.
    pub errors: Option<Vec<CodeError>>,

    /// Media type, typically expressed using a MIME format, of the code.
    pub format: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The value of the expression when it was last evaluated.
    pub output: Option<Node>,

    /// The programming language of the code.
    pub programming_language: Option<String>,
}

/// CodeError
///
/// An error that occurred when parsing, compiling or executing a Code node.
#[derive(Debug, Default)]
pub struct CodeError {
    /// The error message or brief description of the error.
    pub error_message: String,

    /// The type of error e.g. "SyntaxError", "ZeroDivisionError".
    pub error_type: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// Stack trace leading up to the error.
    pub stack_trace: Option<String>,
}

/// ConstantValidator
///
/// A validator specifying a constant value that a node must have.
#[derive(Debug, Default)]
pub struct ConstantValidator {
    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The value that the node must have.
    pub value: Option<Node>,
}

/// Date
///
/// A date encoded as a ISO 8601 string.
#[derive(Debug)]
pub struct Date {
    /// The date as an ISO 8601 string.
    pub value: DateValue,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Mark
///
/// A base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).
#[derive(Debug, Default)]
pub struct Mark {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Delete
///
/// Content that is marked for deletion
#[derive(Debug, Default)]
pub struct Delete {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Emphasis
///
/// Emphasised content.
#[derive(Debug, Default)]
pub struct Emphasis {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Thing
///
/// The most generic type of item.
#[derive(Debug, Default)]
pub struct Thing {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// Brand
///
/// A brand used by an organization or person for labeling a product, product group, or similar.
#[derive(Debug, Default)]
pub struct Brand {
    /// The name of the item.
    pub name: String,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// A logo associated with the brand.
    pub logo: Option<StringImageObject>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// Reviews of the brand.
    pub reviews: Option<Vec<String>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// ContactPoint
///
/// A contact point, for example, a R&D department.
#[derive(Debug, Default)]
pub struct ContactPoint {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// Languages (human not programming) in which it is possible to communicate with the organization/department etc.
    pub available_languages: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// Email address for correspondence.
    pub emails: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Telephone numbers for the contact point.
    pub telephone_numbers: Option<Vec<String>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// CreativeWork
///
/// A creative work, including books, movies, photographs, software programs, etc.
#[derive(Debug, Default)]
pub struct CreativeWork {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// Article
///
/// An article, including news and scholarly articles.
#[derive(Debug, Default)]
pub struct Article {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The page on which the article ends; for example "138" or "xvi".
    pub page_end: Option<IntegerString>,

    /// The page on which the article starts; for example "135" or "xiii".
    pub page_start: Option<IntegerString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pub pagination: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// Collection
///
/// A created collection of CreativeWorks or other artefacts.
#[derive(Debug, Default)]
pub struct Collection {
    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Vec<CreativeWorkTypes>,

    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// Comment
///
/// A comment on an item, e.g on a Article, or SoftwareSourceCode.
#[derive(Debug, Default)]
pub struct Comment {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// The part or facet of the item that is being commented on.
    pub comment_aspect: Option<String>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The parent comment of this comment.
    pub parent_item: Option<Arc<Comment>>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// Datatable
///
/// A table of data.
#[derive(Debug, Default)]
pub struct Datatable {
    /// The columns of data.
    pub columns: Vec<DatatableColumn>,

    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// MediaObject
///
/// A media object, such as an image, video, or audio object embedded in a web page or a downloadable dataset.
#[derive(Debug, Default)]
pub struct MediaObject {
    /// URL for the actual bytes of the media object, for example the image file or video file.
    pub content_url: String,

    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    pub bitrate: Option<Number>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// File size in megabits (Mbit, Mb).
    pub content_size: Option<Number>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    pub embed_url: Option<String>,

    /// Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
    pub format: Option<String>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// AudioObject
///
/// An audio file
#[derive(Debug, Default)]
pub struct AudioObject {
    /// URL for the actual bytes of the media object, for example the image file or video file.
    pub content_url: String,

    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    pub bitrate: Option<Number>,

    /// The caption for this audio recording.
    pub caption: Option<String>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// File size in megabits (Mbit, Mb).
    pub content_size: Option<Number>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    pub embed_url: Option<String>,

    /// Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
    pub format: Option<String>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The transcript of this audio recording.
    pub transcript: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// DatatableColumn
///
/// A column of data within a Datatable.
#[derive(Debug, Default)]
pub struct DatatableColumn {
    /// The name of the item.
    pub name: String,

    /// The data values of the column.
    pub values: Array,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The validator to use to validate data in the column.
    pub validator: Option<ArrayValidator>,
}

/// DefinedTerm
///
/// A word, name, acronym, phrase, etc. with a formal definition.
#[derive(Debug, Default)]
pub struct DefinedTerm {
    /// The name of the item.
    pub name: String,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// A code that identifies this DefinedTerm within a DefinedTermSet
    pub term_code: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// EnumValidator
///
/// A schema specifying that a node must be one of several values.
#[derive(Debug, Default)]
pub struct EnumValidator {
    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// A node is valid if it is equal to any of these values.
    pub values: Option<Vec<Node>>,
}

/// Figure
///
/// Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them.
#[derive(Debug, Default)]
pub struct Figure {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// A caption for the figure.
    pub caption: Option<StringVecNode>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// A short label for the figure.
    pub label: Option<String>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// Function
///
/// A function with a name, which might take Parameters and return a value of a certain type.
#[derive(Debug, Default)]
pub struct Function {
    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the function.
    pub name: Option<String>,

    /// The parameters of the function.
    pub parameters: Option<Vec<Parameter>>,

    /// The return type of the function.
    pub returns: Option<ValidatorTypes>,
}

/// Grant
///
/// A grant, typically financial or otherwise quantifiable, of resources.
#[derive(Debug, Default)]
pub struct Grant {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// Indicates an item funded or sponsored through a Grant.
    pub funded_items: Option<Vec<Thing>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// A person or organization that supports a thing through a pledge, promise, or financial contribution.
    pub sponsors: Option<Vec<PersonOrganization>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// Heading
///
/// Heading
#[derive(Debug, Default)]
pub struct Heading {
    /// Content of the heading.
    pub content: Vec<InlineContent>,

    /// The depth of the heading.
    pub depth: Option<Number>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// ImageObject
///
/// An image file.
#[derive(Debug, Default)]
pub struct ImageObject {
    /// URL for the actual bytes of the media object, for example the image file or video file.
    pub content_url: String,

    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    pub bitrate: Option<Number>,

    /// The caption for this image.
    pub caption: Option<String>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// File size in megabits (Mbit, Mb).
    pub content_size: Option<Number>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    pub embed_url: Option<String>,

    /// Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
    pub format: Option<String>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<Arc<PersonOrganization>>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// Thumbnail image of this image.
    pub thumbnail: Option<Arc<ImageObject>>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// Include
///
/// A directive to include content from an external source (e.g. file, URL) or content.
#[derive(Debug, Default)]
pub struct Include {
    /// The source of the content, a URL or file path, or the content itself.
    pub source: String,

    /// The content to be included.
    pub content: Option<Vec<BlockContent>>,

    /// Media type, typically expressed using a MIME format, of the source content.
    pub format: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// NumberValidator
///
/// A validator specifying the constraints on a numeric node.
#[derive(Debug, Default)]
pub struct NumberValidator {
    /// The exclusive upper limit for a numeric node.
    pub exclusive_maximum: Option<Number>,

    /// The exclusive lower limit for a numeric node.
    pub exclusive_minimum: Option<Number>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// The inclusive upper limit for a numeric node.
    pub maximum: Option<Number>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The inclusive lower limit for a numeric node.
    pub minimum: Option<Number>,

    /// A number that a numeric node must be a multiple of.
    pub multiple_of: Option<Number>,
}

/// IntegerValidator
///
/// A validator specifying the constraints on an integer node.
#[derive(Debug, Default)]
pub struct IntegerValidator {
    /// The exclusive upper limit for a numeric node.
    pub exclusive_maximum: Option<Number>,

    /// The exclusive lower limit for a numeric node.
    pub exclusive_minimum: Option<Number>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// The inclusive upper limit for a numeric node.
    pub maximum: Option<Number>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The inclusive lower limit for a numeric node.
    pub minimum: Option<Number>,

    /// A number that a numeric node must be a multiple of.
    pub multiple_of: Option<Number>,
}

/// Link
///
/// A hyperlink to other pages, sections within the same document, resources, or any URL.
#[derive(Debug, Default)]
pub struct Link {
    /// The textual content of the link.
    pub content: Vec<InlineContent>,

    /// The target of the link.
    pub target: String,

    /// A compilation directive giving the name of the variable to export to the link target.
    pub export_from: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// A compilation directive giving the name of the variable to import the link target as.
    pub import_to: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The relation between the target and the current thing.
    pub relation: Option<String>,

    /// A title for the link.
    pub title: Option<String>,
}

/// List
///
/// A list of items.
#[derive(Debug, Default)]
pub struct List {
    /// The items in the list
    pub items: Vec<ListItem>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// Type of ordering.
    pub order: Option<ListOrder>,
}

/// ListItem
///
/// A single item in a list.
#[derive(Debug, Default)]
pub struct ListItem {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The content of the list item.
    pub content: Option<Vec<Node>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// A flag to indicate if this list item is checked.
    pub is_checked: Option<Bool>,

    /// The item represented by this list item.
    pub item: Option<Node>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The position of the item in a series or sequence of items.
    pub position: Option<Number>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// Math
///
/// A mathematical variable or equation.
#[derive(Debug, Default)]
pub struct Math {
    /// The text of the equation in the language.
    pub text: String,

    /// Errors that occurred when parsing the math equation.
    pub errors: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// The language used for the equation e.g tex, mathml, asciimath.
    pub math_language: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// MathBlock
///
/// A block of math, e.g an equation, to be treated as block content.
#[derive(Debug, Default)]
pub struct MathBlock {
    /// The text of the equation in the language.
    pub text: String,

    /// Errors that occurred when parsing the math equation.
    pub errors: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// A short label for the math block.
    pub label: Option<String>,

    /// The language used for the equation e.g tex, mathml, asciimath.
    pub math_language: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// MathFragment
///
/// A fragment of math, e.g a variable name, to be treated as inline content.
#[derive(Debug, Default)]
pub struct MathFragment {
    /// The text of the equation in the language.
    pub text: String,

    /// Errors that occurred when parsing the math equation.
    pub errors: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// The language used for the equation e.g tex, mathml, asciimath.
    pub math_language: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// MonetaryGrant
///
/// A monetary grant.
#[derive(Debug, Default)]
pub struct MonetaryGrant {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The amount of money.
    pub amounts: Option<Number>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// Indicates an item funded or sponsored through a Grant.
    pub funded_items: Option<Vec<Thing>>,

    /// A person or organization that supports (sponsors) something through some kind of financial contribution.
    pub funders: Option<Vec<PersonOrganization>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// A person or organization that supports a thing through a pledge, promise, or financial contribution.
    pub sponsors: Option<Vec<PersonOrganization>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// NontextualAnnotation
///
/// Inline text that has a non-textual annotation.
#[derive(Debug, Default)]
pub struct NontextualAnnotation {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Organization
///
/// An organization such as a school, NGO, corporation, club, etc.
#[derive(Debug, Default)]
pub struct Organization {
    /// Postal address for the organization.
    pub address: Option<StringPostalAddress>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// Brands that the organization is connected with.
    pub brands: Option<Vec<Brand>>,

    /// Correspondence/Contact points for the organization.
    pub contact_points: Option<Vec<ContactPoint>>,

    /// Departments within the organization. For example, Department of Computer Science, Research & Development etc.
    pub departments: Option<Vec<Organization>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// Organization(s) or person(s) funding the organization.
    pub funders: Option<Vec<OrganizationPerson>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Legal name for the Organization. Should only include letters and spaces.
    pub legal_name: Option<String>,

    /// The logo of the organization.
    pub logo: Option<StringImageObject>,

    /// Person(s) or organization(s) who are members of this organization.
    pub members: Option<Vec<OrganizationPerson>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Entity that the Organization is a part of. For example, parentOrganization to a department is a university.
    pub parent_organization: Option<Arc<Organization>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// Paragraph
///
/// Paragraph
#[derive(Debug, Default)]
pub struct Paragraph {
    /// The contents of the paragraph.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Variable
///
/// A variable representing a name / value pair.
#[derive(Debug, Default)]
pub struct Variable {
    /// The name of the variable.
    pub name: String,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Whether or not a property is mutable. Default is false.
    pub is_readonly: Option<Bool>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The validator that the value is validated against.
    pub validator: Option<ValidatorTypes>,

    /// The value of the variable.
    pub value: Option<Node>,
}

/// Parameter
///
/// A parameter that can be set and used in evaluated code.
#[derive(Debug, Default)]
pub struct Parameter {
    /// The name of the variable.
    pub name: String,

    /// The default value of the parameter.
    pub default: Option<Node>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Indicates that this parameter is variadic and can accept multiple named arguments.
    pub is_extensible: Option<Bool>,

    /// Whether or not a property is mutable. Default is false.
    pub is_readonly: Option<Bool>,

    /// Is this parameter required, if not it should have a default or default is assumed to be null.
    pub is_required: Option<Bool>,

    /// Indicates that this parameter is variadic and can accept multiple arguments.
    pub is_variadic: Option<Bool>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The validator that the value is validated against.
    pub validator: Option<ValidatorTypes>,

    /// The value of the variable.
    pub value: Option<Node>,
}

/// Periodical
///
/// A periodical publication.
#[derive(Debug, Default)]
pub struct Periodical {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// The date this Periodical ceased publication.
    pub date_end: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// The date this Periodical was first published.
    pub date_start: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// The International Standard Serial Number(s) (ISSN) that identifies this serial publication.
    pub issns: Option<Vec<String>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// Person
///
/// A person (alive, dead, undead, or fictional).
#[derive(Debug, Default)]
pub struct Person {
    /// Postal address for the person.
    pub address: Option<StringPostalAddress>,

    /// Organizations that the person is affiliated with.
    pub affiliations: Option<Vec<Organization>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// Email addresses for the person.
    pub emails: Option<Vec<String>>,

    /// Family name. In the U.S., the last name of a person.
    pub family_names: Option<Vec<String>>,

    /// A person or organization that supports (sponsors) something through some kind of financial contribution.
    pub funders: Option<Vec<OrganizationPerson>>,

    /// Given name. In the U.S., the first name of a person.
    pub given_names: Option<Vec<String>>,

    /// An honorific prefix preceding a person's name such as Dr/Mrs/Mr.
    pub honorific_prefix: Option<String>,

    /// An honorific suffix after a person's name such as MD/PhD/MSCSW.
    pub honorific_suffix: Option<String>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// The job title of the person (for example, Financial Manager).
    pub job_title: Option<String>,

    /// An organization (or program membership) to which this person belongs.
    pub member_of: Option<Vec<Organization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Telephone numbers for the person.
    pub telephone_numbers: Option<Vec<String>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// PostalAddress
///
/// A physical mailing address.
#[derive(Debug, Default)]
pub struct PostalAddress {
    /// The country.
    pub address_country: Option<String>,

    /// The locality in which the street address is, and which is in the region.
    pub address_locality: Option<String>,

    /// The region in which the locality is, and which is in the country.
    pub address_region: Option<String>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// Languages (human not programming) in which it is possible to communicate with the organization/department etc.
    pub available_languages: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// Email address for correspondence.
    pub emails: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The post office box number.
    pub post_office_box_number: Option<String>,

    /// The postal code.
    pub postal_code: Option<String>,

    /// The street address.
    pub street_address: Option<String>,

    /// Telephone numbers for the contact point.
    pub telephone_numbers: Option<Vec<String>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// Product
///
/// Any offered product or service. For example, a pair of shoes; a haircut; or an episode of a TV show streamed online.
#[derive(Debug, Default)]
pub struct Product {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// Brands that the product is labelled with.
    pub brands: Option<Vec<Brand>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// The logo of the product.
    pub logo: Option<StringImageObject>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Product identification code.
    pub product_id: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// PropertyValue
///
/// A property-value pair.
#[derive(Debug)]
pub struct PropertyValue {
    /// The value of the property.
    pub value: Node,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// A commonly used identifier for the characteristic represented by the property.
    pub property_id: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// PublicationIssue
///
/// A part of a successively published publication such as a periodical or publication volume, often numbered.
#[derive(Debug, Default)]
pub struct PublicationIssue {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Identifies the issue of publication; for example, "iii" or "2".
    pub issue_number: Option<IntegerString>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The page on which the issue ends; for example "138" or "xvi".
    pub page_end: Option<IntegerString>,

    /// The page on which the issue starts; for example "135" or "xiii".
    pub page_start: Option<IntegerString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pub pagination: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// PublicationVolume
///
/// A part of a successively published publication such as a periodical or multi-volume work.
#[derive(Debug, Default)]
pub struct PublicationVolume {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The page on which the volume ends; for example "138" or "xvi".
    pub page_end: Option<IntegerString>,

    /// The page on which the volume starts; for example "135" or "xiii".
    pub page_start: Option<IntegerString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pub pagination: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,

    /// Identifies the volume of publication or multi-part work; for example, "iii" or "2".
    pub volume_number: Option<IntegerString>,
}

/// Quote
///
/// Inline, quoted content.
#[derive(Debug, Default)]
pub struct Quote {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The source of the quote.
    pub cite: Option<CiteString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// QuoteBlock
///
/// A section quoted from somewhere else.
#[derive(Debug, Default)]
pub struct QuoteBlock {
    /// The content of the quote.
    pub content: Vec<BlockContent>,

    /// The source of the quote.
    pub cite: Option<CiteString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Review
///
/// A review of an item, e.g of an Article, or SoftwareSourceCode.
#[derive(Debug, Default)]
pub struct Review {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// The item that is being reviewed.
    pub item_reviewed: Option<Thing>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The part or facet of the item that is being reviewed.
    pub review_aspect: Option<String>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// SoftwareApplication
///
/// A software application.
#[derive(Debug, Default)]
pub struct SoftwareApplication {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// Requirements for application, including shared libraries that are not included in the application distribution.
    pub software_requirements: Option<Vec<SoftwareApplication>>,

    /// Version of the software.
    pub software_version: Option<String>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// SoftwareEnvironment
///
/// A computational environment.
#[derive(Debug, Default)]
pub struct SoftwareEnvironment {
    /// The name of the item.
    pub name: String,

    /// The packages that this environment adds to the base environments listed under `extends` (if any).,
    pub adds: Option<Vec<SoftwareSourceCode>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// Other environments that this environment extends by adding or removing packages.,
    pub extends: Option<Vec<SoftwareEnvironment>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The packages that this environment removes from the base environments listed under `extends` (if any).,
    pub removes: Option<Vec<SoftwareSourceCode>>,

    /// The URL of the item.
    pub url: Option<String>,
}

/// SoftwareSession
///
/// Definition of a compute session, including its software and compute resource requirements and status.
#[derive(Debug, Default)]
pub struct SoftwareSession {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The maximum number of concurrent clients the session is limited to.
    pub clients_limit: Option<Number>,

    /// The maximum number of concurrent clients requested for the session.
    pub clients_request: Option<Number>,

    /// The amount of CPU the session is limited to.
    pub cpu_limit: Option<Number>,

    /// The amount of CPU requested for the session.
    pub cpu_request: Option<Number>,

    /// The date-time that the session ended.
    pub date_end: Option<Date>,

    /// The date-time that the session began.
    pub date_start: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The maximum duration (seconds) the session is limited to.
    pub duration_limit: Option<Number>,

    /// The maximum duration (seconds) requested for the session.
    pub duration_request: Option<Number>,

    /// The software environment to execute this session in.
    pub environment: Option<SoftwareEnvironment>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// The amount of memory that the session is limited to.
    pub memory_limit: Option<Number>,

    /// The amount of memory requested for the session.
    pub memory_request: Option<Number>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// The amount of network data transfer (GiB) that the session is limited to.
    pub network_transfer_limit: Option<Number>,

    /// The amount of network data transfer (GiB) requested for the session.
    pub network_transfer_request: Option<Number>,

    /// The status of the session (starting, stopped, etc).
    pub status: Option<SoftwareSessionStatus>,

    /// The inactivity timeout (seconds) the session is limited to.
    pub timeout_limit: Option<Number>,

    /// The inactivity timeout (seconds) requested for the session.
    pub timeout_request: Option<Number>,

    /// The URL of the item.
    pub url: Option<String>,

    /// Volumes to mount in the session.
    pub volume_mounts: Option<Vec<VolumeMount>>,
}

/// SoftwareSourceCode
///
/// Computer programming source code. Example: Full (compile ready) solutions, code snippet samples, scripts, templates.
#[derive(Debug, Default)]
pub struct SoftwareSourceCode {
    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Link to the repository where the un-compiled, human readable code and related code is located.
    pub code_repository: Option<String>,

    /// What type of code sample: full (compile ready) solution, code snippet, inline code, scripts, template.
    pub code_sample_type: Option<String>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// The computer programming language.
    pub programming_language: Option<String>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// Runtime platform or script interpreter dependencies (Example - Java v1, Python2.3, .Net Framework 3.0).
    pub runtime_platform: Option<Vec<String>>,

    /// Dependency requirements for the software.
    pub software_requirements: Option<Vec<SoftwareSourceCodeSoftwareApplicationString>>,

    /// Target operating system or product to which the code applies.
    pub target_products: Option<Vec<SoftwareApplication>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// StringValidator
///
/// A schema specifying constraints on a string node.
#[derive(Debug, Default)]
pub struct StringValidator {
    /// The identifier for this item.
    pub id: Option<String>,

    /// The maximum length for a string node.
    pub max_length: Option<Number>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The minimum length for a string node.
    pub min_length: Option<Number>,

    /// A regular expression that a string node must match.
    pub pattern: Option<String>,
}

/// Strong
///
/// Strongly emphasised content.
#[derive(Debug, Default)]
pub struct Strong {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Subscript
///
/// Subscripted content.
#[derive(Debug, Default)]
pub struct Subscript {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Superscript
///
/// Superscripted content.
#[derive(Debug, Default)]
pub struct Superscript {
    /// The content that is marked.
    pub content: Vec<InlineContent>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// Table
///
/// A table.
#[derive(Debug, Default)]
pub struct Table {
    /// Rows of cells in the table.
    pub rows: Vec<TableRow>,

    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// A caption for the table.
    pub caption: Option<StringVecNode>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// A short label for the table.
    pub label: Option<String>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// TableCell
///
/// A cell within a `Table`.
#[derive(Debug, Default)]
pub struct TableCell {
    /// Contents of the table cell.
    pub content: Vec<Node>,

    /// Indicates whether the cell is a header or data.
    pub cell_type: Option<TableCellCellType>,

    /// How many columns the cell extends.
    pub colspan: Option<Integer>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the cell.
    pub name: Option<String>,

    /// How many columns the cell extends.
    pub rowspan: Option<Integer>,
}

/// TableRow
///
/// A row within a Table.
#[derive(Debug, Default)]
pub struct TableRow {
    /// An array of cells in the row.
    pub cells: Vec<TableCell>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// If present, indicates that all cells in this row should be treated as header cells.
    pub row_type: Option<TableRowRowType>,
}

/// ThematicBreak
///
/// A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
#[derive(Debug, Default)]
pub struct ThematicBreak {
    /// The identifier for this item.
    pub id: Option<String>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// TupleValidator
///
/// A validator specifying constraints on an array of heterogeneous items.
#[derive(Debug, Default)]
pub struct TupleValidator {
    /// The identifier for this item.
    pub id: Option<String>,

    /// An array of validators specifying the constraints on each successive item in the array.
    pub items: Option<Vec<ValidatorTypes>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,
}

/// VideoObject
///
/// A video file.
#[derive(Debug, Default)]
pub struct VideoObject {
    /// URL for the actual bytes of the media object, for example the image file or video file.
    pub content_url: String,

    /// The subject matter of the content.
    pub about: Option<Vec<Thing>>,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// The authors of this creative work.
    pub authors: Option<Vec<PersonOrganization>>,

    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    pub bitrate: Option<Number>,

    /// The caption for this video recording.
    pub caption: Option<String>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Node>>,

    /// File size in megabits (Mbit, Mb).
    pub content_size: Option<Number>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    pub embed_url: Option<String>,

    /// Media type (MIME type) as per http://www.iana.org/assignments/media-types/media-types.xhtml.
    pub format: Option<String>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantMonetaryGrant>>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrganization>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<Arc<CreativeWorkTypes>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypesString>>,

    /// The people or organizations who maintain this CreativeWork.
    pub maintainers: Option<Vec<PersonOrganization>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// The name of the item.
    pub name: Option<String>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkTypes>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypesString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// Thumbnail image of this video recording.
    pub thumbnail: Option<ImageObject>,

    /// The title of the creative work.
    pub title: Option<VecInlineContentString>,

    /// The transcript of this video recording.
    pub transcript: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The version of the creative work.
    pub version: Option<StringNumber>,
}

/// VolumeMount
///
/// Describes a volume mount from a host to container.
#[derive(Debug, Default)]
pub struct VolumeMount {
    /// The mount location inside the container.
    pub mount_destination: String,

    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<VecBlockContentVecInlineContentString>,

    /// The identifier for this item.
    pub id: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectString>>,

    /// Metadata associated with this item.
    pub meta: Option<Object>,

    /// A list of options to use when applying the mount.
    pub mount_options: Option<Vec<String>>,

    /// The mount source directory on the host.
    pub mount_source: Option<String>,

    /// The type of mount.
    pub mount_type: Option<String>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,
}

// Types for properties that are manually defined

type DateValue = chrono::DateTime::<chrono::Utc>;


// Enums for properties which use JSON Schema 'enum' or 'anyOf'

#[derive(Debug)]
pub enum CiteCitationMode {
    Parenthetical,
    Narrative,
    NarrativeAuthor,
    NarrativeYear,
    Normal,
    SuppressAuthor,
}

#[derive(Debug)]
pub enum IntegerString {
    Integer(Integer),
    String(String),
}

#[derive(Debug)]
pub enum StringVariable {
    String(String),
    Variable(Variable),
}

#[derive(Debug)]
pub enum StringVecNode {
    String(String),
    VecNode(Vec<Node>),
}

#[derive(Debug)]
pub enum StringVariableFunction {
    String(String),
    Variable(Variable),
    Function(Function),
}

#[derive(Debug)]
pub enum StringSoftwareSourceCodeSoftwareApplication {
    String(String),
    SoftwareSourceCode(SoftwareSourceCode),
    SoftwareApplication(SoftwareApplication),
}

#[derive(Debug)]
pub enum VecBlockContentVecInlineContentString {
    VecBlockContent(Vec<BlockContent>),
    VecInlineContent(Vec<InlineContent>),
    String(String),
}

#[derive(Debug)]
pub enum PropertyValueString {
    PropertyValue(PropertyValue),
    String(String),
}

#[derive(Debug)]
pub enum ImageObjectString {
    ImageObject(ImageObject),
    String(String),
}

#[derive(Debug)]
pub enum StringImageObject {
    String(String),
    ImageObject(ImageObject),
}

#[derive(Debug)]
pub enum PersonOrganization {
    Person(Person),
    Organization(Organization),
}

#[derive(Debug)]
pub enum GrantMonetaryGrant {
    Grant(Grant),
    MonetaryGrant(MonetaryGrant),
}

#[derive(Debug)]
pub enum CreativeWorkTypesString {
    CreativeWorkTypes(CreativeWorkTypes),
    String(String),
}

#[derive(Debug)]
pub enum VecInlineContentString {
    VecInlineContent(Vec<InlineContent>),
    String(String),
}

#[derive(Debug)]
pub enum StringNumber {
    String(String),
    Number(Number),
}

#[derive(Debug)]
pub enum ListOrder {
    Ascending,
    Descending,
    Unordered,
}

#[derive(Debug)]
pub enum StringPostalAddress {
    String(String),
    PostalAddress(PostalAddress),
}

#[derive(Debug)]
pub enum OrganizationPerson {
    Organization(Organization),
    Person(Person),
}

#[derive(Debug)]
pub enum CiteString {
    Cite(Cite),
    String(String),
}

#[derive(Debug)]
pub enum SoftwareSessionStatus {
    Unknown,
    Starting,
    Started,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug)]
pub enum SoftwareSourceCodeSoftwareApplicationString {
    SoftwareSourceCode(SoftwareSourceCode),
    SoftwareApplication(SoftwareApplication),
    String(String),
}

#[derive(Debug)]
pub enum TableCellCellType {
    Data,
    Header,
}

#[derive(Debug)]
pub enum TableRowRowType {
    Header,
    Footer,
}


// Enums for "union" types
  
#[derive(Debug)]
/// Union type for valid block content.

pub enum BlockContent {
    CodeBlock(CodeBlock),
    CodeChunk(CodeChunk),
    Collection(Collection),
    Figure(Figure),
    Heading(Heading),
    List(List),
    ListItem(ListItem),
    MathBlock(MathBlock),
    Paragraph(Paragraph),
    QuoteBlock(QuoteBlock),
    Table(Table),
    ThematicBreak(ThematicBreak),
}

#[derive(Debug)]
/// All type schemas that are derived from CodeBlock

pub enum CodeBlockTypes {
    CodeBlock(CodeBlock),
    CodeChunk(CodeChunk),
}

#[derive(Debug)]
/// All type schemas that are derived from CodeFragment

pub enum CodeFragmentTypes {
    CodeFragment(CodeFragment),
    CodeExpression(CodeExpression),
}

#[derive(Debug)]
/// All type schemas that are derived from Code

pub enum CodeTypes {
    Code(Code),
    CodeBlock(CodeBlock),
    CodeChunk(CodeChunk),
    CodeExpression(CodeExpression),
    CodeFragment(CodeFragment),
}

#[derive(Debug)]
/// All type schemas that are derived from ContactPoint

pub enum ContactPointTypes {
    ContactPoint(ContactPoint),
    PostalAddress(PostalAddress),
}

#[derive(Debug)]
/// All type schemas that are derived from CreativeWork

pub enum CreativeWorkTypes {
    CreativeWork(CreativeWork),
    Article(Article),
    AudioObject(AudioObject),
    Collection(Collection),
    Comment(Comment),
    Datatable(Datatable),
    Figure(Figure),
    ImageObject(ImageObject),
    MediaObject(MediaObject),
    Periodical(Periodical),
    PublicationIssue(PublicationIssue),
    PublicationVolume(PublicationVolume),
    Review(Review),
    SoftwareApplication(SoftwareApplication),
    SoftwareSourceCode(SoftwareSourceCode),
    Table(Table),
    VideoObject(VideoObject),
}

#[derive(Debug)]
/// All type schemas that are derived from Entity

pub enum EntityTypes {
    Entity(Entity),
    ArrayValidator(ArrayValidator),
    Article(Article),
    AudioObject(AudioObject),
    BooleanValidator(BooleanValidator),
    Brand(Brand),
    Cite(Cite),
    CiteGroup(CiteGroup),
    Code(Code),
    CodeBlock(CodeBlock),
    CodeChunk(CodeChunk),
    CodeError(CodeError),
    CodeExpression(CodeExpression),
    CodeFragment(CodeFragment),
    Collection(Collection),
    Comment(Comment),
    ConstantValidator(ConstantValidator),
    ContactPoint(ContactPoint),
    CreativeWork(CreativeWork),
    Datatable(Datatable),
    DatatableColumn(DatatableColumn),
    Date(Date),
    DefinedTerm(DefinedTerm),
    Delete(Delete),
    Emphasis(Emphasis),
    EnumValidator(EnumValidator),
    Figure(Figure),
    Function(Function),
    Grant(Grant),
    Heading(Heading),
    ImageObject(ImageObject),
    Include(Include),
    IntegerValidator(IntegerValidator),
    Link(Link),
    List(List),
    ListItem(ListItem),
    Mark(Mark),
    Math(Math),
    MathBlock(MathBlock),
    MathFragment(MathFragment),
    MediaObject(MediaObject),
    MonetaryGrant(MonetaryGrant),
    NontextualAnnotation(NontextualAnnotation),
    NumberValidator(NumberValidator),
    Organization(Organization),
    Paragraph(Paragraph),
    Parameter(Parameter),
    Periodical(Periodical),
    Person(Person),
    PostalAddress(PostalAddress),
    Product(Product),
    PropertyValue(PropertyValue),
    PublicationIssue(PublicationIssue),
    PublicationVolume(PublicationVolume),
    Quote(Quote),
    QuoteBlock(QuoteBlock),
    Review(Review),
    SoftwareApplication(SoftwareApplication),
    SoftwareEnvironment(SoftwareEnvironment),
    SoftwareSession(SoftwareSession),
    SoftwareSourceCode(SoftwareSourceCode),
    StringValidator(StringValidator),
    Strong(Strong),
    Subscript(Subscript),
    Superscript(Superscript),
    Table(Table),
    TableCell(TableCell),
    TableRow(TableRow),
    ThematicBreak(ThematicBreak),
    Thing(Thing),
    TupleValidator(TupleValidator),
    Variable(Variable),
    VideoObject(VideoObject),
    VolumeMount(VolumeMount),
}

#[derive(Debug)]
/// All type schemas that are derived from Grant

pub enum GrantTypes {
    Grant(Grant),
    MonetaryGrant(MonetaryGrant),
}

#[derive(Debug)]
/// Union type for valid inline content.

pub enum InlineContent {
    CodeFragment(CodeFragment),
    CodeExpression(CodeExpression),
    Delete(Delete),
    Emphasis(Emphasis),
    ImageObject(ImageObject),
    Link(Link),
    MathFragment(MathFragment),
    NontextualAnnotation(NontextualAnnotation),
    Quote(Quote),
    Strong(Strong),
    Subscript(Subscript),
    Superscript(Superscript),
    Cite(Cite),
    CiteGroup(CiteGroup),
    Integer(Integer),
    Number(Number),
    Bool(Bool),
    Null(Null),
    String(String),
}

#[derive(Debug)]
/// All type schemas that are derived from Mark

pub enum MarkTypes {
    Mark(Mark),
    Delete(Delete),
    Emphasis(Emphasis),
    NontextualAnnotation(NontextualAnnotation),
    Quote(Quote),
    Strong(Strong),
    Subscript(Subscript),
    Superscript(Superscript),
}

#[derive(Debug)]
/// All type schemas that are derived from Math

pub enum MathTypes {
    Math(Math),
    MathBlock(MathBlock),
    MathFragment(MathFragment),
}

#[derive(Debug)]
/// All type schemas that are derived from MediaObject

pub enum MediaObjectTypes {
    MediaObject(MediaObject),
    AudioObject(AudioObject),
    ImageObject(ImageObject),
    VideoObject(VideoObject),
}

#[derive(Debug)]
/// Union type for all valid nodes.

pub enum Node {
    Entity(Entity),
    Integer(Integer),
    Number(Number),
    Bool(Bool),
    Null(Null),
    String(String),
    Array(Array),
    Object(Object),
}

#[derive(Debug)]
/// All type schemas that are derived from NumberValidator

pub enum NumberValidatorTypes {
    NumberValidator(NumberValidator),
    IntegerValidator(IntegerValidator),
}

#[derive(Debug)]
/// All type schemas that are derived from Thing

pub enum ThingTypes {
    Thing(Thing),
    Article(Article),
    AudioObject(AudioObject),
    Brand(Brand),
    Collection(Collection),
    Comment(Comment),
    ContactPoint(ContactPoint),
    CreativeWork(CreativeWork),
    Datatable(Datatable),
    DatatableColumn(DatatableColumn),
    DefinedTerm(DefinedTerm),
    Figure(Figure),
    Grant(Grant),
    ImageObject(ImageObject),
    ListItem(ListItem),
    MediaObject(MediaObject),
    MonetaryGrant(MonetaryGrant),
    Organization(Organization),
    Periodical(Periodical),
    Person(Person),
    PostalAddress(PostalAddress),
    Product(Product),
    PropertyValue(PropertyValue),
    PublicationIssue(PublicationIssue),
    PublicationVolume(PublicationVolume),
    Review(Review),
    SoftwareApplication(SoftwareApplication),
    SoftwareEnvironment(SoftwareEnvironment),
    SoftwareSession(SoftwareSession),
    SoftwareSourceCode(SoftwareSourceCode),
    Table(Table),
    VideoObject(VideoObject),
    VolumeMount(VolumeMount),
}

#[derive(Debug)]
/// Union type for all validator types.

pub enum ValidatorTypes {
    ConstantValidator(ConstantValidator),
    EnumValidator(EnumValidator),
    BooleanValidator(BooleanValidator),
    NumberValidator(NumberValidator),
    IntegerValidator(IntegerValidator),
    StringValidator(StringValidator),
    ArrayValidator(ArrayValidator),
    TupleValidator(TupleValidator),
}

#[derive(Debug)]
/// All type schemas that are derived from Variable

pub enum VariableTypes {
    Variable(Variable),
    Parameter(Parameter),
}
