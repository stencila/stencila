// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::automatic_execution::AutomaticExecution;
use super::boolean::Boolean;
use super::compilation_digest::CompilationDigest;
use super::compilation_error::CompilationError;
use super::cord::Cord;
use super::duration::Duration;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_error::ExecutionError;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::integer::Integer;
use super::node::Node;
use super::string::String;
use super::timestamp::Timestamp;
use super::validator::Validator;

/// The value of a `Parameter` to call a document with.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "CallArgument")]
pub struct CallArgument {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("CallArgument"),

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

    /// The name of the parameter.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("name")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"[a-zA-Z_][a-zA-Z0-9]{0,9}"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[^\p{C}]{1,100}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    #[jats(attr = "name")]
    pub name: String,

    /// A short label for the parameter.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub label: Option<String>,

    /// The current value of the parameter.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub value: Option<Box<Node>>,

    /// The default value of the parameter.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub default: Option<Box<Node>>,

    /// The validator that the value is validated against.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(content)]
    pub validator: Option<Validator>,

    /// The code to be evaluated for the parameter.
    #[strip(code)]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("code")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    pub code: Cord,

    /// The programming language of the code.
    #[serde(alias = "programming-language", alias = "programming_language")]
    #[strip(code)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub programming_language: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<CallArgumentOptions>,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct CallArgumentOptions {
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

    /// Errors when executing the node.
    #[serde(alias = "execution-errors", alias = "execution_errors", alias = "executionError", alias = "execution-error", alias = "execution_error")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_errors: Option<Vec<ExecutionError>>,

    /// Whether the parameter should be hidden.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub hidden: Option<Boolean>,

    /// The dotted path to the object (e.g. a database table column) that the parameter should be derived from
    #[serde(alias = "derived-from", alias = "derived_from")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub derived_from: Option<String>,
}

impl CallArgument {
    pub fn new(name: String, code: Cord) -> Self {
        Self {
            name,
            code,
            ..Default::default()
        }
    }
}

impl Entity for CallArgument {
    const NICK: &'static str = "cal";

    fn node_type(&self) -> NodeType {
        NodeType::CallArgument
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
