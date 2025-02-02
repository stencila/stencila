// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::duration::Duration;
use super::execution_bounds::ExecutionBounds;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_message::ExecutionMessage;
use super::execution_mode::ExecutionMode;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::instruction_type::InstructionType;
use super::integer::Integer;
use super::relative_position::RelativePosition;
use super::string::String;
use super::timestamp::Timestamp;

/// A preview of how a prompt will be rendered at a location in the document
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("PromptBlock")]
#[patch(apply_with = "PromptBlock::apply_patch_op")]
pub struct PromptBlock {
    /// The type of this item.
    pub r#type: MustBe!("PromptBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Under which circumstances the node should be executed.
    #[serde(alias = "execution-mode", alias = "execution_mode")]
    #[strip(execution)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    pub execution_mode: Option<ExecutionMode>,

    /// Under which circumstances child nodes should be executed.
    #[serde(alias = "execution-bounds", alias = "execution_bounds")]
    #[strip(execution)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex")]
    pub execution_bounds: Option<ExecutionBounds>,

    /// The type of instruction the  being used for
    #[serde(alias = "instruction-type", alias = "instruction_type")]
    #[patch(format = "md", format = "smd", format = "qmd")]
    pub instruction_type: Option<InstructionType>,

    /// The type of nodes the prompt is being used for
    #[serde(alias = "node-types", alias = "node_types", alias = "nodeType", alias = "node-type", alias = "node_type")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[patch(format = "md", format = "smd", format = "qmd")]
    pub node_types: Option<Vec<String>>,

    /// The relative position of the node being edited, described etc.
    #[serde(alias = "relative-position", alias = "relative_position")]
    #[patch(format = "md", format = "smd", format = "qmd")]
    pub relative_position: Option<RelativePosition>,

    /// A user text query used to infer the `target` prompt
    #[patch(format = "md", format = "smd", format = "qmd")]
    pub query: Option<String>,

    /// An identifier for the prompt to be rendered
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub target: Option<String>,

    /// The executed content of the prompt
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(output)]
    #[walk]
    #[patch()]
    #[dom(elem = "div")]
    pub content: Option<Vec<Block>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<PromptBlockOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PromptBlockOptions {
    /// A digest of the content, semantics and dependencies of the node.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[strip(compilation)]
    #[dom(skip)]
    pub compilation_digest: Option<CompilationDigest>,

    /// Messages generated while compiling the code.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(compilation)]
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

    /// The home directory of the prompt
    #[strip(compilation)]
    #[patch()]
    #[dom(skip)]
    pub directory: Option<String>,
}

impl PromptBlock {
    const NICK: [u8; 3] = [112, 114, 98];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::PromptBlock
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
