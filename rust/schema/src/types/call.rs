// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::call_argument::CallArgument;
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
use super::string::String;
use super::timestamp::Timestamp;

/// Call another document, optionally with arguments, and include its executed content.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Call {
    /// The type of this item
    pub r#type: MustBe!("Call"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The external source of the content, a file path or URL.
    pub source: String,

    /// Media type of the source content.
    pub media_type: Option<String>,

    /// A query to select a subset of content from the source
    pub select: Option<String>,

    /// The structured content decoded from the source.
    pub content: Option<Vec<Block>>,

    /// The value of the source document's parameters to call it with
    pub arguments: Vec<CallArgument>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<CallOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CallOptions {
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
}

impl Call {
    pub fn new(source: String, arguments: Vec<CallArgument>) -> Self {
        Self {
            source,
            arguments,
            ..Default::default()
        }
    }
}
