// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::automatic_execution::AutomaticExecution;
use super::block::Block;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::cord::Cord;
use super::duration::Duration;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_message::ExecutionMessage;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::integer::Integer;
use super::section::Section;
use super::string::String;
use super::timestamp::Timestamp;

/// Repeat a block content for each item in an array.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "ForBlock")]
#[patch(authors_on = "options")]
pub struct ForBlock {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("ForBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Under which circumstances the code should be automatically executed.
    #[serde(alias = "auto", alias = "auto-exec", alias = "auto_exec")]
    #[strip(execution)]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub auto_exec: Option<AutomaticExecution>,

    /// The code.
    #[strip(code)]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("code")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    #[jats(content)]
    pub code: Cord,

    /// The programming language of the code.
    #[serde(alias = "programming-language", alias = "programming_language")]
    #[strip(code)]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Some(String::from("lang"))"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(r"(cpp)|(js)|(py)|(r)|(ts)")"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(r"[a-zA-Z0-9]{1,10}")"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(String::arbitrary())"#))]
    #[jats(attr = "language")]
    pub programming_language: Option<String>,

    /// The name to give to the variable representing each item in the iterated array
    #[strip(code)]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("item")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"[a-zA-Z_][a-zA-Z0-9]{0,9}"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[^\p{C}]{1,100}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub variable: String,

    /// The content to repeat for each item
    #[serde(deserialize_with = "one_or_many")]
    #[strip(code)]
    #[walk]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![p([t("For content")])]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_blocks_non_recursive(4)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks_non_recursive(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks_non_recursive(8)"#))]
    #[dom(elem = "div")]
    pub content: Vec<Block>,

    /// The content to render if there are no items
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(code)]
    #[walk]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(vec_blocks_non_recursive(2))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(vec_blocks_non_recursive(2))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(vec_blocks_non_recursive(4))"#))]
    #[dom(elem = "div")]
    pub otherwise: Option<Vec<Block>>,

    /// The content repeated for each iteration
    #[serde(alias = "iteration")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(output)]
    #[walk]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "div")]
    pub iterations: Option<Vec<Section>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ForBlockOptions>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct ForBlockOptions {
    /// A digest of the content, semantics and dependencies of the node.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(skip)]
    pub compilation_digest: Option<CompilationDigest>,

    /// Messages generated while compiling the code.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// The `compilationDigest` of the node when it was last executed.
    #[serde(alias = "execution-digest", alias = "execution_digest")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(skip)]
    pub execution_digest: Option<CompilationDigest>,

    /// The upstream dependencies of this node.
    #[serde(alias = "execution-dependencies", alias = "execution_dependencies", alias = "executionDependency", alias = "execution-dependency", alias = "execution_dependency")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
    pub execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    #[serde(alias = "execution-dependants", alias = "execution_dependants", alias = "executionDependant", alias = "execution-dependant", alias = "execution_dependant")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
    pub execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution.
    #[serde(alias = "execution-tags", alias = "execution_tags", alias = "executionTag", alias = "execution-tag", alias = "execution_tag")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
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
    #[dom(with = "Timestamp::to_dom_attr")]
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    #[serde(alias = "execution-duration", alias = "execution_duration")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(with = "Duration::to_dom_attr")]
    pub execution_duration: Option<Duration>,

    /// Messages emitted while executing the node.
    #[serde(alias = "execution-messages", alias = "execution_messages", alias = "executionMessage", alias = "execution-message", alias = "execution_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
    pub execution_messages: Option<Vec<ExecutionMessage>>,

    /// The authors of the executable code.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
    pub authors: Option<Vec<Author>>,
}

impl ForBlock {
    const NICK: [u8; 3] = [102, 114, 98];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ForBlock
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(code: Cord, variable: String, content: Vec<Block>) -> Self {
        Self {
            code,
            variable,
            content,
            ..Default::default()
        }
    }
}
