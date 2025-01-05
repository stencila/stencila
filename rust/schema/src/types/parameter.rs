// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

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
use super::integer::Integer;
use super::node::Node;
use super::string::String;
use super::timestamp::Timestamp;
use super::validator::Validator;

/// A parameter of a document.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Parameter")]
#[html(special)]
#[jats(elem = "parameter")]
pub struct Parameter {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Parameter"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Under which circumstances the node should be executed.
    #[serde(alias = "execution-mode", alias = "execution_mode")]
    #[strip(execution)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_mode: Option<ExecutionMode>,

    /// Under which circumstances child nodes should be executed.
    #[serde(alias = "execution-bounds", alias = "execution_bounds")]
    #[strip(execution)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_bounds: Option<ExecutionBounds>,

    /// The name of the parameter.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("name")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"[a-zA-Z_][a-zA-Z0-9]{0,9}"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[^\p{C}]{1,100}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    #[jats(attr = "name")]
    pub name: String,

    /// The current value of the parameter.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub value: Option<Box<Node>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ParameterOptions>,

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
pub struct ParameterOptions {
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

    /// The id of the kernel instance that performed the last execution.
    #[serde(alias = "execution-instance", alias = "execution_instance")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_instance: Option<String>,

    /// The bounds, if any, on the last execution.
    #[serde(alias = "execution-bounded", alias = "execution_bounded")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_bounded: Option<ExecutionBounds>,

    /// The timestamp when the last execution ended.
    #[serde(alias = "execution-ended", alias = "execution_ended")]
    #[strip(execution, timestamps)]
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

    /// A short label for the parameter.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub label: Option<String>,

    /// The default value of the parameter.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub default: Option<Box<Node>>,

    /// The validator that the value is validated against.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "span")]
    #[html(content)]
    pub validator: Option<Validator>,

    /// The dotted path to the object (e.g. a database table column) that the parameter should be derived from
    #[serde(alias = "derived-from", alias = "derived_from")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub derived_from: Option<String>,
}

impl Parameter {
    const NICK: [u8; 3] = [112, 97, 114];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Parameter
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
