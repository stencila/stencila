use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut};
use graph_triples::{resources::Symbol, Relation, Resource};
#[allow(unused_imports)]
use kernel::{
    async_trait::async_trait,
    eyre::{bail, eyre, Result},
    stencila_schema::{CodeError, Node},
    Kernel, KernelId, KernelInfo, KernelStatus, KernelTrait,
};
use kernel::{TaskId, TaskMessages, TaskOutputs};
use serde::Serialize;
use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap, HashSet, VecDeque},
    sync::Arc,
};
use tokio::sync::{broadcast, mpsc, Mutex};

// Re-exports
pub use kernel::{KernelSelector, Task, TaskResult};

/// A "meta" kernel to dispatch to different types of kernels
///
/// In the future this maybe changed to, or augmented with a `Box<dyn KernelTrait>`,
/// to allow dispatching to plugins that are dynamically added at runtime.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize)]
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
        if selector.is_empty() {
            return Ok(MetaKernel::Store(kernel_store::StoreKernel::new()));
        }

        macro_rules! matches_kernel {
            ($feat:literal, $variant:path, $kernel:expr) => {
                #[cfg(feature = $feat)]
                {
                    if selector.matches(&$kernel.spec()) && $kernel.is_available().await {
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
    fn spec(&self) -> Kernel {
        dispatch_variants!(self, spec)
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
            if !selector.matches(&kernel.spec()) {
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
        let kernel_id = slug::slugify(kernel.spec().name);
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
        kernel::eyre::bail!(
            "Unable to connect to running kernel because support for Jupyter kernels is not enabled",
        )
    }

    /// Stop one of the kernels and remove it from the kernel space
    async fn stop(&mut self, id: &str) -> Result<()> {
        self.get_mut(id)?.stop().await?;
        self.remove(id);
        Ok(())
    }

    /// Get a list of kernels in the kernel space
    #[cfg(feature = "cli")]
    pub async fn display(&self) -> Vec<KernelInfo> {
        let mut info = Vec::new();
        for (id, kernel) in self.iter() {
            let id = id.to_string();
            let spec = kernel.spec();
            let status = match kernel.status().await {
                Ok(status) => status,
                Err(error) => {
                    tracing::warn!("While getting kernel status: {}", error);
                    KernelStatus::Unknown
                }
            };
            let interruptable = kernel.is_interruptable().await;
            let forkable = kernel.is_forkable().await;
            info.push(KernelInfo {
                id,
                status,
                spec,
                interruptable,
                forkable,
            })
        }
        info
    }
}

/// Information on a symbol in a kernel space
#[derive(Debug, Clone, Serialize)]
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

    /// The time that the symbol was last assigned in the home kernel
    ///
    /// A symbol is considered assigned when  a `CodeChunk` with an `Assign` relation
    /// to the symbol is executed or the `kernel.set` method is called.
    assigned: DateTime<Utc>,

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
            assigned: Utc::now(),
            mirrored: HashMap::new(),
        }
    }
}

type KernelSymbols = HashMap<String, SymbolInfo>;

#[derive(Debug, Clone, Serialize)]
pub struct TaskInfo {
    /// The unique number for the task within the [`KernelSpace`]
    ///
    /// An easier way to be able to refer to a task than by its [`TaskId`].
    pub num: u64,

    /// The code that was executed
    pub code: String,

    /// The symbols that are used by this code
    pub symbols_used: Vec<Symbol>,

    /// The symbols that are assigned by this code
    pub symbols_assigned: Vec<Symbol>,

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
}

/// A list of [`Task`]s associated with a [`KernelSpace`]
#[derive(Debug, Default)]
struct KernelTasks {
    /// The list of tasks
    inner: Vec<TaskInfo>,

    /// A counter to be able to assign unique numbers to tasks
    counter: u64,
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
    fn get_mut<'lt>(&'lt mut self, task_id: &TaskId) -> Option<&'lt mut TaskInfo> {
        for task_info in self.inner.iter_mut() {
            if task_info.task.id == *task_id {
                return Some(task_info);
            }
        }
        None
    }

    /// Put a task onto the list
    #[allow(clippy::too_many_arguments)]
    async fn put(
        &mut self,
        task: &Task,
        code: &str,
        symbols_used: &[Symbol],
        symbols_assigned: &[Symbol],
        kernel_id: &str,
        is_fork: bool,
        is_deferred: bool,
    ) -> TaskInfo {
        self.counter += 1;

        let task_info = TaskInfo {
            num: self.counter,
            code: code.to_string(),
            symbols_used: symbols_used.into(),
            symbols_assigned: symbols_assigned.into(),
            kernel_id: Some(kernel_id.to_string()),
            is_deferred,
            is_fork,
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
    async fn display(&self) -> cli_utils::Result {
        use cli_utils::result;

        result::value(self.inner.clone())
    }
}

type KernelQueues = HashMap<KernelId, VecDeque<TaskId>>;

#[derive(Debug, Default, Serialize)]
pub struct KernelSpace {
    /// The kernels in the kernel space
    #[serde(skip)]
    kernels: Arc<Mutex<KernelMap>>,

    /// The symbols in the kernel space
    #[serde(skip)]
    symbols: Arc<Mutex<KernelSymbols>>,

    /// The list of all tasks sent to this kernel space
    #[serde(skip)]
    tasks: Arc<Mutex<KernelTasks>>,

    /// The queue of deferred tasks
    #[serde(skip)]
    queues: Arc<Mutex<KernelQueues>>,
}

impl KernelSpace {
    /// Create a new kernel space
    pub fn new() -> Self {
        let new = Self::default();

        let kernels = new.kernels.clone();
        let queue = new.queues.clone();
        let tasks = new.tasks.clone();
        let symbols = new.symbols.clone();
        tokio::spawn(async move { KernelSpace::monitor(&kernels, &queue, &tasks, &symbols).await });

        new
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
        use tokio::time::{sleep, Duration};

        tracing::debug!("Began kernel space monitoring");
        loop {
            KernelSpace::dispatch_queue(queues, tasks, kernels, symbols).await;

            sleep(Duration::from_millis(300)).await;
        }
    }

    /// Get a symbol from the kernel space
    pub async fn get(&mut self, name: &str) -> Result<Node> {
        let symbols = &mut *self.symbols.lock().await;
        let symbol_info = symbols
            .get(name)
            .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

        let kernels = &mut *self.kernels.lock().await;
        let kernel = kernels.get_mut(&symbol_info.home)?;
        kernel.get(name).await
    }

    /// Set a symbol in the kernel space
    pub async fn set(&mut self, name: &str, value: Node, language: &str) -> Result<()> {
        let selector = KernelSelector::parse(language);

        let kernels = &mut *self.kernels.lock().await;

        let kernel_id = kernels.ensure(&selector).await?;
        tracing::debug!("Setting symbol `{}` in kernel `{}`", name, kernel_id);

        let kernel = kernels.get_mut(&kernel_id)?;
        kernel.set(name, value).await?;

        let symbols = &mut *self.symbols.lock().await;
        match symbols.entry(name.to_string()) {
            Entry::Occupied(mut occupied) => {
                let info = occupied.get_mut();
                info.home = kernel_id;
                info.assigned = Utc::now();
            }
            Entry::Vacant(vacant) => {
                vacant.insert(SymbolInfo::new("", &kernel_id));
            }
        }

        Ok(())
    }

    /// Execute some code in the kernel space
    pub async fn exec(
        &mut self,
        code: &str,
        selector: &KernelSelector,
        relations: Option<Vec<(Relation, Resource)>>,
        is_async: bool,
        is_fork: bool,
    ) -> Result<TaskInfo> {
        let kernels = &mut *self.kernels.lock().await;

        // Summarize relations into symbols used and assigned
        let mut symbols_used = Vec::new();
        let mut symbols_assigned = Vec::new();
        if let Some(relations) = relations {
            for pair in relations {
                match pair {
                    (Relation::Use(..), Resource::Symbol(symbol)) => symbols_used.push(symbol),
                    (Relation::Assign(..), Resource::Symbol(symbol)) => {
                        symbols_assigned.push(symbol)
                    }
                    _ => (),
                }
            }
        }

        // Determine the kernel to execute in
        let kernel_id = kernels.ensure(selector).await?;
        tracing::debug!("Executing code in kernel `{}`", kernel_id);

        // If the kernel is busy then defer the task, otherwise dispatch to the kernel now
        let kernel = kernels.get(&kernel_id)?;
        let (deferred, task) = if kernel.is_busy().await? {
            let task = self.defer_task(&kernel_id).await;
            (true, task)
        } else {
            let symbols = &mut *self.symbols.lock().await;
            let task = KernelSpace::dispatch_task(
                code,
                &symbols_used,
                &symbols_assigned,
                is_async,
                is_fork,
                symbols,
                &kernel_id,
                kernels,
            )
            .await?;
            (false, task)
        };

        // Either way, store the task
        let task_info = self
            .store(
                &task,
                code,
                &symbols_used,
                &symbols_assigned,
                is_fork,
                &kernel_id,
                deferred,
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
        symbols_used: &[Symbol],
        symbols_assigned: &[Symbol],
        is_async: bool,
        is_fork: bool,
        symbols: &mut KernelSymbols,
        kernel_id: &str,
        kernels: &mut KernelMap,
    ) -> Result<Task> {
        // Mirror used symbols into the kernel
        for symbol in symbols_used {
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
                if mirrored >= &symbol.assigned {
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
        let kernel = kernels.get_mut(kernel_id)?;
        let task = if is_fork {
            kernel.exec_fork(code).await?
        } else if is_async {
            kernel.exec_async(code).await?
        } else {
            kernel.exec_sync(code).await?
        };

        // Record symbols assigned in kernel (unless it was a fork)
        if !is_fork {
            for symbol in symbols_assigned {
                symbols
                    .entry(symbol.name.clone())
                    .and_modify(|info| {
                        info.home = kernel_id.to_string();
                        info.assigned = Utc::now();
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
    async fn defer_task(&self, kernel_id: &str) -> Task {
        let (sender, ..) = broadcast::channel(1);
        let (canceller, mut cancellee) = mpsc::channel(1);
        let task = Task::create(Some(sender), Some(canceller));

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

    /// Dispatch tasks of a kernel space queues
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

                    if let Some(mut task_info) = tasks.get_mut(&task_id) {
                        tracing::debug!("Dispatching task `{}` to kernel `{}`", task_id, kernel_id);
                        if let Err(error) = KernelSpace::dispatch_deferred(
                            &mut task_info,
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
        let deferred_task = &mut task_info.task;
        let task_id = deferred_task.id.clone();

        // Dispatch the task
        let started_task = KernelSpace::dispatch_task(
            &task_info.code,
            &task_info.symbols_used,
            &task_info.symbols_assigned,
            task_info.is_async,
            task_info.is_fork,
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
        symbols_used: &[Symbol],
        symbols_assigned: &[Symbol],
        is_fork: bool,
        kernel_id: &str,
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
            .put(
                task,
                code,
                symbols_used,
                symbols_assigned,
                kernel_id,
                is_fork,
                is_deferred,
            )
            .await
    }

    /// Cancel a task
    async fn cancel(&mut self, task_num_or_id: &str) -> Result<()> {
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

    /// A read-evaluate-print function
    ///
    /// Primarily intended for use in interactive mode as an execution REPL.
    /// Adds execution related shortcuts e.g. `%symbols` for changing the language.
    #[cfg(feature = "cli")]
    pub async fn repl(
        &mut self,
        code: &str,
        language: Option<String>,
        kernel: Option<String>,
    ) -> cli_utils::Result {
        use cli_utils::result;
        use events::{subscribe, unsubscribe, Subscriber};
        use once_cell::sync::Lazy;
        use regex::Regex;

        static SYMBOL: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^\w+$").expect("Unable to create regex"));

        if code.is_empty() {
            result::nothing()
        } else if code == "%kernels" {
            let kernels = self.kernels.lock().await;
            result::value(kernels.display().await)
        } else if code == "%symbols" {
            let symbols = self.symbols.lock().await;
            result::value(symbols.clone())
        } else if code == "%tasks" {
            let tasks = self.tasks.lock().await;
            tasks.display().await
        } else if code == "%queues" {
            let queue = self.queues.lock().await;
            result::value(queue.clone())
        } else if let Some(task_num_or_id) = code.strip_prefix("%cancel") {
            let task_num_or_id = task_num_or_id.trim();
            let task_num_or_id = task_num_or_id.strip_prefix('#').unwrap_or(task_num_or_id);
            self.cancel(task_num_or_id).await?;
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
            let (background, code) = if code.contains("@back") {
                (true, code.replace("@back", ""))
            } else {
                (false, code.to_string())
            };
            let (fork, code) = if code.contains("@fork") {
                (true, code.replace("@fork", ""))
            } else {
                (false, code)
            };

            let code = code.trim().replace("\\n", "\n");

            let language = language.unwrap_or_else(|| "calc".to_string());

            // If possible, parse the code so that we can use the relations to determine variables that
            // are assigned or used (needed for variable mirroring).
            let relations = match parsers::parse("<cli>", &code, &language) {
                Ok(pairs) => pairs,
                Err(..) => Vec::new(),
            };

            // Determine the kernel selector
            let selector = match kernel {
                Some(kernel) => {
                    let mut selector = KernelSelector::parse(&kernel);
                    selector.lang = Some(language);
                    selector
                }
                None => KernelSelector::new(None, Some(language), None),
            };

            // Execute the code
            let mut task_info = self
                .exec(&code, &selector, Some(relations), true, fork)
                .await?;

            if background {
                // Indicate task is running in background
                tracing::info!("Task #{} is running in background", task_info.num);
                result::nothing()
            } else {
                // If cancellable, subscribe to user interrupt
                let subscription_id = if let Some(canceller) = task_info.task.canceller.as_ref() {
                    let canceller = canceller.clone();
                    let (sender, mut receiver) = mpsc::unbounded_channel();
                    tokio::spawn(async move {
                        // This ends on interrupt or on unsubscribe (when the channel sender is dropped)
                        if let Some(..) = receiver.recv().await {
                            if let Err(error) = canceller.send(()).await {
                                tracing::error!(
                                    "While cancelling task when interrupted: {}",
                                    error
                                );
                            }
                        }
                    });

                    let subscription_id = subscribe("interrupt", Subscriber::Sender(sender))?;
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
#[allow(clippy::vec_init_then_push)]
pub async fn available() -> Result<Vec<Kernel>> {
    let mut available: Vec<Kernel> = Vec::new();

    #[cfg(feature = "kernel-calc")]
    available.push(kernel_calc::CalcKernel::new().spec());

    #[cfg(feature = "kernel-jupyter")]
    available.append(&mut kernel_jupyter::JupyterKernel::available().await?);

    macro_rules! microkernel_available {
        ($feat:literal, $crat:ident, $list:expr) => {
            #[cfg(feature = $feat)]
            {
                let kernel = $crat::new();
                if kernel.is_available().await {
                    $list.push(kernel.spec())
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
            languages.insert(lang);
        }
    }
    let mut languages: Vec<String> = languages.into_iter().collect();
    languages.sort();
    Ok(languages)
}

/// List the kernels (and servers) that are currently running on this machine
pub async fn running() -> Result<serde_json::Value> {
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
pub async fn directories() -> Result<serde_json::Value> {
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

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use cli_utils::{result, Result, Run};
    use once_cell::sync::Lazy;
    use structopt::StructOpt;
    use tokio::sync::Mutex;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage kernels",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Command {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        Available(Available),
        Languages(Languages),
        Running(Running),
        Directories(Directories),
        Execute(Execute),
        Start(Start),
        Connect(Connect),
        Stop(Stop),
        Status(Status),
        Show(Show),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::Available(action) => action.run().await,
                Action::Languages(action) => action.run().await,
                Action::Running(action) => action.run().await,
                Action::Directories(action) => action.run().await,
                Action::Execute(action) => action.run().await,
                Action::Start(action) => action.run().await,
                Action::Connect(action) => action.run().await,
                Action::Stop(action) => action.run().await,
                Action::Status(action) => action.run().await,
                Action::Show(action) => action.run().await,
            }
        }
    }

    /// List the kernels that are available on this machine
    ///
    /// The list of available kernels includes those that are built into the Stencila
    /// binary (e.g. `calc`), Jupyter kernels installed on the machine, and Microkernels
    /// for which a supporting runtime (e.g. `python`) is installed.
    #[derive(Debug, StructOpt)]
    #[structopt(
        alias = "avail",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
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
    #[derive(Debug, StructOpt)]
    #[structopt(
        alias = "langs",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Languages {}
    #[async_trait]
    impl Run for Languages {
        async fn run(&self) -> Result {
            result::value(languages().await?)
        }
    }

    /// List the Jupyter kernels (and servers) that are currently running
    ///
    /// This command scans the Jupyter `runtime` directory to get a list of running
    /// Jupyter notebook servers. It then gets a list of kernels from the REST API
    /// of each of those servers.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Running {}
    #[async_trait]
    impl Run for Running {
        async fn run(&self) -> Result {
            result::value(running().await?)
        }
    }

    /// List the directories on this machine that will be searched for Jupyter kernel specs
    /// and running kernels
    #[derive(Debug, StructOpt)]
    #[structopt(
        alias = "dirs",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Directories {}
    #[async_trait]
    impl Run for Directories {
        async fn run(&self) -> Result {
            result::value(directories().await?)
        }
    }

    /// A lazily initialized kernel space for the execute command. Required so that
    /// kernel state is maintained in successive calls to `Execute::run` when in
    /// interactive mode
    static KERNEL_SPACE: Lazy<Mutex<KernelSpace>> = Lazy::new(|| Mutex::new(KernelSpace::new()));

    /// Execute code within a temporary "kernel space"
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
    #[derive(Debug, StructOpt)]
    #[structopt(
        alias = "exec",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Execute {
        /// Code to execute within the document's kernel space
        // Using a Vec and the `multiple` option allows for spaces in the code
        #[structopt(multiple = true)]
        code: Vec<String>,

        /// The name of the programming language
        #[structopt(short, long)]
        lang: Option<String>,

        /// The kernel where the code should executed (a kernel selector string)
        #[structopt(short, long)]
        kernel: Option<String>,
    }
    #[async_trait]
    impl Run for Execute {
        async fn run(&self) -> Result {
            KERNEL_SPACE
                .lock()
                .await
                .repl(&self.code.join(" "), self.lang.clone(), self.kernel.clone())
                .await
        }
    }

    /// Start a kernel
    ///
    /// Mainly intended for testing that kernels that rely on external files or processes
    /// (i.e. a Jupyter kernel or a Microkernel) can be started successfully.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Start {
        /// The name or programming language of the kernel
        selector: String,
    }
    #[async_trait]
    impl Run for Start {
        async fn run(&self) -> Result {
            let kernel_space = KERNEL_SPACE.lock().await;
            let mut kernels = kernel_space.kernels.lock().await;
            let selector = KernelSelector::parse(&self.selector);
            let kernel_id = kernels.start(&selector).await?;
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
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Stop {
        /// The id of the kernel (see `kernels status`)
        id: String,
    }
    #[async_trait]
    impl Run for Stop {
        async fn run(&self) -> Result {
            let kernel_space = KERNEL_SPACE.lock().await;
            let mut kernels = kernel_space.kernels.lock().await;
            kernels.stop(&self.id).await?;
            tracing::info!("Stopped kernel `{}`", self.id);
            result::nothing()
        }
    }

    /// Connect to a running kernel
    ///
    /// Mainly intended for testing that Stencila is able to connect
    /// to an existing kernel (e.g. one that was started from Jupyter notebook).
    ///
    /// To get a list of externally started kernels that can be connected to run,
    ///
    /// > kernels running
    ///
    /// and then connect to a kernel using its Jupyter id e.g.,
    ///
    /// > kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
    ///
    /// Alternatively, use the path (relative or absolute) of the Jupyter notebook
    /// whose (already started) kernel you wish to connect to e.g.,
    ///
    /// > kernels connect ../main.ipynb
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Connect {
        /// The id of the kernel e.g. `31248fc2-38d0-4d11-80a1-f8a1bd3842fb`
        /// or the relative path of the notebook
        id_or_path: String,
    }
    impl Connect {
        pub async fn run(&self) -> Result {
            let kernel_space = KERNEL_SPACE.lock().await;
            let mut kernels = kernel_space.kernels.lock().await;
            let id = kernels.connect(&self.id_or_path).await?;
            tracing::info!("Connected to kernel `{}`", id);
            result::nothing()
        }
    }

    /// Get a list of the kernels in the current kernel space
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
    /// To get a list of externally started kernels that can be connected to run,
    ///
    /// > kernels running
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Status {}
    impl Status {
        pub async fn run(&self) -> Result {
            let kernel_space = KERNEL_SPACE.lock().await;
            let kernels = kernel_space.kernels.lock().await;
            result::value(kernels.display().await)
        }
    }

    /// Show the details of a current kernel
    ///
    /// Mainly intended for interactive mode testing / inspection.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The id of the kernel (see `kernels status`)
        id: KernelId,
    }
    impl Show {
        pub async fn run(&self) -> Result {
            let kernel_space = KERNEL_SPACE.lock().await;
            let kernels = kernel_space.kernels.lock().await;
            let kernel = kernels.get(&self.id)?;
            result::value(kernel)
        }
    }
}
