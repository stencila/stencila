use std::{collections::HashMap, path::PathBuf};

use common::{async_trait::async_trait, eyre::Result};
use formats::Format;
use graph_triples::{ResourceInfo, TagMap};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::{Address, AddressMap};
use node_patch::Patch;

/// Trait for executable document nodes
///
/// This trait is implemented below for all (or at least most) node types.
#[async_trait]
pub trait Executable {
    async fn assemble(
        &mut self,
        _address: &mut Address,
        _context: &mut AssembleContext,
    ) -> Result<()> {
        Ok(())
    }

    async fn compile(&self, _context: &mut CompileContext) -> Result<()> {
        Ok(())
    }

    async fn execute_begin(
        &mut self,
        _resource_info: &ResourceInfo,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
        //_call_docs: &CallDocuments,
    ) -> Result<Option<TaskInfo>> {
        Ok(None)
    }

    async fn execute_end(&mut self, _task_info: TaskInfo, _task_result: TaskResult) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AssembleContext {
    /// The path of the document being compiled.
    /// Used to resolve relative paths e.g. in `Include` nodes
    pub path: PathBuf,

    /// Counts of the number of node ids with each prefix assigned
    pub ids: HashMap<String, usize>,

    /// A map of node ids to addresses
    pub address_map: AddressMap,

    /// A stack of ids of container nodes
    pub(crate) container_ids: Vec<String>,

    /// A map of node ids to the ids of nodes they contain
    //pub(crate) container_map: ContainerMap,

    /// A map of `Call` ids to their `source`
    /// Used so a document can maintain a `Document` for each `Call`
    /// (thereby reducing startup times associated with each execution of the call)
    //pub call_docs: Arc<RwLock<CallDocuments>>,

    /// A list of patch operations representing changes to nodes.
    pub patches: Vec<Patch>,
}

impl AssembleContext {
    /// Generate a unique id for a node
    ///
    /// These generated ids use a prefix reflecting the node type (i.g. "cc-" for `CodeChunk` nodes)
    /// which can be used to determine that it was generated (so, for example it is not persisted).
    /// They are deterministic which is also useful (and maybe assumed elsewhere in the code?)
    pub(crate) fn ensure_id(&mut self, prefix: &str) -> String {
        let count = self
            .ids
            .entry(prefix.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        [prefix, "-", &count.to_string()].concat()
    }

    /// Register a node id
    pub(crate) fn register_id(&mut self, id: String, address: Address) {
        self.address_map.insert(id, address);
    }
}

/// Ensure the node has an `id`, generating one if necessary, and register it in the context
///
/// This needs to be (?) a macro, rather than a generic function, because
/// it is not possible to define a bound that the type must have the `id` property.
#[macro_export]
macro_rules! register_id {
    ($prefix:expr, $node:expr, $address:expr, $context:expr) => {{
        let id = if let Some(id) = $node.id.as_deref() {
            id.clone()
        } else {
            let id = $context.ensure_id($prefix);
            $node.id = Some(Box::new(id.clone()));
            id
        };
        $context.register_id(id.clone(), $address.clone());
        id
    }};
}

/// Assert that a node has an id
#[macro_export]
macro_rules! assert_id {
    ($node:expr) => {
        $node
            .id
            .as_deref()
            .ok_or_else(|| common::eyre::eyre!("Node should have `id` assigned in assemble phase"))
    };
}

#[derive(Debug, Default)]
pub struct CompileContext {
    /// The path of the document being compiled.
    /// Used to resolve relative paths e.g. in `ImageObject` nodes
    pub path: PathBuf,

    /// The project that the document is within.
    /// Used to restrict any file links to be within the project
    pub project: PathBuf,

    /// The programming language of the last code node encountered during
    /// compilation
    pub programming_language: Format,

    /// A list of resources compiled from the nodes
    pub resource_infos: Vec<ResourceInfo>,

    /// Any global tags defined in code chunks
    pub global_tags: TagMap,

    /// A map of `Call` ids to their `source`
    /// Used so a document can get the parameters of the called doc
    //pub call_docs: Arc<RwLock<CallDocuments>>,

    /// A list of patch operations representing changes to nodes.
    pub patches: Vec<Patch>,
}

/// Set the programming of a node or of the context
///
/// Ok, bad name but it's like `ensure_id!`: if the node does
/// not have a `programming_language` then we'll use the context's
/// and if it does than we'll set the context's.
#[macro_export]
macro_rules! ensure_lang {
    ($node:expr, $context:expr) => {
        if $node.programming_language.is_empty() {
            match $context.programming_language {
                formats::Format::Unknown => String::new(),
                _ => $context.programming_language.spec().title,
            }
        } else {
            let format = formats::match_name(&$node.programming_language);
            $context.programming_language = format;
            format.spec().title
        }
    };
}

#[derive(Debug, Default)]
pub struct ExecuteContext {}


mod button;
mod call;
mod code_chunk;
mod code_expression;
mod shared;
mod division;
mod for_;
mod form;
mod if_;
mod include;
mod link;
mod media;
mod others;
mod parameter;
mod software_source_code;
mod span;
