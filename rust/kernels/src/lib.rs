use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap, HashSet, VecDeque},
    sync::Arc,
    time::Duration,
};

use graph_triples::ResourceInfo;
#[allow(unused_imports)]
use kernel::{
    common::{
        async_trait::async_trait,
        chrono::{DateTime, Utc},
        derive_more::{Deref, DerefMut},
        eyre::{bail, eyre, Result},
        serde::Serialize,
        serde_json, slug,
        strum::{EnumString, EnumVariantNames, VariantNames},
        tokio::{
            self,
            sync::{broadcast, mpsc, Mutex},
            task::JoinHandle,
        },
        tracing,
    },
    stencila_schema::{CodeError, Node},
    KernelId, KernelInfo, KernelStatus, KernelTrait, TaskId, TaskMessages, TaskOutputs,
};

// Re-exports
pub use kernel::{Kernel, KernelSelector, KernelType, Task, TaskResult};

/// A "meta" kernel to dispatch to different types of kernels
///
/// In the future this maybe changed to, or augmented with a `Box<dyn KernelTrait>`,
/// to allow dispatching to plugins that are dynamically added at runtime.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
enum MetaKernel {
    #[cfg(feature = "kernel-store")]
    Store(kernel_store::StoreKernel),

    #[cfg(feature = "kernel-calc")]
    Calc(kernel_calc::CalcKernel),

    #[cfg(feature = "kernel-micro")]
    Micro(kernel_micro::MicroKernel),

    #[cfg(feature = "kernel-jupyter")]
    Jupyter(kernel_jupyter::JupyterKernel),
}

impl MetaKernel {
    /// Create a new `MetaKernel` instance based on a selector which matches against the
    /// name or language of the kernel
    async fn new(selector: &KernelSelector) -> Result<Self> {
        #[cfg(feature = "kernel-store")]
        {
            let kernel = kernel_store::StoreKernel::new();
            if selector.is_empty() || selector.matches(&kernel.spec().await) {
                return Ok(MetaKernel::Store(kernel));
            }
        }

        macro_rules! matches_kernel {
            ($feat:literal, $variant:path, $kernel:expr) => {
                #[cfg(feature = $feat)]
                {
                    if selector.matches(&$kernel.spec().await) && $kernel.is_available().await {
                        return Ok($variant($kernel));
                    }
                }
            };
        }

        matches_kernel!(
            "kernel-calc",
            MetaKernel::Calc,
            kernel_calc::CalcKernel::new()
        );

        matches_kernel!("kernel-bash", MetaKernel::Micro, kernel_bash::new());
        matches_kernel!("kernel-deno", MetaKernel::Micro, kernel_deno::new());
        matches_kernel!("kernel-node", MetaKernel::Micro, kernel_node::new());
        matches_kernel!("kernel-python", MetaKernel::Micro, kernel_python::new());
        matches_kernel!("kernel-r", MetaKernel::Micro, kernel_r::new());
        matches_kernel!("kernel-zsh", MetaKernel::Micro, kernel_zsh::new());

        matches_kernel!(
            "kernel-jupyter",
            MetaKernel::Jupyter,
            kernel_jupyter::JupyterKernel::new(selector).await
        );

        bail!(
            "Unable to create an execution kernel for selector `{}`",
            selector
        )
    }
}

macro_rules! dispatch_variants {
    ($var:expr, $method:ident $(,$arg:expr)*) => {
        match $var {
            #[cfg(feature = "kernel-store")]
            MetaKernel::Store(kernel) => kernel.$method($($arg),*),
            #[cfg(feature = "kernel-calc")]
            MetaKernel::Calc(kernel) => kernel.$method($($arg),*),
            #[cfg(feature = "kernel-micro")]
            MetaKernel::Micro(kernel) => kernel.$method($($arg),*),
            #[cfg(feature = "kernel-jupyter")]
            MetaKernel::Jupyter(kernel) => kernel.$method($($arg),*),
        }
    };
}

#[async_trait]
impl KernelTrait for MetaKernel {
    async fn spec(&self) -> Kernel {
        dispatch_variants!(self, spec).await
    }

    async fn is_available(&self) -> bool {
        dispatch_variants!(self, is_available).await
    }

    async fn is_interruptable(&self) -> bool {
        dispatch_variants!(self, is_interruptable).await
    }

    async fn is_forkable(&self) -> bool {
        dispatch_variants!(self, is_forkable).await
    }

    async fn start(&mut self) -> Result<()> {
        dispatch_variants!(self, start).await
    }

    async fn stop(&mut self) -> Result<()> {
        dispatch_variants!(self, stop).await
    }

    async fn status(&self) -> Result<KernelStatus> {
        dispatch_variants!(self, status).await
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        dispatch_variants!(self, get, name).await
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        dispatch_variants!(self, set, name, value).await
    }

    async fn exec(&mut self, code: &str) -> Result<(TaskOutputs, TaskMessages)> {
        dispatch_variants!(self, exec, code).await
    }

    async fn exec_sync(&mut self, code: &str) -> Result<Task> {
        dispatch_variants!(self, exec_sync, code).await
    }

    async fn exec_async(&mut self, code: &str) -> Result<Task> {
        dispatch_variants!(self, exec_async, code).await
    }

    async fn exec_fork(&mut self, code: &str) -> Result<Task> {
        dispatch_variants!(self, exec_fork, code).await
    }
}

/// A map of kernel ids to kernels.
#[derive(Debug, Default, Deref, DerefMut, Serialize)]
#[serde(crate = "common::serde")]
struct KernelMap(BTreeMap<KernelId, MetaKernel>);

