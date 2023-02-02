//! Generated file, do not edit

use crate::prelude::*;

use super::code_error::CodeError;
use super::duration::Duration;
use super::execution_auto::ExecutionAuto;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_digest::ExecutionDigest;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::if_clause::IfClause;
use super::integer::Integer;
use super::string::String;
use super::timestamp::Timestamp;

/// Show and execute alternative content conditional upon an executed expression
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct If {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("If"),

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

    /// The clauses making up the `If` node
    clauses: Vec<IfClause>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<IfOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct IfOptions {
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
}
