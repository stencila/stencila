use std::str::FromStr;

use common::{
    eyre::{self, bail, Result},
    serde::Serialize,
    strum::AsRefStr,
};
use graph_triples::ResourceInfo;

/// An execution plan for a document
#[derive(Debug, Default, Serialize)]
#[serde(crate = "common::serde")]
pub struct Plan {
    /// The options used to generate the plan
    pub options: PlanOptions,

    /// The stages to be executed
    pub stages: Vec<PlanStage>,
}

impl Plan {
    pub fn to_markdown(&self) -> String {
        let options = self.options.to_markdown();
        let stages = self
            .stages
            .iter()
            .enumerate()
            .map(|(index, stage)| format!("## Stage {}\n\n{}", index + 1, stage.to_markdown()))
            .collect::<Vec<String>>()
            .join("\n\n");
        format!("{}\n\n{}", options, stages)
    }
}

/// Options for generating a plan
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct PlanOptions {
    /// The ordering of nodes used when generating the plan
    pub ordering: PlanOrdering,

    /// The maximum task concurrency
    ///
    /// Limits the number of tasks that can be grouped together in a stage.
    /// Defaults to the number of logical CPU cores on the current machine.
    pub max_concurrency: usize,
}

impl PlanOptions {
    pub fn default_ordering() -> PlanOrdering {
        PlanOrdering::Topological
    }

    pub fn default_max_concurrency() -> usize {
        num_cpus::get()
    }

    pub fn to_markdown(&self) -> String {
        format!(
            r"## Options

- Ordering: {}
- Maximum concurrency: {}",
            self.ordering.as_ref(),
            self.max_concurrency
        )
    }
}

impl Default for PlanOptions {
    fn default() -> Self {
        Self {
            ordering: Self::default_ordering(),
            max_concurrency: Self::default_max_concurrency(),
        }
    }
}

/// The ordering of nodes used when generating a plan
#[derive(Debug, Clone, Serialize, AsRefStr)]
#[serde(crate = "common::serde")]
pub enum PlanOrdering {
    /// Only a single, specified, node is to be executed
    Single,

    /// Nodes are executed in the order that they appear in the
    /// document, top to bottom, left to right.
    Appearance,

    /// Nodes are executed in the order that ensures
    /// that the dependencies of a node are executed before it is
    Topological,
}

impl FromStr for PlanOrdering {
    type Err = eyre::Report;

    fn from_str(str: &str) -> Result<PlanOrdering> {
        Ok(match str.to_lowercase().as_str() {
            "s" | "si" | "sin" | "single" => PlanOrdering::Single,
            "a" | "ap" | "app" | "appear" | "appearance" => PlanOrdering::Appearance,
            "t" | "to" | "top" | "topo" | "topological" => PlanOrdering::Topological,
            _ => bail!("Unrecognized plan ordering: {}", str),
        })
    }
}

/// The scope of a cancellation request
#[derive(Debug, Clone, Serialize, AsRefStr)]
#[serde(crate = "common::serde")]
pub enum PlanScope {
    /// A single node in the current execution plan
    Single,

    /// All nodes in the current execution plan
    All,
}

impl FromStr for PlanScope {
    type Err = eyre::Report;

    fn from_str(str: &str) -> Result<PlanScope> {
        Ok(match str.to_lowercase().as_str() {
            "s" | "si" | "sin" | "single" => PlanScope::Single,
            "a" | "all" => PlanScope::All,
            _ => bail!("Unrecognized plan scope: {}", str),
        })
    }
}

/// A stage in an execution plan
///
/// A stage represents a group of [`PlanTask`]s that can be executed concurrently
/// (e.g. because they can be executed in different kernels or a kernel fork)
#[derive(Debug, Default, Serialize)]
#[serde(crate = "common::serde")]
pub struct PlanStage {
    /// The tasks to be executed
    pub tasks: Vec<PlanTask>,
}

impl PlanStage {
    pub fn to_markdown(&self) -> String {
        self.tasks
            .iter()
            .map(|task| ["- ", &task.to_markdown()].concat())
            .collect::<Vec<String>>()
            .join("\n")
    }
}

/// A task in an execution plan
///
/// A task is the smallest unit in an execution plan and corresponds to a kernel [`Task`]
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
pub struct PlanTask {
    /// Information on the resource to be executed
    ///
    /// Passed to kernel for `symbols_used` etc
    pub resource_info: ResourceInfo,

    /// The name of the kernel that the code will be executed in
    ///
    /// If this is `None` it indicates that no kernel capable of executing
    /// the node is available on the machine
    pub kernel_name: Option<String>,

    /// The code will be executed in a fork of the kernel
    ///
    /// Code that has no side effects or who's side effects should be
    /// ignored (i.e. is "@pure") are executed in a fork of the kernel.
    pub is_fork: bool,
}

impl PlanTask {
    pub fn to_markdown(&self) -> String {
        let node_type = self.resource_info.resource.node_type().unwrap_or("?");
        let node_id = self.resource_info.resource.node_id().unwrap_or("?");
        let kernel_name = self.kernel_name.as_deref().unwrap_or("?");
        let fork = if self.is_fork { "**fork**" } else { "" };

        format!(
            "Run `{}` *{}* in *{}* kernel {}",
            node_type, node_id, kernel_name, fork,
        )
    }
}
