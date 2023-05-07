// Generated file. Do not edit; see `schema-gen` crate.

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
use super::parameter::Parameter;
use super::string::String;
use super::timestamp::Timestamp;
use super::validator::Validator;

/// The value of a `Parameter` to call a document with
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CallArgument {
    /// The type of this item
    pub r#type: MustBe!("CallArgument"),

    /// The identifier for this item
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
    pub validator: Option<Validator>,

    /// The code to be evaluated for the parameter.
    pub code: String,

    /// The programming language of the code.
    pub programming_language: String,

    /// Whether the programming language of the code should be guessed based on syntax and variables used
    pub guess_language: Option<Boolean>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<CallArgumentOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CallArgumentOptions {
    /// Under which circumstances the code should be automatically executed.
    pub execution_auto: Option<ExecutionAuto>,

    /// A digest of the content, semantics and dependencies of the node.
    pub compilation_digest: Option<ExecutionDigest>,

    /// The `compileDigest` of the node when it was last executed.
    pub execution_digest: Option<ExecutionDigest>,

    /// The upstream dependencies of this node.
    pub execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    pub execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution
    pub execution_tags: Option<Vec<ExecutionTag>>,

    /// A count of the number of times that the node has been executed.
    pub execution_count: Option<Integer>,

    /// Whether, and why, the code requires execution or re-execution.
    pub execution_required: Option<ExecutionRequired>,

    /// The id of the kernel that the node was last executed in.
    pub execution_kernel: Option<String>,

    /// Status of the most recent, including any current, execution.
    pub execution_status: Option<ExecutionStatus>,

    /// The timestamp when the last execution ended.
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    pub execution_duration: Option<Duration>,

    /// Errors when compiling (e.g. syntax errors) or executing the node.
    pub errors: Option<Vec<CodeError>>,

    /// Whether the parameter should be hidden.
    pub hidden: Option<Boolean>,

    /// The dotted path to the object (e.g. a database table column) that the parameter should be derived from
    pub derived_from: Option<String>,
}

impl CallArgument {
    pub fn new(name: String, code: String, programming_language: String) -> Self {
        Self {
            name,
            code,
            programming_language,
            ..Default::default()
        }
    }
}
impl_into!(CallArgument, Parameter);
impl_merge!(CallArgument, Parameter);
