// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::blocks_or_string::BlocksOrString;
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
use super::node::Node;
use super::string::String;
use super::timestamp::Timestamp;

/// A executable chunk of code.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[jats(elem = "code", attribs(executable = "yes"))]
pub struct CodeChunk {
    /// The type of this item
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("CodeChunk"),

    /// The identifier for this item
    #[strip(id)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code.
    #[strip(code)]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("code")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9\s\t\n]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^`]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    #[jats(content)]
    pub code: Cord,

    /// The programming language of the code.
    #[strip(code)]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("lang")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"(cpp)|(js)|(py)|(r)|(ts)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9]{1,10}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    #[jats(attr = "language")]
    pub programming_language: String,

    /// Whether the programming language of the code should be guessed based on syntax and variables used
    #[strip(code)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub guess_language: Option<Boolean>,

    /// Outputs from executing the chunk.
    #[strip(output)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub outputs: Option<Vec<Node>>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<CodeChunkOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct CodeChunkOptions {
    /// Under which circumstances the code should be automatically executed.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_auto: Option<ExecutionAuto>,

    /// A digest of the content, semantics and dependencies of the node.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_digest: Option<ExecutionDigest>,

    /// The `compileDigest` of the node when it was last executed.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_digest: Option<ExecutionDigest>,

    /// The upstream dependencies of this node.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_dependencies: Option<Vec<ExecutionDependency>>,

    /// The downstream dependants of this node.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_dependants: Option<Vec<ExecutionDependant>>,

    /// Tags in the code which affect its execution
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_tags: Option<Vec<ExecutionTag>>,

    /// A count of the number of times that the node has been executed.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_count: Option<Integer>,

    /// Whether, and why, the code requires execution or re-execution.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_required: Option<ExecutionRequired>,

    /// The id of the kernel that the node was last executed in.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_kernel: Option<String>,

    /// Status of the most recent, including any current, execution.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_status: Option<ExecutionStatus>,

    /// The timestamp when the last execution ended.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_ended: Option<Timestamp>,

    /// Duration of the last execution.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_duration: Option<Duration>,

    /// Errors when compiling (e.g. syntax errors) or executing the node.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub errors: Option<Vec<CodeError>>,

    /// Whether the code should be treated as side-effect free when executed.
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_pure: Option<Boolean>,

    /// A short label for the CodeChunk.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub label: Option<String>,

    /// A caption for the CodeChunk.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub caption: Option<BlocksOrString>,
}

impl CodeChunk {
    pub fn new(code: Cord, programming_language: String) -> Self {
        Self {
            code,
            programming_language,
            ..Default::default()
        }
    }
}