impl KernelMap {
    /// Get a reference to a kernel
    fn get(&self, kernel_id: &str) -> Result<&MetaKernel> {
        (**self)
            .get(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Get a mutable reference to a kernel
    fn get_mut(&mut self, kernel_id: &str) -> Result<&mut MetaKernel> {
        (**self)
            .get_mut(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Ensure that a kernel exists for a selector
    ///
    /// Returns the kernel's id.
    async fn ensure(&mut self, selector: &KernelSelector) -> Result<KernelId> {
        // Is there already a running kernel that matches the selector?
        for (kernel_id, kernel) in self.iter_mut() {
            if let Some(id) = &selector.id {
                if id != kernel_id {
                    // Not the right id, so keep looking
                    continue;
                }
            } else if !selector.matches(&kernel.spec().await) {
                // Not a match, so keep looking
                continue;
            }

            let status = match kernel.status().await {
                Ok(status) => status,
                Err(error) => {
                    tracing::error!("While getting status of kernel `{}`: {}", kernel_id, error);
                    continue;
                }
            };

            match status {
                // For these, use the existing kernel
                KernelStatus::Pending
                | KernelStatus::Starting
                | KernelStatus::Idle
                | KernelStatus::Busy => return Ok(kernel_id.clone()),
                // For these, keep on looking
                KernelStatus::Unresponsive
                | KernelStatus::Stopping
                | KernelStatus::Finished
                | KernelStatus::Failed
                | KernelStatus::Unknown => continue,
            }
        }

        // If unable to set in an existing kernel then start a new kernel
        // for the selector.
        self.start(selector).await
    }

    /// Start a kernel for a selector
    async fn start(&mut self, selector: &KernelSelector) -> Result<KernelId> {
        let mut kernel = MetaKernel::new(selector).await?;
        kernel.start().await?;

        // Generate the kernel id from the selector, adding a numeric suffix if necessary
        let kernel_id = slug::slugify(kernel.spec().await.name);
        let count = self
            .keys()
            .filter(|key| key.starts_with(&kernel_id))
            .count();
        let kernel_id = if count == 0 {
            kernel_id
        } else {
            [kernel_id, count.to_string()].concat()
        };

        self.insert(kernel_id.clone(), kernel);

        Ok(kernel_id)
    }

    /// Stop one of the kernels and remove it from the kernel space
    async fn stop(&mut self, id: &str) -> Result<()> {
        self.get_mut(id)?.stop().await?;
        self.remove(id);
        Ok(())
    }

    /// Connect to a running kernel
    #[allow(unused_variables)]
    async fn connect(&mut self, id_or_path: &str) -> Result<KernelId> {
        #[cfg(feature = "kernel-jupyter")]
        {
            let (kernel_id, kernel) = kernel_jupyter::JupyterKernel::connect(id_or_path).await?;
            self.insert(kernel_id.clone(), MetaKernel::Jupyter(kernel));

            Ok(kernel_id)
        }

        #[cfg(not(feature = "kernel-jupyter"))]
        kernel::common::eyre::bail!(
            "Unable to connect to running kernel because support for Jupyter kernels is not enabled",
        )
    }

    /// Get a list of kernels (including their details and status)
    pub async fn list(&self) -> KernelInfos {
        let mut list = KernelInfos::new();
        for (id, kernel) in self.iter() {
            let id = id.to_string();
            let spec = kernel.spec().await;
            let status = match kernel.status().await {
                Ok(status) => status,
                Err(error) => {
                    tracing::warn!("While getting kernel status: {}", error);
                    KernelStatus::Unknown
                }
            };
            let interruptable = kernel.is_interruptable().await;
            let forkable = kernel.is_forkable().await;
            list.insert(
                id.to_string(),
                KernelInfo {
                    id,
                    status,
                    spec,
                    interruptable,
                    forkable,
                },
            );
        }
        list
    }

    /// Display a list of kernels in the kernel space
    #[cfg(feature = "cli")]
    pub async fn display(&self) -> cli_utils::Result {
        use cli_utils::result;

        let list = self.list().await;

        let cols = "|--|------|----|----|---------|-------------|--------|";
        let head = "|Id|Status|Type|Name|Languages|Interruptable|Forkable|";
        let body = list
            .iter()
            .map(|(_, info)| {
                format!(
                    "|{}|{}|{}|{}|{}|{}|{}|",
                    info.id,
                    info.status,
                    info.spec.r#type,
                    info.spec.name,
                    info.spec.languages.join(", "),
                    info.interruptable,
                    info.forkable
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let md = format!(
            "{top}\n{head}\n{align}\n{body}\n{bottom}\n",
            top = cols,
            head = head,
            align = cols,
            body = body,
            bottom = if !list.is_empty() { cols } else { "" }
        );

        result::new("md", &md, list)
    }
}

/// Information on a symbol in a kernel space
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct SymbolInfo {
    /// The type of the object that the symbol refers to (e.g `Number`, `Function`)
    ///
    /// Should be used as a hint only, to the underlying, native type of the symbol.
    kind: String,

    /// The home kernel of the symbol
    ///
    /// The home kernel of a symbol is the kernel that it was last assigned in.
    /// As such, a symbol's home kernel can change, although this is discouraged.
    home: KernelId,

    /// The time that the symbol was last modified in the home kernel
    ///
    /// A symbol is considered modified when a `CodeChunk` with an `Declare`, `Assign` or `Alter`
    /// relation to the symbol is executed or the `kernel.set` method is called.
    modified: DateTime<Utc>,

    /// The time that the symbol was last mirrored to other kernels
    ///
    /// A timestamp is recorded for each time that a symbol is mirrored to another
    /// kernel. This allows unnecessary mirroring to be avoided if the symbol has
    /// not been assigned since it was last mirrored to that kernel.
    mirrored: HashMap<KernelId, DateTime<Utc>>,
}

impl SymbolInfo {
    pub fn new(kind: &str, kernel_id: &str) -> Self {
        SymbolInfo {
            kind: kind.into(),
            home: kernel_id.into(),
            modified: Utc::now(),
            mirrored: HashMap::new(),
        }
    }
}

pub type KernelSymbols = HashMap<String, SymbolInfo>;

/// Disassociates a kernel with a symbol. If a symbol is mirrored in other kernels
/// then the last kernel that is was mirrored to will become it's home.
fn purge_kernel_from_symbols(symbols: &mut KernelSymbols, kernel_id: &str) {
    let mut remove = Vec::new();
    for (symbol, symbol_info) in symbols.iter_mut() {
        if symbol_info.home == kernel_id {
            let mut mirrors = symbol_info.mirrored.iter().collect::<Vec<_>>();
            mirrors.sort_by(|a, b| a.1.cmp(b.1));
            match mirrors.last() {
                Some((kernel, _)) => symbol_info.home = kernel.to_string(),
                None => remove.push(symbol.to_string()),
            }
        } else {
            symbol_info.mirrored.retain(|kernel, _| kernel != kernel_id);
        }
    }
    symbols.retain(|symbol, _| !remove.contains(symbol));
}

/// Display kernel symbols
#[cfg(feature = "cli")]
fn display_symbols(symbols: &KernelSymbols) -> cli_utils::Result {
    use cli_utils::result;

    let cols = "|------|----|-----------|------------|-------------------|";
    let head = "|Symbol|Type|Home kernel|Last altered|Mirrored in kernels|";
    let body = symbols
        .iter()
        .map(|(symbol, symbol_info)| {
            format!(
                "|{}|{}|{}|{}|{}|",
                symbol,
                symbol_info.kind,
                symbol_info.home,
                format_time(symbol_info.modified),
                symbol_info
                    .mirrored
                    .iter()
                    .map(|(kernel, time)| format!("{} ({})", kernel, format_time(*time)))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let md = format!(
        "{top}\n{head}\n{align}\n{body}\n{bottom}\n",
        top = cols,
        head = head,
        align = cols,
        body = body,
        bottom = if !symbols.is_empty() { cols } else { "" }
    );

    result::new("md", &md, symbols)
}

#[derive(Debug, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct TaskInfo {
    /// The unique number for the task within the [`KernelSpace`]
    ///
    /// An easier way to be able to refer to a task than by its [`TaskId`].
    pub num: u64,

    /// The code that was executed
    pub code: String,

    /// The result of parsing the code
    pub resource_info: ResourceInfo,

    /// The id of the kernel that the task was dispatched to
    pub kernel_id: Option<String>,

    /// Whether the task has been scheduled to run in a fork of the kernel
    pub is_fork: bool,

    /// Whether the task has been deferred until the kernel is idle
    pub is_deferred: bool,

    /// Whether the task is asynchronous
    pub is_async: bool,

    /// Whether the task can be cancelled
    pub is_cancellable: bool,

    /// The task that this information is for
    #[serde(flatten)]
    pub task: Task,
}

impl TaskInfo {
    /// Convenience method to access the task results
    pub async fn result(&mut self) -> Result<TaskResult> {
        self.task.result().await
    }

    /// Was the task started?
    pub fn was_started(&self) -> bool {
        self.task.started.is_some()
    }

    /// Was the task finished?
    pub fn was_finished(&self) -> bool {
        self.task.finished.is_some()
    }

    /// Was the task cancelled?
    pub fn was_cancelled(&self) -> bool {
        self.task.cancelled.is_some()
    }

    /// Time that the the task ended (either finished, cancelled, or `None`)
    pub fn ended(&self) -> Option<DateTime<Utc>> {
        self.task.finished.or(self.task.cancelled)
    }

    /// Calculate the task duration in seconds
    pub fn duration(&self) -> Option<f64> {
        let duration = if let (Some(began), Some(ended)) = (self.task.started, self.ended()) {
            Some(ended - began)
        } else {
            self.task.started.map(|started| (Utc::now() - started))
        };
        duration.map(|duration| duration.to_std().unwrap_or(Duration::ZERO).as_secs_f64())
    }
}

/// A list of [`Task`]s associated with a [`KernelSpace`]
///
/// The main purpose of maintaining this list of tasks is for introspection
/// and the ability to cancel running or queued tasks. To avoid the list growing
/// indefinitely, tasks are removed on a periodic basis, if the list is greater
/// than a certain size.
#[derive(Debug, Default)]
struct KernelTasks {
    /// The list of tasks
    inner: Vec<TaskInfo>,

    /// A counter to be able to assign unique numbers to tasks
    counter: u64,
}

#[derive(Debug, EnumVariantNames, EnumString)]
#[strum(serialize_all = "lowercase", crate = "kernel::common::strum")]
enum KernelTaskSorting {
    /// Sort by task number (default)
    Number,
    /// Sort by time created
    Created,
    /// Sort by time started
    Started,
    /// Sort by time finished
    Finished,
    /// Sort by time cancelled
    Cancelled,
}

impl KernelTasks {
    /// Find a task using its `num` or [`TaskId`]
    fn find_mut<'lt>(&'lt mut self, num_or_id: &str) -> Option<&'lt mut TaskInfo> {
        if let Ok(num) = num_or_id.parse::<u64>() {
            for task_info in self.inner.iter_mut() {
                if task_info.num == num {
                    return Some(task_info);
                }
            }
            None
        } else {
            match TaskId::try_from(num_or_id) {
                Ok(id) => self.get_mut(&id),
                Err(..) => None,
            }
        }
    }

    /// Get a task using its [`TaskId`]
    fn get<'lt>(&'lt self, task_id: &TaskId) -> Option<&'lt TaskInfo> {
        for task_info in self.inner.iter() {
            if task_info.task.id == *task_id {
                return Some(task_info);
            }
        }
        None
    }

    /// Get a task using its [`TaskId`]
    fn get_mut<'lt>(&'lt mut self, task_id: &TaskId) -> Option<&'lt mut TaskInfo> {
        for task_info in self.inner.iter_mut() {
            if task_info.task.id == *task_id {
                return Some(task_info);
            }
        }
        None
    }

    /// Put a task onto the list
    async fn put(
        &mut self,
        task: &Task,
        code: &str,
        resource_info: &ResourceInfo,
        kernel_id: &str,
        is_fork: bool,
        is_deferred: bool,
    ) -> TaskInfo {
        self.counter += 1;

        let task_info = TaskInfo {
            num: self.counter,
            code: code.to_string(),
            resource_info: resource_info.clone(),
            kernel_id: Some(kernel_id.to_string()),
            is_fork,
            is_deferred,
            is_async: task.is_async(),
            is_cancellable: task.is_cancellable(),
            task: task.clone(),
        };

        self.inner.push(task_info.clone());

        task_info
    }

    /// Display the tasks
    ///
    /// Mainly for inspection, in the future may return a formatted table
    /// with more information
    #[cfg(feature = "cli")]
    async fn display(
        &self,
        num: usize,
        sort: &KernelTaskSorting,
        desc: bool,
        kernel: Option<KernelId>,
        queues: &KernelQueues,
    ) -> cli_utils::Result {
        use cli_utils::result;

        let mut list = self.inner.clone();

        if kernel.is_some() {
            list = list
                .into_iter()
                .filter(|task_info| task_info.kernel_id == kernel)
                .collect::<Vec<TaskInfo>>();
        }

        match sort {
            KernelTaskSorting::Number => (),
            &KernelTaskSorting::Created => list.sort_by(|a, b| a.task.created.cmp(&b.task.created)),
            &KernelTaskSorting::Started => list.sort_by(|a, b| a.task.started.cmp(&b.task.started)),
            &KernelTaskSorting::Finished => {
                list.sort_by(|a, b| a.task.finished.cmp(&b.task.finished))
            }
            &KernelTaskSorting::Cancelled => {
                list.sort_by(|a, b| a.task.cancelled.cmp(&b.task.cancelled))
            }
        }

        if desc {
            list.reverse()
        }

        if list.len() > num {
            if desc {
                list.drain(..num);
            } else {
                list.drain(..(list.len() - num));
            }
        }

        let cols =
            "|-|-------|-------|--------|---------|--------|------|------|------|-----------|----|";
        let head =
            "|#|Created|Started|Finished|Cancelled|Duration|Kernel|Queued|Forked|Cancellable|Code|";
        let align =
            "|-|------:|------:|-------:|--------:|-------:|:-----|-----:|-----:|----------:|:---|";
        let body = list
            .iter()
            .map(|task_info| {
                let task = &task_info.task;

                let kernel_id = task_info.kernel_id.clone().unwrap_or_default();

                let queue_pos = queues
                    .get(&kernel_id)
                    .and_then(|queue| queue.binary_search(&task.id).ok())
                    .map(|index| (index + 1).to_string())
                    .unwrap_or_default();

                let fork = if task_info.is_fork { "yes" } else { "no" };

                let cancellable = if task_info.is_cancellable {
                    "yes"
                } else {
                    "no"
                };

                let mut code = task_info.code.clone();
                if code.len() > 20 {
                    code.truncate(17);
                    code += "...";
                }
                code = code.replace('\n', "; ");

                format!(
                    "|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|`{}`|",
                    task_info.num,
                    format_time(task.created),
                    task.started.map(format_time).unwrap_or_default(),
                    task.finished.map(format_time).unwrap_or_default(),
                    task.cancelled.map(format_time).unwrap_or_default(),
                    format_duration(task_info.duration()),
                    kernel_id,
                    queue_pos,
                    fork,
                    cancellable,
                    code,
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let md = format!(
            "{top}\n{head}\n{align}\n{body}\n{bottom}\n",
            top = cols,
            head = head,
            align = align,
            body = body,
            bottom = if !list.is_empty() { cols } else { "" }
        );

        result::new("md", &md, list)
    }
}

type KernelQueues = HashMap<KernelId, VecDeque<TaskId>>;

/// Display task queues
#[cfg(feature = "cli")]
fn display_queues(queues: &KernelQueues, tasks: &KernelTasks) -> cli_utils::Result {
    use cli_utils::result;

    let md = queues
        .keys()
        .map(|kernel_id| {
            let display = display_queue(queues, kernel_id, tasks)
                .map(|value| value.content.unwrap_or_default())
                .unwrap_or_else(|err| err.to_string());
            format!("## Kernel '{}'\n\n{}", kernel_id, display)
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    result::new("md", &md, queues)
}

/// Display a task queue
#[cfg(feature = "cli")]
fn display_queue(queues: &KernelQueues, kernel_id: &str, tasks: &KernelTasks) -> cli_utils::Result {
    use cli_utils::result;

    let queue = queues
        .get(kernel_id)
        .map_or_else(VecDeque::new, |queue| queue.clone());

    let cols = "|--------|-----------|--------|-------|";
    let head = "|Position|Task number| Task id|Created|";
    let body = queue
        .iter()
        .enumerate()
        .map(|(index, task_id)| {
            let task_info = match tasks.get(task_id) {
                Some(task) => task,
                None => return "".to_string(),
            };
            let task = &task_info.task;

            format!(
                "|{}|{}|{}|{}|",
                index + 1,
                task_info.num,
                task.id,
                format_time(task.created)
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let md = format!(
        "{top}\n{head}\n{align}\n{body}\n{bottom}\n",
        top = cols,
        head = head,
        align = cols,
        body = body,
        bottom = if !queue.is_empty() { cols } else { "" }
    );

    result::new("md", &md, queue)
}

#[derive(Debug, Default)]
pub struct KernelSpace {
    /// The kernels in the kernel space
    kernels: Arc<Mutex<KernelMap>>,

    /// The symbols in the kernel space
    symbols: Arc<Mutex<KernelSymbols>>,

    /// The list of all tasks sent to this kernel space
    tasks: Arc<Mutex<KernelTasks>>,

    /// The queue of deferred tasks
    queues: Arc<Mutex<KernelQueues>>,

    /// The monitoring task for the kernel
    monitoring: Option<JoinHandle<()>>,
}

impl Drop for KernelSpace {
    fn drop(&mut self) {
        if let Some(monitoring) = &self.monitoring {
            monitoring.abort()
        }
    }
}

pub type KernelInfos = HashMap<KernelId, KernelInfo>;

impl KernelSpace {
    /// Create a new kernel space and start its monitoring task
    pub fn new() -> Self {
        let mut kernel_space = Self::default();

        let kernels = kernel_space.kernels.clone();
        let queue = kernel_space.queues.clone();
        let tasks = kernel_space.tasks.clone();
        let symbols = kernel_space.symbols.clone();
        kernel_space.monitoring = Some(tokio::spawn(async move {
            KernelSpace::monitor(&kernels, &queue, &tasks, &symbols).await
        }));

        kernel_space
    }

    /// Monitor the kernel space
    ///
    /// Checks for and dispatches queued tasks and monitors the health of kernels and tasks.
    async fn monitor(
        kernels: &Arc<Mutex<KernelMap>>,
        queues: &Arc<Mutex<KernelQueues>>,
        tasks: &Arc<Mutex<KernelTasks>>,
        symbols: &Arc<Mutex<KernelSymbols>>,
    ) {
        const PERIOD: Duration = Duration::from_millis(100);

        tracing::trace!("Began kernel space monitoring");
        loop {
            KernelSpace::dispatch_queue(queues, tasks, kernels, symbols).await;
            KernelSpace::clean_tasks(tasks).await;
            tokio::time::sleep(PERIOD).await;
        }
    }

    /// Get the list of kernels in the kernel space
    pub async fn kernels(&self) -> KernelInfos {
        let kernels = &*self.kernels.lock().await;
        kernels.list().await
    }

    /// Get the list of symbols in the kernel space
    pub async fn symbols(&self) -> KernelSymbols {
        let symbols = &*self.symbols.lock().await;
        symbols.clone()
    }

    /// Start a kernel
    pub async fn start(&self, selector: &KernelSelector) -> Result<KernelId> {
        let kernels = &mut *self.kernels.lock().await;
        kernels.start(selector).await
    }

    /// Stop a kernel
    pub async fn stop(&self, id: &str) -> Result<()> {
        let kernels = &mut *self.kernels.lock().await;
        kernels.stop(id).await?;

        let symbols = &mut *self.symbols.lock().await;
        purge_kernel_from_symbols(symbols, id);

        Ok(())
    }

    /// Restart one, or all, kernels in the kernel space
    pub async fn restart(&self, id: Option<String>) -> Result<()> {
        let kernels = &mut *self.kernels.lock().await;
        let symbols = &mut *self.symbols.lock().await;

        let ids = match id {
            Some(id) => vec![id],
            None => kernels.keys().cloned().collect(),
        };

        for id in &ids {
            let kernel = kernels.get(id)?;
            let spec = kernel.spec().await;
            let selector = KernelSelector::new(Some(spec.name), None, None);

            kernels.stop(id).await?;
            purge_kernel_from_symbols(symbols, id);
            kernels.start(&selector).await?;
        }

        Ok(())
    }

    /// Get a symbol from the kernel space
    pub async fn get(&self, name: &str) -> Result<Node> {
        let symbols = &mut *self.symbols.lock().await;
        let symbol_info = symbols
            .get(name)
            .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

        let kernels = &mut *self.kernels.lock().await;
        let kernel = kernels.get_mut(&symbol_info.home)?;
        kernel.get(name).await
    }

    /// Set a symbol in the kernel space
    pub async fn set(&self, name: &str, value: Node, selector: &KernelSelector) -> Result<()> {
        let kernels = &mut *self.kernels.lock().await;

        let kernel_id = kernels.ensure(selector).await?;
        tracing::debug!("Setting symbol `{}` in kernel `{}`", name, kernel_id);

        let kernel = kernels.get_mut(&kernel_id)?;
        kernel.set(name, value).await?;

        let symbols = &mut *self.symbols.lock().await;
        match symbols.entry(name.to_string()) {
            Entry::Occupied(mut occupied) => {
                let info = occupied.get_mut();
                info.home = kernel_id;
                info.modified = Utc::now();
            }
            Entry::Vacant(vacant) => {
                vacant.insert(SymbolInfo::new("", &kernel_id));
            }
        }

        Ok(())
    }

    /// Execute some code in the kernel space
    pub async fn exec(
        &self,
        code: &str,
        resource_info: &ResourceInfo,
        force_fork: bool,
        selector: &KernelSelector,
    ) -> Result<TaskInfo> {
        let kernels = &mut *self.kernels.lock().await;

        // Determine the kernel to execute in
        let kernel_id = kernels.ensure(selector).await?;

        // If the kernel is busy then defer the task, otherwise dispatch to the kernel now
        let kernel = kernels.get(&kernel_id)?;
        let (is_deferred, task) = if kernel.is_busy().await? {
            let task = self.defer_task(&kernel_id).await;
            (true, task)
        } else {
            let symbols = &mut *self.symbols.lock().await;
            let task = match KernelSpace::dispatch_task(
                code,
                resource_info,
                force_fork,
                symbols,
                &kernel_id,
                kernels,
            )
            .await
            {
                Ok(task) => task,
                Err(error) => Task::not_dispatched(&error.to_string()),
            };
            (false, task)
        };

        // Either way, store the task
        let task_info = self
            .store(
                &task,
                code,
                resource_info,
                &kernel_id,
                force_fork,
                is_deferred,
            )
            .await;
        Ok(task_info)
    }

    /// Dispatch a task to a kernel
    ///
    /// Symbols that the code uses, but have a different home kernel, are mirrored to the kernel.
    /// Ensures that the kernel has necessary variables before dispatching and that any
    /// variables it assigns are recorded.
    #[allow(clippy::too_many_arguments)]
    async fn dispatch_task(
        code: &str,
        resource_info: &ResourceInfo,
        force_fork: bool,
        symbols: &mut KernelSymbols,
        kernel_id: &str,
        kernels: &mut KernelMap,
    ) -> Result<Task> {
        tracing::trace!("Dispatching task to kernel `{}`", kernel_id);

        // Mirror used symbols into the kernel
        for symbol in resource_info.symbols_used() {
            let name = &symbol.name;
            let symbol = match symbols.get_mut(name) {
                Some(symbol) => symbol,
                // Skip if unknown symbol (e.g a package, or variable assigned elsewhere)
                None => continue,
            };

            // Skip if home is the target kernel
            if symbol.home == *kernel_id {
                continue;
            }

            // Skip if already mirrored since last assigned
            if let Some(mirrored) = symbol.mirrored.get(kernel_id) {
                if mirrored >= &symbol.modified {
                    continue;
                }
            }

            tracing::debug!(
                "Mirroring symbol `{}` from kernel `{}` to kernel `{}`",
                name,
                symbol.home,
                kernel_id
            );

            let home_kernel = kernels.get_mut(&symbol.home)?;
            let value = home_kernel.get(name).await?;

            let mirror_kernel = kernels.get_mut(kernel_id)?;
            mirror_kernel.set(name, value).await?;

            symbol
                .mirrored
                .entry(kernel_id.to_string())
                .and_modify(|datetime| *datetime = Utc::now())
                .or_insert_with(Utc::now);
        }

        // Execute the code in the kernel
        let pure = resource_info.is_pure();
        let kernel = kernels.get_mut(kernel_id)?;
        let task = if force_fork || (pure && kernel.is_forkable().await) {
            kernel.exec_fork(code).await?
        } else {
            kernel.exec_async(code).await?
        };

        // Record symbols assigned in kernel (unless it was a fork)
        if !pure {
            for symbol in resource_info.symbols_modified() {
                symbols
                    .entry(symbol.name.clone())
                    .and_modify(|info| {
                        info.home = kernel_id.to_string();
                        info.modified = Utc::now();
                    })
                    .or_insert_with(|| SymbolInfo::new(&symbol.kind, kernel_id));
            }
        }

        Ok(task)
    }

    /// Defer a task
    ///
    /// Used when a kernel is busy. Instead of dispatching the task to the kernel,
    /// add it to the task queue so it can be more easily, and less expensively, cancelled
    /// by simply removing it from the queue rather than interrupting the kernel.
    ///
    /// When using an execution `Plan` this method should not be necessary since the tasks
    /// will usually only be created when the kernel is `Idle`. Nonetheless, this method
    /// may be invoked in other circumstances such as when multiple background tasks are
    /// dispatched to the same kernel from the CLI.
    async fn defer_task(&self, kernel_id: &str) -> Task {
        tracing::trace!("Deferring task for kernel `{}`", kernel_id);

        let (sender, ..) = broadcast::channel(1);
        let (canceller, mut cancellee) = mpsc::channel(1);
        let task = Task::defer(Some(sender), Some(canceller));

        // Add the task to the queue for the kernel
        let mut queues = self.queues.lock().await;
        let task_id = task.id.clone();
        queues
            .entry(kernel_id.to_string())
            .and_modify(|queue| queue.push_back(task_id.clone()))
            .or_insert_with(|| VecDeque::from_iter(vec![task_id]));

        // When cancelled, remove the task from the queue for the kernel
        let queues = self.queues.clone();
        let kernel_id_clone = kernel_id.to_string();
        let task_id = task.id.clone();
        tokio::spawn(async move {
            if let Some(..) = cancellee.recv().await {
                let mut queues = queues.lock().await;
                queues
                    .entry(kernel_id_clone)
                    .and_modify(|queue| queue.retain(|task_idd| *task_idd != task_id));
            }
        });

        task
    }

    /// Dispatch tasks from the queue to a kernel
    async fn dispatch_queue(
        queues: &Arc<Mutex<KernelQueues>>,
        tasks: &Arc<Mutex<KernelTasks>>,
        kernels: &Arc<Mutex<KernelMap>>,
        symbols: &Arc<Mutex<KernelSymbols>>,
    ) {
        let mut queues = queues.lock().await;

        if queues.is_empty() {
            return;
        }

        let mut kernels = kernels.lock().await;
        let mut tasks = tasks.lock().await;
        let mut symbols = symbols.lock().await;

        let mut kernels_removed = Vec::new();
        for (kernel_id, queue) in queues.iter_mut() {
            if queue.is_empty() {
                continue;
            }
            if let Ok(kernel) = kernels.get_mut(kernel_id) {
                if !kernel.is_busy().await.unwrap_or(false) {
                    let task_id = queue
                        .pop_front()
                        .expect("Should have at least one because we checked above");

                    if let Some(task_info) = tasks.get_mut(&task_id) {
                        tracing::debug!("Dispatching task `{}` to kernel `{}`", task_id, kernel_id);
                        if let Err(error) = KernelSpace::dispatch_deferred(
                            task_info,
                            kernel_id,
                            &mut *kernels,
                            &mut *symbols,
                        )
                        .await
                        {
                            tracing::error!(
                                "While dispatching task `{}` to kernel `{}`: {}",
                                task_id,
                                kernel_id,
                                error
                            );
                        }
                    } else {
                        tracing::debug!(
                            "Unable to find task `{}`; was removed from queue for kernel `{}`",
                            task_id,
                            kernel_id
                        );
                    }
                }
            } else {
                tracing::debug!(
                    "Unable to find kernel `{}`; associated queue will be removed",
                    kernel_id
                );
                kernels_removed.push(kernel_id.clone());
            }
        }

        for kernel_id in kernels_removed {
            queues.remove(&kernel_id);
        }
    }

    /// Dispatch a previously deferred task to a kernel
    async fn dispatch_deferred(
        task_info: &mut TaskInfo,
        kernel_id: &str,
        kernels: &mut KernelMap,
        symbols: &mut KernelSymbols,
    ) -> Result<()> {
        tracing::trace!(
            "Dispatching deferred task `{}` to kernel `{}`",
            task_info.task.id,
            kernel_id
        );

        let deferred_task = &mut task_info.task;
        let task_id = deferred_task.id.clone();

        // Dispatch the task
        let started_task = KernelSpace::dispatch_task(
            &task_info.code,
            &task_info.resource_info,
            false,
            symbols,
            kernel_id,
            kernels,
        )
        .await?;

        // Update the started time
        deferred_task.started = started_task.started;

        // Update `is_async` and forward the result of the started task to the deferred task (because
        // other parts of the code will be waiting on it).
        if let Some(result_sender) = deferred_task.sender.as_ref() {
            if let Some(result) = started_task.result {
                // Result is available already so send now
                task_info.is_async = false;

                if let Err(error) = result_sender.send(result) {
                    tracing::debug!(
                        "While forwarding result for deferred task `{}`: {}",
                        task_id,
                        error
                    )
                }
            } else if let Ok(mut result_receiver) = started_task.subscribe() {
                // Task is async so subscribe to result channel and forward on the deferred task
                task_info.is_async = true;

                let result_sender = result_sender.clone();
                let task_id = task_id.clone();
                tokio::spawn(async move {
                    if let Ok(result) = result_receiver.recv().await {
                        if let Err(error) = result_sender.send(result) {
                            tracing::debug!(
                                "While forwarding result for deferred task `{}`: {}",
                                task_id,
                                error
                            )
                        }
                    }
                });
            } else {
                bail!("Started task had neither a result nor a sender")
            }
        } else {
            bail!("Deferred task did not have expected result sender")
        }

        // Update `is_cancellable` (a deferred task is always cancellable)
        // This will work if cancellation is done using a `TaskId` but might not
        // if `.cancel()` is called on the original deferred task.
        if let Some(canceller) = started_task.canceller.as_ref() {
            task_info.is_cancellable = true;
            deferred_task.canceller = Some(canceller.clone());
        } else {
            task_info.is_cancellable = false;
            deferred_task.canceller = None;
        }

        Ok(())
    }

    /// Store a task (either one that has been dispatched or is deferred)
    ///
    /// If the task is async, subscribe to it so that it's result can be updated when it
    /// is complete.
    #[allow(clippy::too_many_arguments)]
    async fn store(
        &self,
        task: &Task,
        code: &str,
        resource_info: &ResourceInfo,
        kernel_id: &str,
        is_fork: bool,
        is_deferred: bool,
    ) -> TaskInfo {
        if let (false, Ok(mut receiver)) = (task.is_done(), task.subscribe()) {
            // When finished, update the tasks info stored in `tasks`
            let tasks = self.tasks.clone();
            let task_id = task.id.clone();
            tokio::spawn(async move {
                match receiver.recv().await {
                    Ok(result) => {
                        let mut tasks = tasks.lock().await;
                        match KernelTasks::get_mut(&mut tasks, &task_id) {
                            // Finish the task with the result
                            Some(task_info) => task_info.task.finished(result),
                            // Task may have been removed from list, so just debug here
                            None => tracing::debug!("Unable to find task `{}`", task_id),
                        }
                    }
                    Err(error) => tracing::error!(
                        "While receiving result for async task `{}`: {}",
                        task_id,
                        error
                    ),
                }
            });
        }

        let mut tasks = self.tasks.lock().await;
        tasks
            .put(task, code, resource_info, kernel_id, is_fork, is_deferred)
            .await
    }

    /// Cancel a task
    async fn cancel(&self, task_num_or_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task_info) = tasks.find_mut(task_num_or_id) {
            task_info.task.cancel().await
        } else {
            tracing::warn!(
                "Unable to find task `{}`; it may have already been cleaned up",
                task_num_or_id
            );
            Ok(())
        }
    }

    /// Cancel all unfinished tasks
    async fn cancel_all(&self) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        for TaskInfo { task, .. } in tasks.inner.iter_mut() {
            if !task.is_done() {
                task.cancel().await?;
            }
        }
        Ok(())
    }

    /// Remove old tasks to avoid the `tasks` list growing indefinitely in long running processes.
    async fn clean_tasks(tasks: &Arc<Mutex<KernelTasks>>) {
        // Currently a large MAX_SIZE to avoid removing unfinished task and assuming each task
        // does not take up too much memory.
        // May need to be made an env var and the default reduced.
        const MAX_SIZE: usize = 100_000;

        let tasks = &mut *tasks.lock().await;
        let list = &mut tasks.inner;

        // Work out how many tasks need to be removed
        let count = list.len().saturating_sub(MAX_SIZE);
        if count == 0 {
            return;
        }

        tracing::debug!("Removing `{}` tasks from task list", count);
        let mut remove = Vec::with_capacity(count);

        // Try to remove those tasks that are "done" first
        for (index, task_info) in list.iter().enumerate() {
            if remove.len() >= count {
                break;
            }
            if task_info.task.is_done() {
                remove.push(index);
            }
        }

        // If not are enough are removed then warn, and remove the oldest
        if remove.len() < count {
            tracing::warn!("While cleaning tasks, have to remove `{}` unfinished tasks to respect MAX_SIZE of {}", count - remove.len(), MAX_SIZE);
            for index in 0..(list.len()) {
                if remove.len() >= count {
                    break;
                }
                if !remove.contains(&index) {
                    remove.push(index)
                }
            }
        }

        // Finally, do the actual removal. This involves a call to `cancel` for tasks that are
        // not finished to make sure they are removed from the queue and any result receivers are stopped.
        for index in remove {
            let TaskInfo { task, .. } = &mut list[index];
            if !task.is_done() {
                if let Err(error) = task.cancel().await {
                    tracing::debug!("While cancelling unfinished task `{}`: {}", task.id, error)
                }
            }
            list.remove(index);
        }
    }

    /// A read-evaluate-print function
    ///
    /// Primarily intended for use in interactive mode to execute a line of code REPL style
    /// (see the `Execute` CLI command).
    #[cfg(feature = "cli")]
    pub async fn repl(
        &self,
        code: &str,
        language: Option<String>,
        kernel: Option<String>,
        background: bool,
        is_fork: bool,
    ) -> cli_utils::Result {
        use cli_utils::result;
        use common::{once_cell::sync::Lazy, regex::Regex};
        use events::{subscribe, unsubscribe, Subscriber};
        use graph_triples::resources;
        use std::path::PathBuf;

        static SYMBOL: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[a-zA-Z]\w*$").expect("Unable to create regex"));

        if code.is_empty() {
            result::nothing()
        } else if language.is_none() && kernel.is_none() && SYMBOL.is_match(code) {
            match self.get(code).await {
                Ok(node) => result::value(node),
                Err(err) => {
                    tracing::error!("{}", err);
                    result::nothing()
                }
            }
        } else {
            let code = code.trim().replace("\\n", "\n");

            // If possible, parse the code so that we can use the relations to determine variables that
            // are assigned or used (needed for variable mirroring).
            let path = PathBuf::from("<cli>");
            let resource = resources::code(&path, "<id>", "<file>", language.clone());
            let resource_info = match parsers::parse(resource.clone(), &code) {
                Ok(resource_info) => resource_info,
                Err(..) => ResourceInfo::default(resource),
            };

            // Determine the kernel selector
            let selector = match kernel {
                Some(kernel) => {
                    // Combine the kernel language option with the provided selector
                    let mut selector = KernelSelector::parse(&kernel);
                    selector.lang = language;
                    selector
                }
                None => match language {
                    // Selector based on language only
                    Some(_) => KernelSelector::new(None, language, None),
                    None => {
                        let tasks = self.tasks.lock().await;
                        match tasks.inner.last() {
                            // Select for kernel used for the last task
                            Some(task_info) => KernelSelector {
                                id: task_info.kernel_id.clone(),
                                ..Default::default()
                            },
                            // Select anything (will select the first kernel)
                            None => KernelSelector::new(None, None, None),
                        }
                    }
                },
            };

            // Execute the code
            let mut task_info = self.exec(&code, &resource_info, is_fork, &selector).await?;

            if background {
                // Indicate task is running in background
                tracing::info!("Task #{} is running in background", task_info.num);
                result::nothing()
            } else {
                // If cancellable, subscribe to user interrupt, cancelling the task on that event
                let subscription_id = if task_info.task.is_cancellable() {
                    let tasks = self.tasks.clone();
                    let task_id = task_info.task.id.clone();
                    let (sender, mut receiver) = mpsc::unbounded_channel();
                    tokio::spawn(async move {
                        // This ends on interrupt or on unsubscribe (when the channel sender is dropped)
                        if let Some(..) = receiver.recv().await {
                            let mut tasks = tasks.lock().await;
                            if let Some(TaskInfo { task, .. }) = tasks.find_mut(&task_id) {
                                if let Err(error) = task.cancel().await {
                                    tracing::error!(
                                        "While cancelling task `{}`: {}",
                                        task_id,
                                        error
                                    );
                                }
                            }
                        }
                    });

                    let subscription_id =
                        subscribe("interrupt", Subscriber::UnboundedSender(sender))?;
                    Some(subscription_id)
                } else {
                    None
                };

                // Wait for the result
                let TaskResult { outputs, messages } = task_info.result().await?;

                // Cancel interrupt subscription
                if let Some(subscription_id) = subscription_id {
                    unsubscribe(&subscription_id)?;
                }

                // Output messages and  outputs
                if !messages.is_empty() {
                    for error in messages {
                        let mut err = error.error_message;
                        if let Some(trace) = error.stack_trace {
                            err += &format!("\n{}", trace);
                        }
                        tracing::error!("{}", err)
                    }
                }
                match outputs.len() {
                    0 => result::nothing(),
                    1 => result::value(outputs[0].clone()),
                    _ => result::value(outputs),
                }
            }
        }
    }
}

/// List the kernels that are available on this machine
#[allow(clippy::vec_init_then_push, unused_mut)]
pub async fn available() -> Result<Vec<Kernel>> {
    let mut available: Vec<Kernel> = Vec::new();

    #[cfg(feature = "kernel-store")]
    available.push(kernel_store::StoreKernel::new().spec().await);

    #[cfg(feature = "kernel-calc")]
    available.push(kernel_calc::CalcKernel::new().spec().await);

    macro_rules! microkernel_available {
        ($feat:literal, $crat:ident, $list:expr) => {
            #[cfg(feature = $feat)]
            {
                let kernel = $crat::new();
                if kernel.is_available().await {
                    $list.push(kernel.spec().await)
                }
            }
        };
    }
    microkernel_available!("kernel-bash", kernel_bash, available);
    microkernel_available!("kernel-deno", kernel_deno, available);
    microkernel_available!("kernel-node", kernel_node, available);
    microkernel_available!("kernel-python", kernel_python, available);
    microkernel_available!("kernel-r", kernel_r, available);
    microkernel_available!("kernel-zsh", kernel_zsh, available);

    #[cfg(feature = "kernel-jupyter")]
    available.append(&mut kernel_jupyter::JupyterKernel::available().await?);

    Ok(available)
}

/// List the languages supported by the kernels available on this machine
///
/// Returns a list of unique language names across the available kernels.
#[allow(clippy::vec_init_then_push)]
pub async fn languages() -> Result<Vec<String>> {
    let mut languages: HashSet<String> = HashSet::new();
    let kernels = available().await?;
    for kernel in kernels {
        for lang in kernel.languages {
            let format = formats::match_name(&lang);
            let lang = match format {
                formats::Format::Unknown => lang,
                _ => format.spec().title,
            };
            languages.insert(lang);
        }
    }
    let mut languages: Vec<String> = languages.into_iter().collect();
    languages.sort();
    Ok(languages)
}

/// List the Jupyter kernels and servers that are currently running on this machine
pub async fn jupyter_running() -> Result<serde_json::Value> {
    #[cfg(feature = "kernel-jupyter")]
    {
        let kernels = kernel_jupyter::JupyterKernel::running().await?;
        let servers = kernel_jupyter::JupyterServer::running().await?;
        Ok(serde_json::json!({
            "kernels": kernels,
            "servers": servers
        }))
    }
    #[cfg(not(feature = "kernel-jupyter"))]
    {
        bail!("Jupyter kernels are not enabled")
    }
}

/// List the directories that are searched for Jupyter kernel spaces
pub async fn jupyter_directories() -> Result<serde_json::Value> {
    #[cfg(feature = "kernel-jupyter")]
    {
        Ok(serde_json::json!({
            "kernels": kernel_jupyter::dirs::kernel_dirs(),
            "runtime": kernel_jupyter::dirs::runtime_dirs(),
        }))
    }
    #[cfg(not(feature = "kernel-jupyter"))]
    {
        bail!("Jupyter kernels are not enabled")
    }
}

/// Format an optional `DateTime` into a human readable "ago" duration
fn format_time(time: DateTime<Utc>) -> String {
    let duration = (Utc::now() - time).to_std().unwrap_or(Duration::ZERO);
    let rounded = Duration::from_secs(duration.as_secs());
    [
        humantime::format_duration(rounded).to_string(),
        " ago".to_string(),
    ]
    .concat()
}

/// Format begin and end times into a human readable, rounded to milliseconds
fn format_duration(seconds: Option<f64>) -> String {
    match seconds {
        Some(seconds) => {
            let duration = Duration::from_secs(seconds as u64);
            humantime::format_duration(duration).to_string()
        }
        _ => "".to_string(),
    }
}

#[cfg(feature = "cli")]
#[allow(deprecated)] // Remove when using clap 4.0 (https://github.com/clap-rs/clap/issues/3822)
pub mod commands {
    use cli_utils::{
        clap::{self, Parser},
        result, Result, Run,
    };
    use common::once_cell::sync::Lazy;
    use tokio::sync::Mutex;

    use super::*;

    /// Manage and use execution kernels
    #[derive(Debug, Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, Parser)]
    pub enum Action {
        Available(Available),
        Languages(Languages),

        Running(Running),
        Start(Start),
        Connect(Connect),
        Stop(Stop),
        Show(Show),

        Execute(Execute),
        Tasks(Tasks),
        Queues(Queues),
        Cancel(Cancel),
        Symbols(Symbols),
        Restart(Restart),

        External(External),
        Directories(Directories),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::Available(action) => action.run().await,
                Action::Languages(action) => action.run().await,
                Action::External(action) => action.run().await,
                Action::Directories(action) => action.run().await,
                _ => {
                    let kernel_space = &mut *KERNEL_SPACE.lock().await;
                    match action {
                        Action::Running(action) => action.run(kernel_space).await,
                        Action::Start(action) => action.run(kernel_space).await,
                        Action::Connect(action) => action.run(kernel_space).await,
                        Action::Stop(action) => action.run(kernel_space).await,
                        Action::Show(action) => action.run(kernel_space).await,

                        Action::Execute(action) => action.run(kernel_space).await,
                        Action::Tasks(action) => action.run(kernel_space).await,
                        Action::Queues(action) => action.run(kernel_space).await,
                        Action::Cancel(action) => action.run(kernel_space).await,
                        Action::Symbols(action) => action.run(kernel_space).await,
                        Action::Restart(action) => action.run(kernel_space).await,

                        _ => bail!("Unhandled action: {:?}", action),
                    }
                }
            }
        }
    }

    /// List the kernels that are available on this machine
    ///
    /// The list of available kernels includes those that are built into the Stencila
    /// binary (e.g. `calc`), Jupyter kernels installed on the machine, and Microkernels
    /// for which a supporting runtime (e.g. `python`) is installed.
    #[derive(Debug, Parser)]
    #[clap(alias = "avail")]
    pub struct Available {}
    #[async_trait]
    impl Run for Available {
        async fn run(&self) -> Result {
            result::value(available().await?)
        }
    }

    /// List the languages supported by the kernels available on this machine
    ///
    /// Returns a unique list of languages across all kernels available.
    #[derive(Debug, Parser)]
    #[clap(alias = "langs")]
    pub struct Languages {}
    #[async_trait]
    impl Run for Languages {
        async fn run(&self) -> Result {
            result::value(languages().await?)
        }
    }

    /// List the Jupyter kernels and servers that are currently running on this machine
    ///
    /// This command scans the Jupyter `runtime` directory to get a list of running
    /// Jupyter notebook servers. It then gets a list of kernels from the REST API
    /// of each of those servers.
    #[derive(Debug, Parser)]
    pub struct External {}
    #[async_trait]
    impl Run for External {
        async fn run(&self) -> Result {
            result::value(jupyter_running().await?)
        }
    }

    /// List the directories on this machine that will be searched for Jupyter kernel specs
    /// and running kernels
    #[derive(Debug, Parser)]
    #[clap(alias = "dirs")]
    pub struct Directories {}
    #[async_trait]
    impl Run for Directories {
        async fn run(&self) -> Result {
            result::value(jupyter_directories().await?)
        }
    }

    /// A lazily initialized kernel space for the execute command. Required so that
    /// kernel state is maintained in successive calls to `Execute::run` when in
    /// interactive mode
    static KERNEL_SPACE: Lazy<Mutex<KernelSpace>> = Lazy::new(|| Mutex::new(KernelSpace::new()));

    /// Execute code within a document kernel space
    ///
    /// Mainly intended for testing that Stencila is able to talk
    /// to Jupyter kernels and execute code within them.
    ///
    /// Use the `--kernel` option to specify, by name, language or type, which kernel the code
    /// should be executed in e.g.,
    ///
    /// > kernels execute Math.PI --lang=javascript
    ///
    /// > kernels execute Math.PI --lang javascript --kernel="type:jupyter"
    ///
    /// In interactive mode, you can set the command prefix to "stay" in a particular
    /// language and mimic a REPL in that language e.g.,
    ///
    /// > >> kernels execute --lang=javascript
    /// > let r = 10
    /// > 2 * Math.PI * r
    ///
    /// If a kernel is not yet running for the language then one will be started
    /// (if installed on the machine).
    #[derive(Debug, Parser)]
    #[clap(alias = "exec")]
    pub struct Execute {
        /// Code to execute within the kernel space
        // Using a `Vec` and the `multiple_values` option allows for spaces in the code
        #[clap(multiple_values = true)]
        code: Vec<String>,

        /// The programming language of the code
        #[clap(short, long)]
        lang: Option<String>,

        /// The kernel where the code should executed (a kernel selector string)
        #[clap(short, long)]
        kernel: Option<String>,

        /// The task should run be in the background
        #[clap(short, long, alias = "back")]
        background: bool,

        /// The task should run be in a kernel fork (if possible)
        #[clap(long)]
        fork: bool,
    }
    impl Execute {
        pub async fn run(&self, kernel_space: &mut KernelSpace) -> Result {
            kernel_space
                .repl(
                    &self.code.join(" "),
                    self.lang.clone(),
                    self.kernel.clone(),
                    self.background,
                    self.fork,
                )
                .await
        }
    }

    /// List the code execution tasks in a document kernel space
    #[derive(Debug, Parser)]
    pub struct Tasks {
        /// The maximum number of tasks to show
        #[clap(short, long, default_value = "100")]
        num: usize,

        /// The order to sort tasks (defaults to by task number)
        #[clap(
            short, long,
            possible_values = KernelTaskSorting::VARIANTS,
            default_value = "number"
        )]
        sort: KernelTaskSorting,

        /// Whether to sort in descending order
        #[clap(short, long)]
        desc: bool,

        /// Only show tasks assigned to a specific kernel
        #[clap(short, long)]
        kernel: Option<KernelId>,
    }
    impl Tasks {
        pub async fn run(&self, kernel_space: &KernelSpace) -> Result {
            let tasks = kernel_space.tasks.lock().await;
            let queues = kernel_space.queues.lock().await;
            tasks
                .display(
                    self.num,
                    &self.sort,
                    self.desc,
                    self.kernel.clone(),
                    &*queues,
                )
                .await
        }
    }

    /// Show the code execution queues in a document kernel space
    #[derive(Debug, Parser)]
    pub struct Queues {
        /// Only show the queue for a specific kernel
        #[clap(short, long)]
        kernel: Option<KernelId>,
    }
    impl Queues {
        pub async fn run(&self, kernel_space: &KernelSpace) -> Result {
            let tasks = kernel_space.tasks.lock().await;
            let queues = kernel_space.queues.lock().await;
            match &self.kernel {
                Some(kernel) => display_queue(&*queues, kernel, &*tasks),
                None => display_queues(&*queues, &*tasks),
            }
        }
    }

    /// Show the code symbols in a document kernel space
    #[derive(Debug, Parser)]
    pub struct Symbols {}
    impl Symbols {
        pub async fn run(&self, kernel_space: &KernelSpace) -> Result {
            let symbols = kernel_space.symbols.lock().await;
            display_symbols(&*symbols)
        }
    }

    /// Cancel a code execution task, or all tasks, in a document kernel space
    ///
    /// Use an integer to cancel a task by it's number.
    /// Use "all" to cancel all unfinished tasks.
    #[derive(Debug, Parser)]
    pub struct Cancel {
        /// The task number or id, or "all"
        task: String,
    }
    impl Cancel {
        pub async fn run(&self, kernel_space: &mut KernelSpace) -> Result {
            let which = self.task.trim();
            if which == "all" {
                kernel_space.cancel_all().await?;
            } else {
                let which = which.strip_prefix('#').unwrap_or(which);
                kernel_space.cancel(which).await?;
            }
            result::nothing()
        }
    }

    /// List the kernels in a document kernel space
    ///
    /// Mainly intended for interactive mode testing / inspection. Note that
    /// for a kernel to be in this list it must have either been started by Stencila,
    ///
    /// > kernels start r
    ///
    /// or connected to from Stencila,
    ///  
    /// > kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
    ///
    /// To get a list of externally started Jupyter kernels that can be connected to run,
    ///
    /// > kernels external
    #[derive(Debug, Parser)]
    #[clap(alias = "kernels")]
    pub struct Running {}
    impl Running {
        pub async fn run(&self, kernel_space: &KernelSpace) -> Result {
            let kernels = kernel_space.kernels.lock().await;
            kernels.display().await
        }
    }

    /// Start a kernel
    ///
    /// Mainly intended for testing that kernels that rely on external files or processes
    /// (i.e. a Jupyter kernel or a Microkernel) can be started successfully.
    #[derive(Debug, Parser)]
    pub struct Start {
        /// The name or programming language of the kernel
        selector: String,
    }
    impl Start {
        pub async fn run(&self, kernel_space: &mut KernelSpace) -> Result {
            let selector = KernelSelector::parse(&self.selector);
            let kernel_id = kernel_space.start(&selector).await?;
            let kernels = kernel_space.kernels.lock().await;
            let kernel = kernels.get(&kernel_id)?;
            tracing::info!("Successfully started kernel");
            result::value(kernel)
        }
    }

    /// Stop a kernel
    ///
    /// Mainly intended for testing that Jupyter kernels are successfully
    /// stopped (e.g. cleanup of connection files).
    ///
    /// Only kernels that were started by Stencila can be stopped. A kernel
    /// that were started externally by a Jupyter server and then connected to
    /// will still run but Stencila will clone any connections to it.
    #[derive(Debug, Parser)]
    pub struct Stop {
        /// The id of the kernel
        id: String,
    }
    impl Stop {
        pub async fn run(&self, kernel_space: &mut KernelSpace) -> Result {
            kernel_space.stop(&self.id).await?;
            tracing::info!("Stopped kernel `{}`", self.id);
            result::nothing()
        }
    }

    /// Restart one or all of the kernels
    #[derive(Debug, Parser)]
    pub struct Restart {
        /// The id of the kernel (defaults to all)
        id: Option<String>,
    }
    impl Restart {
        pub async fn run(&self, kernel_space: &KernelSpace) -> Result {
            kernel_space.restart(self.id.clone()).await?;
            match &self.id {
                Some(id) => tracing::info!("Restarted kernel `{}`", id),
                None => tracing::info!("Restarted all kernels"),
            };
            result::nothing()
        }
    }

    /// Connect to a running Jupyter kernel
    ///
    /// Mainly intended for testing that Stencila is able to connect
    /// to an existing kernel (e.g. one that was started from Jupyter notebook).
    ///
    /// To get a list of externally started kernels that can be connected to run,
    ///
    /// > kernels external
    ///
    /// and then connect to a kernel using its Jupyter id e.g.,
    ///
    /// > kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
    ///
    /// Alternatively, use the path (relative or absolute) of the Jupyter notebook
    /// whose (already started) kernel you wish to connect to e.g.,
    ///
    /// > kernels connect ../main.ipynb
    #[derive(Debug, Parser)]
    pub struct Connect {
        /// The id of the kernel e.g. `31248fc2-38d0-4d11-80a1-f8a1bd3842fb`
        /// or the relative path of the notebook
        id_or_path: String,
    }
    impl Connect {
        pub async fn run(&self, kernel_space: &mut KernelSpace) -> Result {
            let mut kernels = kernel_space.kernels.lock().await;
            let id = kernels.connect(&self.id_or_path).await?;
            tracing::info!("Connected to kernel `{}`", id);
            result::nothing()
        }
    }

    /// Show the details of a current kernel
    ///
    /// Mainly intended for interactive mode testing / inspection.
    #[derive(Debug, Parser)]
    pub struct Show {
        /// The id of the kernel (see `kernels status`)
        id: KernelId,
    }
    impl Show {
        pub async fn run(&self, kernel_space: &mut KernelSpace) -> Result {
            let kernels = kernel_space.kernels.lock().await;
            let kernel = kernels.get(&self.id)?;
            result::value(kernel)
        }
    }
}
