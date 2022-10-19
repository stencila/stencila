use std::{collections::HashMap, path::PathBuf};

use common::{async_trait::async_trait, eyre::Result};
use graph_triples::{ResourceInfo, TagMap};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::{Address, AddressMap};
use node_patch::Patch;

/// Trait for executable document nodes
///
/// This trait is implemented below for all (or at least, most) node types.
#[async_trait]
pub trait Executable {
    /// Assemble a node
    ///
    /// Should ensure that the node has a id and that its address is registered
    /// against that id. The resulting `AddressMap` allows the use of pointers
    /// to reach in to the node tree and compile or execute a node when needed
    /// and potentially in a different order to how they appear in the tree.
    async fn assemble(
        &mut self,
        _address: &mut Address,
        _context: &mut AssembleContext,
    ) -> Result<()> {
        Ok(())
    }

    /// Compile a node
    async fn compile(&mut self, _context: &mut CompileContext) -> Result<()> {
        Ok(())
    }

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

    async fn execute(&mut self, _context: &mut ExecuteContext) -> Result<()> {
        Ok(())
    }
}

/// The context passed down through calls to the `Executable::assemble` method
#[derive(Debug, Default)]
pub struct AssembleContext {
    /// The path of the document being compiled
    ///
    /// Used to resolve relative paths e.g. in `Include` nodes
    pub path: PathBuf,

    /// Counts of the number of node ids with each prefix assigned
    ///
    /// Used to generate unique auto-incremented integer ids by node
    /// type.
    pub ids: HashMap<String, usize>,

    /// A map of node ids to addresses
    pub address_map: AddressMap,

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

    /// Register a node id by adding its id and address to the address map
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
        $node.id.as_deref().ok_or_else(|| {
            common::eyre::eyre!(
                "Node of type `{}` does not have an id",
                std::any::type_name::<Self>()
            )
        })
    };
}

#[derive(Debug)]
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

    /// A list of resources compiled from the nodes
    pub resource_infos: Vec<ResourceInfo>,

    /// Any global tags defined in code chunks
    pub global_tags: TagMap
}

#[derive(Debug)]
pub struct ExecuteContext<'lt> {
    kernel_space: &'lt KernelSpace,
}

mod button;
mod call;
mod code_chunk;
mod code_expression;
mod division;
mod for_;
mod form;
mod generics;
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
