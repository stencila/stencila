// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::automatic_execution::AutomaticExecution;
use super::block::Block;
use super::boolean::Boolean;
use super::compilation_digest::CompilationDigest;
use super::compilation_error::CompilationError;
use super::cord::Cord;
use super::duration::Duration;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_message::ExecutionMessage;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::integer::Integer;
use super::string::String;
use super::timestamp::Timestamp;

/// A clause within an `IfBlock` node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "IfBlockClause")]
pub struct IfBlockClause {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("IfBlockClause"),

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

    /// The code.
    #[strip(code)]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("code")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    #[dom(elem = "pre")]
    #[jats(content)]
    pub code: Cord,

    /// The programming language of the code.
    #[serde(alias = "programming-language", alias = "programming_language")]
    #[strip(code)]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Some(String::from("lang"))"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(r"(cpp)|(js)|(py)|(r)|(ts)")"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(r"[a-zA-Z0-9]{1,10}")"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(String::arbitrary())"#))]
    #[jats(attr = "language")]
    pub programming_language: Option<String>,

    /// The content to render if the result is truthy
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![p([t("If clause content")])]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_blocks_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks_non_recursive(4)"#))]
    #[dom(elem = "div")]
    #[html(slot = "div")]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<IfBlockClauseOptions>,

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
pub struct IfBlockClauseOptions {
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

    /// The authors of the executable code.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
    pub authors: Option<Vec<Author>>,

    /// Whether this clause is the active clause in the parent `IfBlock` node
    #[serde(alias = "is-active", alias = "is_active")]
    #[strip(output)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_active: Option<Boolean>,
}

impl IfBlockClause {
    const NICK: &'static str = "ibc";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::IfBlockClause
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(code: Cord, content: Vec<Block>) -> Self {
        Self {
            code,
            content,
            ..Default::default()
        }
    }
}
