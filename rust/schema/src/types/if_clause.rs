// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::boolean::Boolean;
use super::code_error::CodeError;
use super::cord::Cord;
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

/// A clause within a `If` node.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "div", custom)]
pub struct IfClause {
    /// The type of this item.
    pub r#type: MustBe!("IfClause"),

    /// The identifier for this item.
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code.
    #[strip(code)]
    #[jats(content)]
    pub code: Cord,

    /// The programming language of the code.
    #[strip(code)]
    #[jats(attr = "language")]
    pub programming_language: String,

    /// Whether the programming language of the code should be guessed based on syntax and variables used.
    #[strip(code)]
    pub guess_language: Option<Boolean>,

    /// The content to render if the result is truthy
    #[strip(types)]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<IfClauseOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct IfClauseOptions {
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

    /// Whether this clause is the active clause in the parent `If` node
    #[strip(output)]
    pub is_active: Option<Boolean>,
}

impl IfClause {
    pub fn new(code: Cord, programming_language: String, content: Vec<Block>) -> Self {
        Self {
            code,
            programming_language,
            content,
            ..Default::default()
        }
    }
}
