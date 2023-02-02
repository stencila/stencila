//! Generated file, do not edit

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

/// The value of a `Parameter` to call a document with
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CallArgument {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("CallArgument"),

    /// The identifier for this item
    id: Option<String>,

    /// Under which circumstances the code should be automatically executed.
    execution_auto: ExecutionAuto,

    /// A count of the number of times that the node has been executed.
    execution_count: Integer,

    /// Whether, and why, the code requires execution or re-execution.
    execution_required: ExecutionRequired,

    /// Status of the most recent, including any current, execution.
    execution_status: ExecutionStatus,

    /// The name of the parameter.
    name: String,

    /// A short label for the parameter.
    label: Option<String>,

    /// The current value of the parameter.
    value: Option<Box<Node>>,

    /// The default value of the parameter.
    default: Option<Box<Node>>,

    /// The validator that the value is validated against.
    validator: Option<Validator>,

    /// The code to be evaluated for the parameter.
    code: String,

    /// The programming language of the code.
    programming_language: String,

    /// Whether the programming language of the code should be guessed based on syntax and variables used
    guess_language: Option<Boolean>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<CallArgumentOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CallArgumentOptions {
    /// A digest of the content, semantics and dependencies of the node.
    compile_digest: Option<ExecutionDigest>,

    /// The `compileDigest` of the node when it was last executed.
    execute_digest: Option<ExecutionDigest>,

    /// The upstream dependencies of this node.
    execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution
    execution_tags: Option<Vec<ExecutionTag>>,

    /// The id of the kernel that the node was last executed in.
    execution_kernel: Option<String>,

    /// The timestamp when the last execution ended.
    execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    execution_duration: Option<Duration>,

    /// Errors when compiling (e.g. syntax errors) or executing the node.
    errors: Option<Vec<CodeError>>,

    /// Whether the parameter should be hidden.
    hidden: Option<Boolean>,

    /// The dotted path to the object (e.g. a database table column) that the parameter should be derived from
    derived_from: Option<String>,
}
