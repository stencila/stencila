// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::automatic_execution::AutomaticExecution;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::duration::Duration;
use super::execution_dependant::ExecutionDependant;
use super::execution_dependency::ExecutionDependency;
use super::execution_message::ExecutionMessage;
use super::execution_required::ExecutionRequired;
use super::execution_status::ExecutionStatus;
use super::execution_tag::ExecutionTag;
use super::if_block_clause::IfBlockClause;
use super::integer::Integer;
use super::string::String;
use super::timestamp::Timestamp;

/// Show and execute alternative content conditional upon an executed expression.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "IfBlock")]
pub struct IfBlock {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("IfBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Under which circumstances the code should be automatically executed.
    #[serde(alias = "auto", alias = "auto-exec", alias = "auto_exec")]
    #[strip(execution)]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub auto_exec: Option<AutomaticExecution>,

    /// The clauses making up the `IfBlock` node
    #[serde(alias = "clause")]
    #[serde(deserialize_with = "one_or_many")]
    #[strip(code)]
    #[walk]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![ibc("true", None::<String>, [p([t("If clause")])])]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec(IfBlockClause::arbitrary(), size_range(1..=3))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec(IfBlockClause::arbitrary(), size_range(1..=10))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec(IfBlockClause::arbitrary(), size_range(1..=10))"#))]
    #[dom(elem = "div")]
    #[html(slot = "div")]
    pub clauses: Vec<IfBlockClause>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<IfBlockOptions>,

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
pub struct IfBlockOptions {
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

    /// The id of the actor that the node was last executed by.
    #[serde(alias = "execution-actor", alias = "execution_actor")]
    #[strip(execution)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub execution_actor: Option<String>,

    /// The timestamp when the last execution ended.
    #[serde(alias = "execution-ended", alias = "execution_ended")]
    #[strip(execution)]
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
}

impl IfBlock {
    const NICK: [u8; 3] = [105, 102, 98];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::IfBlock
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(clauses: Vec<IfBlockClause>) -> Self {
        Self {
            clauses,
            ..Default::default()
        }
    }
}
