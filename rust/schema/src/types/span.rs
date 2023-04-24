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
use super::inline::Inline;
use super::integer::Integer;
use super::string::String;
use super::timestamp::Timestamp;

/// Styled inline content
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Span {
    /// The type of this item
    pub r#type: MustBe!("Span"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The code.
    pub code: String,

    /// The programming language of the code.
    pub programming_language: String,

    /// Whether the programming language of the code should be guessed based on syntax and variables used
    pub guess_language: Option<Boolean>,

    /// A Cascading Style Sheet (CSS) transpiled from the output of evaluating the `text` property.
    pub css: Option<String>,

    /// A list of class names associated with the document node
    pub classes: Option<Vec<String>>,

    /// The content within the span
    pub content: Vec<Inline>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<SpanOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct SpanOptions {
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

    /// Media type, typically expressed using a MIME format, of the code.
    pub media_type: Option<String>,
}

impl Span {
    pub fn new(code: String, programming_language: String, content: Vec<Inline>) -> Self {
        Self {
            code,
            programming_language,
            content,
            ..Default::default()
        }
    }
}
