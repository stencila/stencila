//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
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
use super::string::String;
use super::timestamp::Timestamp;

/// Styled block content
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Division {
    /// The type of this item
    pub r#type: MustBe!("Division"),

    /// The identifier for this item
    pub id: Option<String>,

    /// Under which circumstances the code should be automatically executed.
    pub execution_auto: ExecutionAuto,

    /// A count of the number of times that the node has been executed.
    pub execution_count: Integer,

    /// Whether, and why, the code requires execution or re-execution.
    pub execution_required: ExecutionRequired,

    /// Status of the most recent, including any current, execution.
    pub execution_status: ExecutionStatus,

    /// The code.
    pub code: String,

    /// The programming language of the code.
    pub programming_language: String,

    /// Whether the programming language of the code should be guessed based on syntax and variables used
    pub guess_language: Boolean,

    /// A Cascading Style Sheet (CSS) transpiled from the output of evaluating the `text` property.
    pub css: Option<String>,

    /// A list of class names associated with the document node
    pub classes: Option<Vec<String>>,

    /// The content within the division
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<DivisionOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct DivisionOptions {
    /// A digest of the content, semantics and dependencies of the node.
    pub compile_digest: Option<ExecutionDigest>,

    /// The `compileDigest` of the node when it was last executed.
    pub execute_digest: Option<ExecutionDigest>,

    /// The upstream dependencies of this node.
    pub execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    pub execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution
    pub execution_tags: Option<Vec<ExecutionTag>>,

    /// The id of the kernel that the node was last executed in.
    pub execution_kernel: Option<String>,

    /// The timestamp when the last execution ended.
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    pub execution_duration: Option<Duration>,

    /// Errors when compiling (e.g. syntax errors) or executing the node.
    pub errors: Option<Vec<CodeError>>,

    /// Media type, typically expressed using a MIME format, of the code.
    pub media_type: Option<String>,
}

impl Division {
    pub fn new(execution_auto: ExecutionAuto, execution_count: Integer, execution_required: ExecutionRequired, execution_status: ExecutionStatus, code: String, programming_language: String, guess_language: Boolean, content: Vec<Block>) -> Self {
        Self{
            execution_auto,
            execution_count,
            execution_required,
            execution_status,
            code,
            programming_language,
            guess_language,
            content,
            ..Default::default()
        }
    }
}

