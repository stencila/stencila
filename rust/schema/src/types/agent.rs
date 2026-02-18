// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::bibliography::Bibliography;
use super::block::Block;
use super::boolean::Boolean;
use super::comment::Comment;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_variant::CreativeWorkVariant;
use super::creative_work_variant_or_string::CreativeWorkVariantOrString;
use super::date::Date;
use super::grant_or_monetary_grant::GrantOrMonetaryGrant;
use super::image_object::ImageObject;
use super::inline::Inline;
use super::integer::Integer;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::provenance_count::ProvenanceCount;
use super::reference::Reference;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::text::Text;
use super::thing_variant::ThingVariant;

/// An agent definition specifying model, tools, and behavioral configuration.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Agent")]
#[patch(authors_on = "options")]
pub struct Agent {
    /// The type of this item.
    pub r#type: MustBe!("Agent"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// A description of the item.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub description: String,

    /// The name of the agent.
    #[patch(format = "md", format = "myst", format = "qmd", format = "smd")]
    pub name: String,

    /// The type of `CreativeWork` (e.g. article, book, software application).
    #[serde(alias = "work-type", alias = "work_type")]
    pub work_type: Option<CreativeWorkType>,

    /// The work's Digital Object Identifier (https://doi.org/).
    pub doi: Option<String>,

    /// Frontmatter containing agent metadata.
    #[strip(metadata)]
    #[patch(format = "md", format = "myst", format = "qmd", format = "smd")]
    pub frontmatter: Option<String>,

    /// The content of the agent (the Markdown body providing system instructions).
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[dom(elem = "section")]
    pub content: Option<Vec<Block>>,

    /// Model identifier for the agent.
    pub model: Option<String>,

    /// Provider identifier for the agent.
    pub provider: Option<String>,

    /// Reasoning effort level for the agent.
    #[serde(alias = "reasoning-effort", alias = "reasoning_effort")]
    pub reasoning_effort: Option<String>,

    /// Skill names this agent can use.
    #[serde(alias = "allowed-skills", alias = "allowed_skills", alias = "allowedSkill", alias = "allowed-skill", alias = "allowed_skill")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub allowed_skills: Option<Vec<String>>,

    /// Tool names available to the agent.
    #[serde(alias = "allowed-tools", alias = "allowed_tools", alias = "allowedTool", alias = "allowed-tool", alias = "allowed_tool")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub allowed_tools: Option<Vec<String>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<AgentOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct AgentOptions {
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
    pub about: Option<Vec<ThingVariant>>,

    /// A short description that summarizes a `CreativeWork`.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[walk]
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
    #[dom(with = "Date::to_dom_attr")]
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    #[serde(alias = "date-received", alias = "date_received")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[dom(with = "Date::to_dom_attr")]
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    #[serde(alias = "date-accepted", alias = "date_accepted")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[dom(with = "Date::to_dom_attr")]
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    #[serde(alias = "date-modified", alias = "date_modified")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[dom(with = "Date::to_dom_attr")]
    pub date_modified: Option<Date>,

    /// Date of first publication.
    #[serde(alias = "date", alias = "date-published", alias = "date_published")]
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[dom(with = "Date::to_dom_attr")]
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
    pub is_part_of: Option<CreativeWorkVariant>,

    /// License documents that applies to this content, typically indicated by URL, but may be a `CreativeWork` itself.
    #[serde(alias = "license")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub licenses: Option<Vec<CreativeWorkVariantOrString>>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    #[serde(alias = "hasParts", alias = "part")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(content)]
    #[dom(elem = "section")]
    pub parts: Option<Vec<CreativeWorkVariant>>,

    /// A publisher of the CreativeWork.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub publisher: Option<PersonOrOrganization>,

    /// A bibliography of references which may be cited in the work.
    #[strip(output)]
    pub bibliography: Option<Bibliography>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    #[serde(alias = "citations", alias = "reference")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[dom(elem = "section")]
    pub references: Option<Vec<Reference>>,

    /// The textual content of this creative work.
    #[strip(content)]
    pub text: Option<Text>,

    /// The title of the creative work.
    #[serde(alias = "headline")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[walk]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[dom(elem = "h1")]
    pub title: Option<Vec<Inline>>,

    /// URL of the repository where the un-compiled, human readable source of the work is located.
    pub repository: Option<String>,

    /// The file system path of the source of the work.
    #[strip(metadata)]
    pub path: Option<String>,

    /// The commit hash (or similar) of the source of the work.
    #[strip(metadata)]
    pub commit: Option<String>,

    /// The version of the creative work.
    #[strip(metadata)]
    pub version: Option<StringOrNumber>,

    /// Whether to enable MCP tools.
    #[serde(alias = "enable-mcp", alias = "enable_mcp")]
    pub enable_mcp: Option<Boolean>,

    /// Whether to enable MCP codemode orchestration.
    #[serde(alias = "enable-mcp-codemode", alias = "enable_mcp_codemode")]
    pub enable_mcp_codemode: Option<Boolean>,

    /// MCP server IDs this agent is allowed to use.
    #[serde(alias = "allowed-mcp-servers", alias = "allowed_mcp_servers", alias = "allowedMcpServer", alias = "allowed-mcp-server", alias = "allowed_mcp_server")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub allowed_mcp_servers: Option<Vec<String>>,

    /// Maximum conversation turns (0 = unlimited).
    #[serde(alias = "max-turns", alias = "max_turns")]
    pub max_turns: Option<Integer>,

    /// Default timeout for tool execution in seconds.
    #[serde(alias = "tool-timeout", alias = "tool_timeout")]
    pub tool_timeout: Option<Integer>,

    /// Maximum tool-call rounds per user input.
    #[serde(alias = "max-tool-rounds", alias = "max_tool_rounds")]
    pub max_tool_rounds: Option<Integer>,

    /// Maximum subagent nesting depth.
    #[serde(alias = "max-subagent-depth", alias = "max_subagent_depth")]
    pub max_subagent_depth: Option<Integer>,

    /// Environment requirements for the agent.
    pub compatibility: Option<String>,
}

impl Agent {
    const NICK: [u8; 3] = *b"agt";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Agent
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(description: String, name: String) -> Self {
        Self {
            description,
            name,
            ..Default::default()
        }
    }
}
