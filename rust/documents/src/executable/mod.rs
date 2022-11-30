use std::path::PathBuf;

use common::{async_trait::async_trait, eyre::Result};
use formats::Format;
use graph_triples::{ResourceInfo, TagMap};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::Address;
use node_patch::Patch;
use parsers::ParseInfo;
use stencila_schema::{
    ExecutionDependent, ExecutionDependentNode, ExecutionDependentRelation, Variable,
};

use crate::document::DocumentEventListener;

/// Trait for executable document nodes
///
/// This trait is implemented below for all (or at least, most) node types.
#[async_trait]
pub trait Executable {
    /// Compile a node
    async fn compile(&self, _address: &mut Address, _context: &mut CompileContext) -> Result<()> {
        Ok(())
    }

    // Review if should have an `ExecuteContext` similar to `CompileContext` to reduce number of args
    async fn execute_begin(
        &mut self,
        _resource_info: &ResourceInfo,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        Ok(None)
    }

    async fn execute_end(&mut self, _task_info: TaskInfo, _task_result: TaskResult) -> Result<()> {
        Ok(())
    }

    // TODO: Review whether `execute` method is necessary in addition to `exec_begin` and `exec_sync`.
    async fn execute(&mut self, _context: &mut ExecuteContext) -> Result<()> {
        Ok(())
    }
}

pub struct CompileContext<'lt> {
    /// The path of the document being compiled
    ///
    /// Used to resolve relative paths e.g. in `ImageObject` nodes
    pub path: PathBuf,

    /// The project that the document is within
    ///
    /// Used to restrict any file links to be within the project.
    pub project: PathBuf,

    /// The document's kernel space
    ///
    /// Used to guess programming languages from syntax and variables used
    pub kernel_space: &'lt KernelSpace,

    pub event_listeners: Vec<(String, String, DocumentEventListener)>,

    /// A list of resources compiled from the nodes
    pub resource_infos: Vec<ResourceInfo>,

    /// Any global tags defined in code chunks
    pub global_tags: TagMap,

    /// A list of patch operations representing changes to nodes.
    pub patches: Vec<Patch>,
}

impl<'lt> CompileContext<'lt> {
    /// Push a patch to the context
    pub fn push_patch(&mut self, patch: Patch) {
        self.patches.push(patch)
    }

    /// Push an event listener to the context
    pub fn push_event_listener(
        &mut self,
        listener_id: String,
        event_topic: String,
        event_listener: DocumentEventListener,
    ) {
        self.event_listeners
            .push((listener_id, event_topic, event_listener))
    }

    /// Guess the language of the code using the context's kernel space
    pub fn guess_language(&self, code: &str) -> Format {
        self.kernel_space
            .guess_language(code, Format::Calc, None, None)
    }

    /// Parse code using the context's path as the namespace
    pub fn parse_code(&self, language: &str, code: &str) -> Result<ParseInfo> {
        let format = formats::match_name(language);
        parsers::parse(format, code, Some(&self.path))
    }

    /// Create an dependent `Variable` using the context's path as the namespace
    pub fn dependent_variable(&self, name: &str, kind: &str) -> ExecutionDependent {
        ExecutionDependent {
            dependent_relation: ExecutionDependentRelation::Assigns,
            dependent_node: ExecutionDependentNode::Variable(Variable {
                namespace: self.path.to_string_lossy().to_string(),
                name: name.to_string(),
                kind: if kind.is_empty() {
                    None
                } else {
                    Some(Box::new(kind.to_string()))
                },
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct ExecuteContext<'lt> {
    #[allow(dead_code)]
    kernel_space: &'lt KernelSpace,
}

/// Ensure the node has an `id`, generating one if necessary
#[macro_export]
macro_rules! ensure_id {
    ($node:expr, $prefix:expr, $context:expr) => {{
        let id = if let Some(id) = $node.id.as_deref() {
            id
        } else {
            $node.id = Some(suids::generate($prefix));
            $node
                .id
                .as_deref()
                .expect("Has an id because it was just assigned")
        };
        id
    }};
}

/// Assert that a node has an id
#[macro_export]
macro_rules! assert_id {
    ($node:expr) => {
        $node.id.as_deref().ok_or_else(|| {
            common::eyre::eyre!(
                "Node of type `{}` does not have an id",
                std::any::type_name::<Self>()
            )
        })
    };
}

mod prelude;

mod generics;
mod structs;

mod button;
mod call;
mod code_chunk;
mod code_expression;
mod code_static;
mod division;
mod for_;
mod form;
mod if_;
mod include;
mod link;
mod math;
mod media;
mod others;
mod parameter;
mod shared;
mod software_source_code;
mod span;
