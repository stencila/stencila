// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::bibliography::Bibliography;
use super::block::Block;
use super::comment::Comment;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_variant::CreativeWorkVariant;
use super::creative_work_variant_or_string::CreativeWorkVariantOrString;
use super::date::Date;
use super::duration::Duration;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_message::ExecutionMessage;
use super::execution_mode::ExecutionMode;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::grant_or_monetary_grant::GrantOrMonetaryGrant;
use super::image_object::ImageObject;
use super::inline::Inline;
use super::integer::Integer;
use super::integer_or_string::IntegerOrString;
use super::list::List;
use super::node::Node;
use super::object::Object;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::provenance_count::ProvenanceCount;
use super::reference::Reference;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::text::Text;
use super::thing_variant::ThingVariant;
use super::timestamp::Timestamp;

/// An article, including news and scholarly articles.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display("Article")]
#[patch(authors_on = "self")]
#[html(elem = "article")]
#[jats(elem = "article", special)]
pub struct Article {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Article"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The type of `CreativeWork` (e.g. article, book, software application).
    #[serde(alias = "work-type", alias = "work_type")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub work_type: Option<CreativeWorkType>,

    /// The work's Digital Object Identifier (https://doi.org/).
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub doi: Option<String>,

    /// A short description that summarizes a `CreativeWork`.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[walk]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub r#abstract: Option<Vec<Block>>,

    /// The authors of the `CreativeWork`.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the content within the work.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// Date of first publication.
    #[serde(alias = "date", alias = "date-published", alias = "date_published")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_published: Option<Date>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    #[serde(alias = "citations", alias = "reference")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub references: Option<Vec<Reference>>,

    /// The title of the creative work.
    #[serde(alias = "headline")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[walk]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub title: Option<Vec<Inline>>,

    /// Under which circumstances the node should be executed.
    #[serde(alias = "execution-mode", alias = "execution_mode")]
    #[strip(code)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_mode: Option<ExecutionMode>,

    /// Frontmatter containing document metadata.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub frontmatter: Option<String>,

    /// The content of the article.
    #[serde(deserialize_with = "one_or_many")]
    #[strip(content)]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_blocks(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_blocks(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks(8)"#))]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ArticleOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct ArticleOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub description: Option<String>,

    /// Any kind of identifier for any kind of Thing.
    #[serde(alias = "identifier")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[serde(alias = "image")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub url: Option<String>,

    /// The subject matter of the content.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub about: Option<Vec<ThingVariant>>,

    /// A secondary contributor to the `CreativeWork`.
    #[serde(alias = "contributor")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub contributors: Option<Vec<Author>>,

    /// People who edited the `CreativeWork`.
    #[serde(alias = "editor")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub editors: Option<Vec<Person>>,

    /// The maintainers of the `CreativeWork`.
    #[serde(alias = "maintainer")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub maintainers: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    #[serde(alias = "comment")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub comments: Option<Vec<Comment>>,

    /// Date/time of creation.
    #[serde(alias = "date-created", alias = "date_created")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    #[serde(alias = "date-received", alias = "date_received")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    #[serde(alias = "date-accepted", alias = "date_accepted")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    #[serde(alias = "date-modified", alias = "date_modified")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_modified: Option<Date>,

    /// People or organizations that funded the `CreativeWork`.
    #[serde(alias = "funder")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    #[serde(alias = "funded-by", alias = "funded_by")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub genre: Option<Vec<String>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    #[serde(alias = "keyword")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub keywords: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    #[serde(alias = "is-part-of", alias = "is_part_of")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_part_of: Option<CreativeWorkVariant>,

    /// License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.
    #[serde(alias = "license")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub licenses: Option<Vec<CreativeWorkVariantOrString>>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    #[serde(alias = "hasParts", alias = "part")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(content)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub parts: Option<Vec<CreativeWorkVariant>>,

    /// A publisher of the CreativeWork.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub publisher: Option<PersonOrOrganization>,

    /// A bibliography of references which may be cited in the work.
    #[strip(output)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub bibliography: Option<Bibliography>,

    /// The textual content of this creative work.
    #[strip(content)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub text: Option<Text>,

    /// URL of the repository where the un-compiled, human readable source of the work is located.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub repository: Option<String>,

    /// The file system path of the source of the work.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub path: Option<String>,

    /// The commit hash (or similar) of the source of the work.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub commit: Option<String>,

    /// The version of the creative work.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub version: Option<StringOrNumber>,

    /// A digest of the content, semantics and dependencies of the node.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[strip(compilation)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_digest: Option<CompilationDigest>,

    /// Messages generated while compiling the code.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(compilation)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// The `compilationDigest` of the node when it was last executed.
    #[serde(alias = "execution-digest", alias = "execution_digest")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_digest: Option<CompilationDigest>,

    /// The upstream dependencies of this node.
    #[serde(alias = "execution-dependencies", alias = "execution_dependencies", alias = "executionDependency", alias = "execution-dependency", alias = "execution_dependency")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    #[serde(alias = "execution-dependants", alias = "execution_dependants", alias = "executionDependant", alias = "execution-dependant", alias = "execution_dependant")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution.
    #[serde(alias = "execution-tags", alias = "execution_tags", alias = "executionTag", alias = "execution-tag", alias = "execution_tag")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_tags: Option<Vec<ExecutionTag>>,

    /// A count of the number of times that the node has been executed.
    #[serde(alias = "execution-count", alias = "execution_count")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_count: Option<Integer>,

    /// Whether, and why, the code requires execution or re-execution.
    #[serde(alias = "execution-required", alias = "execution_required")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_required: Option<ExecutionRequired>,

    /// Status of the most recent, including any current, execution.
    #[serde(alias = "execution-status", alias = "execution_status")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_status: Option<ExecutionStatus>,

    /// The id of the kernel instance that performed the last execution.
    #[serde(alias = "execution-instance", alias = "execution_instance")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_instance: Option<String>,

    /// The timestamp when the last execution ended.
    #[serde(alias = "execution-ended", alias = "execution_ended")]
    #[strip(execution, timestamps)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    #[serde(alias = "execution-duration", alias = "execution_duration")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_duration: Option<Duration>,

    /// Messages emitted while executing the node.
    #[serde(alias = "execution-messages", alias = "execution_messages", alias = "executionMessage", alias = "execution-message", alias = "execution_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_messages: Option<Vec<ExecutionMessage>>,

    /// The page on which the article starts; for example "135" or "xiii".
    #[serde(alias = "page-start", alias = "page_start")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub page_start: Option<IntegerOrString>,

    /// The page on which the article ends; for example "138" or "xvi".
    #[serde(alias = "page-end", alias = "page_end")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub pagination: Option<String>,

    /// A list of links to headings, including implied section headings, within the document
    #[strip(content, temporary)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub headings: Option<List>,

    /// Nodes, usually from within `content` of the article, that have been archived.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(content, archive)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub archive: Option<Vec<Node>>,

    /// Additional metadata for the article.
    #[serde(flatten, deserialize_with = "empty_object_is_none")]
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "docx")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub extra: Option<Object>,
}

impl Article {
    const NICK: [u8; 3] = *b"art";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Article
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
