// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::automatic_execution::AutomaticExecution;
use super::block::Block;
use super::compilation_digest::CompilationDigest;
use super::compilation_error::CompilationError;
use super::duration::Duration;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_message::ExecutionMessage;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::integer::Integer;
use super::message::Message;
use super::string::String;
use super::suggestion_block_type::SuggestionBlockType;
use super::timestamp::Timestamp;

/// An instruction to edit some block content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "InstructionBlock")]
pub struct InstructionBlock {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("InstructionBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Under which circumstances the code should be automatically executed.
    #[serde(alias = "auto", alias = "auto-exec", alias = "auto_exec")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub auto_exec: Option<AutomaticExecution>,

    /// Messages involved in the instruction.
    #[serde(alias = "message")]
    #[serde(deserialize_with = "one_or_many")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[dom(elem = "div")]
    pub messages: Vec<Message>,

    /// The content to which the instruction applies.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(vec_blocks_non_recursive(1))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(vec_blocks_non_recursive(2))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(vec_blocks_non_recursive(4))"#))]
    #[dom(elem = "div")]
    pub content: Option<Vec<Block>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<InstructionBlockOptions>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct InstructionBlockOptions {
    /// A digest of the content, semantics and dependencies of the node.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_digest: Option<CompilationDigest>,

    /// Errors generated when compiling the code.
    #[serde(alias = "compilation-errors", alias = "compilation_errors", alias = "compilationError", alias = "compilation-error", alias = "compilation_error")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_errors: Option<Vec<CompilationError>>,

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

    /// The id of the actor that the node was last executed by.
    #[serde(alias = "execution-actor", alias = "execution_actor")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_actor: Option<String>,

    /// The timestamp when the last execution ended.
    #[serde(alias = "execution-ended", alias = "execution_ended")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    #[serde(alias = "execution-duration", alias = "execution_duration")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_duration: Option<Duration>,

    /// Messages emitted when executing the node.
    #[serde(alias = "execution-messages", alias = "execution_messages", alias = "executionMessage", alias = "execution-message", alias = "execution_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_messages: Option<Vec<ExecutionMessage>>,

    /// A list of candidates for the assignee property.
    #[serde(alias = "candidate")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(vec(r"[a-zA-Z][a-zA-Z\-_/.@]", size_range(1..5)))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(vec(String::arbitrary(), size_range(1..10)))"#))]
    pub candidates: Option<Vec<String>>,

    /// An identifier for the agent assigned to perform the instruction
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(r"[a-zA-Z][a-zA-Z\-_/.@]")"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(String::arbitrary())"#))]
    pub assignee: Option<String>,

    /// The authors of the instruction.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "div")]
    pub authors: Option<Vec<Author>>,

    /// A suggestion for the instruction
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "div")]
    pub suggestion: Option<SuggestionBlockType>,
}

impl InstructionBlock {
    const NICK: &'static str = "isb";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::InstructionBlock
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }
}
