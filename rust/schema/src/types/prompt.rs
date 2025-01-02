// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::block::Block;
use super::comment::Comment;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_type_or_text::CreativeWorkTypeOrText;
use super::date::Date;
use super::duration::Duration;
use super::execution_bounds::ExecutionBounds;
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
use super::instruction_type::InstructionType;
use super::integer::Integer;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::provenance_count::ProvenanceCount;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::text::Text;
use super::thing_type::ThingType;
use super::timestamp::Timestamp;
use super::unsigned_integer_or_string::UnsignedIntegerOrString;

/// A prompt for creating or editing document content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Prompt")]
#[patch(authors_on = "options")]
pub struct Prompt {
    /// The type of this item.
    pub r#type: MustBe!("Prompt"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A description of the item.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub description: String,

    /// The name of the item.
    #[strip(metadata)]
    pub name: String,

    /// The version of the creative work.
    #[strip(metadata)]
    pub version: StringOrNumber,

    /// Under which circumstances the node should be executed.
    #[serde(alias = "execution-mode", alias = "execution_mode")]
    #[strip(execution)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub execution_mode: Option<ExecutionMode>,

    /// Under which circumstances child nodes should be executed.
    #[serde(alias = "execution-bounds", alias = "execution_bounds")]
    #[strip(execution)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub execution_bounds: Option<ExecutionBounds>,

    /// The types of instructions that the prompt supports
    #[serde(alias = "instruction-types", alias = "instruction_types", alias = "instructionType", alias = "instruction-type", alias = "instruction_type")]
    #[serde(deserialize_with = "one_or_many")]
    pub instruction_types: Vec<InstructionType>,

    /// Regular expressions used to match the prompt with a user instruction
    #[serde(alias = "instruction-patterns", alias = "instruction_patterns", alias = "instructionPattern", alias = "instruction-pattern", alias = "instruction_pattern")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub instruction_patterns: Option<Vec<String>>,

    /// The types of nodes that the prompt supports
    #[serde(alias = "node-types", alias = "node_types", alias = "nodeType", alias = "node-type", alias = "node_type")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub node_types: Option<Vec<String>>,

    /// The number of nodes that the prompt supports
    #[serde(alias = "node-count", alias = "node_count")]
    pub node_count: Option<UnsignedIntegerOrString>,

    /// The content of the prompt.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[dom(elem = "section")]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<PromptOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PromptOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

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

    /// The URL of the item.
    #[strip(metadata)]
    pub url: Option<String>,

    /// The subject matter of the content.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub about: Option<Vec<ThingType>>,

    /// A a short description that summarizes a `CreativeWork`.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub r#abstract: Option<Vec<Block>>,

    /// The authors of the `CreativeWork`.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[dom(elem = "section")]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the content within the work.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[dom(elem = "div")]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// A secondary contributor to the `CreativeWork`.
    #[serde(alias = "contributor")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub contributors: Option<Vec<Author>>,

    /// People who edited the `CreativeWork`.
    #[serde(alias = "editor")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub editors: Option<Vec<Person>>,

    /// The maintainers of the `CreativeWork`.
    #[serde(alias = "maintainer")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub maintainers: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    #[serde(alias = "comment")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub comments: Option<Vec<Comment>>,

    /// Date/time of creation.
    #[serde(alias = "date-created", alias = "date_created")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    #[serde(alias = "date-received", alias = "date_received")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    #[serde(alias = "date-accepted", alias = "date_accepted")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    #[serde(alias = "date-modified", alias = "date_modified")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    pub date_modified: Option<Date>,

    /// Date of first publication.
    #[serde(alias = "date", alias = "date-published", alias = "date_published")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    pub date_published: Option<Date>,

    /// People or organizations that funded the `CreativeWork`.
    #[serde(alias = "funder")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    #[serde(alias = "funded-by", alias = "funded_by")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub genre: Option<Vec<String>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    #[serde(alias = "keyword")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub keywords: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    #[serde(alias = "is-part-of", alias = "is_part_of")]
    #[strip(metadata)]
    pub is_part_of: Option<CreativeWorkType>,

    /// License documents that applies to this content, typically indicated by URL.
    #[serde(alias = "license")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub licenses: Option<Vec<CreativeWorkTypeOrText>>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    #[serde(alias = "hasParts", alias = "part")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(content)]
    #[dom(elem = "section")]
    pub parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    #[serde(alias = "citations", alias = "reference")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub references: Option<Vec<CreativeWorkTypeOrText>>,

    /// The textual content of this creative work.
    #[strip(content)]
    pub text: Option<Text>,

    /// The title of the creative work.
    #[serde(alias = "headline")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[dom(elem = "h1")]
    pub title: Option<Vec<Inline>>,

    /// A digest of the content, semantics and dependencies of the node.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[strip(execution)]
    #[dom(skip)]
    pub compilation_digest: Option<CompilationDigest>,

    /// Messages generated while compiling the code.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[dom(elem = "span")]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// The `compilationDigest` of the node when it was last executed.
    #[serde(alias = "execution-digest", alias = "execution_digest")]
    #[strip(execution)]
    #[dom(skip)]
    pub execution_digest: Option<CompilationDigest>,

    /// The upstream dependencies of this node.
    #[serde(alias = "execution-dependencies", alias = "execution_dependencies", alias = "executionDependency", alias = "execution-dependency", alias = "execution_dependency")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[dom(elem = "span")]
    pub execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    #[serde(alias = "execution-dependants", alias = "execution_dependants", alias = "executionDependant", alias = "execution-dependant", alias = "execution_dependant")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[dom(elem = "span")]
    pub execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution.
    #[serde(alias = "execution-tags", alias = "execution_tags", alias = "executionTag", alias = "execution-tag", alias = "execution_tag")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[dom(elem = "span")]
    pub execution_tags: Option<Vec<ExecutionTag>>,

    /// A count of the number of times that the node has been executed.
    #[serde(alias = "execution-count", alias = "execution_count")]
    #[strip(execution)]
    pub execution_count: Option<Integer>,

    /// Whether, and why, the code requires execution or re-execution.
    #[serde(alias = "execution-required", alias = "execution_required")]
    #[strip(execution)]
    pub execution_required: Option<ExecutionRequired>,

    /// Status of the most recent, including any current, execution.
    #[serde(alias = "execution-status", alias = "execution_status")]
    #[strip(execution)]
    pub execution_status: Option<ExecutionStatus>,

    /// The id of the kernel instance that performed the last execution.
    #[serde(alias = "execution-instance", alias = "execution_instance")]
    #[strip(execution)]
    pub execution_instance: Option<String>,

    /// The bounds, if any, on the last execution.
    #[serde(alias = "execution-bounded", alias = "execution_bounded")]
    #[strip(execution)]
    pub execution_bounded: Option<ExecutionBounds>,

    /// The timestamp when the last execution ended.
    #[serde(alias = "execution-ended", alias = "execution_ended")]
    #[strip(execution, timestamps)]
    #[dom(with = "Timestamp::to_dom_attr")]
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    #[serde(alias = "execution-duration", alias = "execution_duration")]
    #[strip(execution)]
    #[dom(with = "Duration::to_dom_attr")]
    pub execution_duration: Option<Duration>,

    /// Messages emitted while executing the node.
    #[serde(alias = "execution-messages", alias = "execution_messages", alias = "executionMessage", alias = "execution-message", alias = "execution_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[dom(elem = "span")]
    pub execution_messages: Option<Vec<ExecutionMessage>>,
}

impl Prompt {
    const NICK: [u8; 3] = [112, 114, 111];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Prompt
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
}
