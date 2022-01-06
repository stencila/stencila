use chrono::{DateTime, Utc};
use graph_triples::{resources, ResourceId};
use parsers::ParseInfo;
use serde::Serialize;

/// A step in an execution plan
///
/// A step is the smallest unit in an execution plan and corresponds to a kernel [`Task`]
/// (but to avoid confusion we use a different name here).
#[derive(Debug, Serialize)]
pub struct Step {
    /// The id of the resource to be executed
    ///
    /// This differs from the id of the node (which is scoped to the containing
    /// document).
    pub(crate) resource_id: ResourceId,

    /// The code resource to be executed
    pub(crate) code: resources::Code,

    /// The name of the kernel that the code will be executed in
    ///
    /// If this is `None` it indicates that no kernel capable of executing
    /// the node is available on the machine
    pub(crate) kernel_name: Option<String>,

    /// The parse info for the code
    pub(crate) parse_info: Option<ParseInfo>,

    /// The code will be executed in a fork of the kernel
    ///
    /// Code that has no side effects or who's side effects should be
    /// ignored (i.e. is "@pure") are executed in a fork of the kernel.
    pub(crate) is_fork: bool,
}

/// A summary of the execution of a step
///
/// Only records the information required by the planner. See kernel `Task` and `TaskInfo`
/// for more information.
#[derive(Debug, Clone, Serialize)]
pub struct StepInfo {
    /// When the step finished
    pub(crate) finished: DateTime<Utc>,

    /// The `code_hash` of the code at the time it was executed
    pub(crate) code_hash: u64,

    /// The `semantic_hash` of the code at the time it was executed
    pub(crate) semantic_hash: u64,
}

/// A stage in an execution plan
///
/// A stage represents a group of [`Step`]s that can be executed concurrently
/// (e.g. because they can be executed in different kernels or a kernel fork)
#[derive(Debug, Default, Serialize)]
pub struct Stage {
    /// The steps to be executed
    pub(crate) steps: Vec<Step>,
}

/// The ordering of nodes used when generating a plan
#[derive(Debug, Clone, Serialize)]
pub enum PlanOrdering {
    /// Nodes are executed in the order that they appear in the
    /// document, top to bottom, left to right.
    Appearance,

    /// Nodes are executed in the order that ensures
    /// that the dependencies of a node are executed before it is
    Topological,
}

/// Options for generating a plan
#[derive(Debug, Clone, Serialize)]
pub struct PlanOptions {
    /// The ordering of nodes used when generating the plan
    pub ordering: PlanOrdering,

    /// The maximum step concurrency
    ///
    /// Limits the number of [`Step`]s that can be grouped together in a [`Stage`].
    /// Defaults to the number of logical CPU cores on the current machine.
    pub max_concurrency: usize,
}

impl Default for PlanOptions {
    fn default() -> Self {
        Self {
            ordering: PlanOrdering::Topological,
            max_concurrency: num_cpus::get(),
        }
    }
}

/// An execution plan for a document
#[derive(Debug, Default, Serialize)]
pub struct Plan {
    /// The options used to generate the plan
    pub(crate) options: PlanOptions,

    /// The stages to be executed
    pub(crate) stages: Vec<Stage>,
}
