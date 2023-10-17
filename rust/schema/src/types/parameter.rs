// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::code_error::CodeError;
use super::duration::Duration;
use super::execution_auto::ExecutionAuto;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_digest::ExecutionDigest;
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
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "span", custom)]
#[markdown(special)]
pub struct Parameter {
    /// The type of this item.
    pub r#type: MustBe!("Parameter"),

    /// The identifier for this item.
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the parameter.
    pub name: String,

    /// A short label for the parameter.
    pub label: Option<String>,

    /// The current value of the parameter.
    pub value: Option<Box<Node>>,

    /// The default value of the parameter.
    pub default: Option<Box<Node>>,

    /// The validator that the value is validated against.
    #[html(content)]
    pub validator: Option<Validator>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<ParameterOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ParameterOptions {
    /// Under which circumstances the code should be automatically executed.
    #[strip(execution)]
    pub execution_auto: Option<ExecutionAuto>,

    /// A digest of the content, semantics and dependencies of the node.
    #[strip(execution)]
    pub compilation_digest: Option<ExecutionDigest>,

    /// The `compileDigest` of the node when it was last executed.
    #[strip(execution)]
    pub execution_digest: Option<ExecutionDigest>,

    /// The upstream dependencies of this node.
    #[strip(execution)]
    pub execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    #[strip(execution)]
    pub execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution.
    #[strip(execution)]
    pub execution_tags: Option<Vec<ExecutionTag>>,

    /// A count of the number of times that the node has been executed.
    #[strip(execution)]
    pub execution_count: Option<Integer>,

    /// Whether, and why, the code requires execution or re-execution.
    #[strip(execution)]
    pub execution_required: Option<ExecutionRequired>,

    /// The id of the kernel that the node was last executed in.
    #[strip(execution)]
    pub execution_kernel: Option<String>,

    /// Status of the most recent, including any current, execution.
    #[strip(execution)]
    pub execution_status: Option<ExecutionStatus>,

    /// The timestamp when the last execution ended.
    #[strip(execution)]
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    #[strip(execution)]
    pub execution_duration: Option<Duration>,

    /// Errors when compiling (e.g. syntax errors) or executing the node.
    #[strip(execution)]
    pub errors: Option<Vec<CodeError>>,

    /// Whether the parameter should be hidden.
    pub hidden: Option<Boolean>,

    /// The dotted path to the object (e.g. a database table column) that the parameter should be derived from
    pub derived_from: Option<String>,
}

impl Parameter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}
