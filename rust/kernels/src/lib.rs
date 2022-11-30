use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap, HashSet},
    env::current_dir,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use common::{itertools::Itertools, once_cell::sync::Lazy, strum::AsRefStr, tokio::sync::RwLock};
use formats::Format;
use kernel::parser::ParseInfo;
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
    KernelId, KernelInfo, KernelStatus, KernelTrait, TagMap, TaskId, TaskMessages, TaskOutputs,
};

// Re-exports
pub use kernel::{Kernel, KernelSelector, KernelType, Task, TaskResult};

/// A "meta" kernel to dispatch to different types of kernels
///
/// In the future this maybe changed to, or augmented with a `Box<dyn KernelTrait>`,
/// to allow dispatching to plugins that are dynamically added at runtime.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, AsRefStr)]
#[serde(crate = "common::serde")]
enum MetaKernel {
    #[cfg(feature = "kernel-store")]
    Store(kernel_store::StoreKernel),

    #[cfg(feature = "kernel-calc")]
    Calc(kernel_calc::CalcKernel),

    #[cfg(feature = "kernel-http")]
    Http(kernel_http::HttpKernel),

    #[cfg(feature = "kernel-postgrest")]
    Postgrest(kernel_postgrest::PostgrestKernel),

    #[cfg(feature = "kernel-tailwind")]
    Tailwind(kernel_tailwind::TailwindKernel),

    #[cfg(feature = "kernel-sql")]
    Sql(kernel_sql::SqlKernel),

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

        matches_kernel!(
            "kernel-http",
            MetaKernel::Http,
            kernel_http::HttpKernel::new()
        );

        matches_kernel!(
            "kernel-postgrest",
            MetaKernel::Postgrest,
            kernel_postgrest::PostgrestKernel::new()
        );

        matches_kernel!(
            "kernel-tailwind",
            MetaKernel::Tailwind,
            kernel_tailwind::TailwindKernel::new()
        );

        matches_kernel!(
            "kernel-sql",
            MetaKernel::Sql,
            kernel_sql::SqlKernel::new(selector)
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

    /// Fork a `MetaKernel` instance
    ///
    /// Returns the new `MetaKernel` and a boolean indicating whether the kernel is a
    /// clone of the parent.
    ///
    /// This is easier to done here, not in KernelTrait, given that the returned kernel needs
    /// to be the same variant as its ancestor.
    async fn fork(&self) -> Result<(MetaKernel, bool)> {
        match self {
            #[cfg(feature = "kernel-store")]
            MetaKernel::Store(kernel) => Ok((MetaKernel::Store(kernel.clone()), true)),

            #[cfg(feature = "kernel-calc")]
            MetaKernel::Calc(kernel) => Ok((MetaKernel::Calc(kernel.clone()), true)),

            #[cfg(feature = "kernel-http")]
            MetaKernel::Http(kernel) => Ok((MetaKernel::Http(kernel.clone()), true)),

            #[cfg(feature = "kernel-postgrest")]
            MetaKernel::Postgrest(kernel) => Ok((MetaKernel::Postgrest(kernel.clone()), true)),

            #[cfg(feature = "kernel-tailwind")]
            MetaKernel::Tailwind(kernel) => Ok((MetaKernel::Tailwind(kernel.clone()), true)),

            #[cfg(feature = "kernel-sql")]
            MetaKernel::Sql(kernel) => Ok((MetaKernel::Sql(kernel.clone()), true)),

            #[cfg(feature = "kernel-micro")]
            MetaKernel::Micro(kernel) => {
                let (kernel, is_clone) = if kernel.is_forkable().await {
                    (kernel.create_fork("").await?, true)
                } else {
                    (kernel.create_knife().await?, false)
                };
                Ok((MetaKernel::Micro(kernel), is_clone))
            }

            #[cfg(feature = "kernel-jupyter")]
            MetaKernel::Jupyter(_) => {
                bail!("Doing a `MetaKernel::fork` is not yet implemented for Jupyter kernels",)
            }
        }
    }
}

macro_rules! dispatch_variants {
    ($var:expr, $method:ident $(,$arg:expr)*) => {
        match $var {
            #[cfg(feature = "kernel-store")]
            MetaKernel::Store(kernel) => kernel.$method($($arg),*),

            #[cfg(feature = "kernel-calc")]
            MetaKernel::Calc(kernel) => kernel.$method($($arg),*),

            #[cfg(feature = "kernel-http")]
            MetaKernel::Http(kernel) => kernel.$method($($arg),*),

            #[cfg(feature = "kernel-postgrest")]
            MetaKernel::Postgrest(kernel) => kernel.$method($($arg),*),

            #[cfg(feature = "kernel-tailwind")]
            MetaKernel::Tailwind(kernel) => kernel.$method($($arg),*),

            #[cfg(feature = "kernel-sql")]
            MetaKernel::Sql(kernel) => kernel.$method($($arg),*),

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

    async fn start(&mut self, directory: &Path) -> Result<()> {
        dispatch_variants!(self, start, directory).await
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

    async fn derive(&mut self, what: &str, from: &str) -> Result<Vec<Node>> {
        dispatch_variants!(self, derive, what, from).await
    }

    async fn exec(
        &mut self,
        code: &str,
        lang: Format,
        tags: Option<&TagMap>,
    ) -> Result<(TaskOutputs, TaskMessages)> {
        dispatch_variants!(self, exec, code, lang, tags).await
    }

    async fn exec_sync(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task> {
        dispatch_variants!(self, exec_sync, code, lang, tags).await
    }

    async fn exec_async(
        &mut self,
        code: &str,
        lang: Format,
        tags: Option<&TagMap>,
    ) -> Result<Task> {
        dispatch_variants!(self, exec_async, code, lang, tags).await
    }

    async fn exec_fork(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task> {
        dispatch_variants!(self, exec_fork, code, lang, tags).await
    }
}

/// A map of kernel ids to kernels.
#[derive(Debug, Default, Deref, DerefMut, Serialize)]
#[serde(crate = "common::serde")]
struct KernelMap(BTreeMap<KernelId, (KernelSelector, MetaKernel)>);

impl KernelMap {
    /// Get a reference to a kernel
    fn get(&self, kernel_id: &str) -> Result<&MetaKernel> {
        (**self)
            .get(kernel_id)
            .map(|(.., kernel)| kernel)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Get a mutable reference to a kernel
    fn get_mut(&mut self, kernel_id: &str) -> Result<&mut MetaKernel> {
        (**self)
            .get_mut(kernel_id)
            .map(|(.., kernel)| kernel)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Ensure that a kernel exists for a selector
    ///
    /// Returns the kernel's id.
    async fn ensure(
        &mut self,
        desired_selector: &KernelSelector,
        directory: &Path,
    ) -> Result<KernelId> {
        tracing::trace!("Ensuring kernel matching selector `{}`", desired_selector);

        // Is there already a running kernel that matches the selector?
        for (kernel_id, (existing_selector, kernel)) in self.iter_mut() {
            let mut matched = false;
            // If id is specified in selector, this takes precedence
            if let Some(id) = &desired_selector.id {
                matched = id == kernel_id;
            }
            // Otherwise, if config is specified then matches if the selector is the same as the
            // existing kernel
            else if desired_selector.config.is_some() {
                matched = desired_selector == existing_selector;
            }
            // Finally, if the selector does not specify `id` or `config` then match lang etc
            // against the spec
            else if desired_selector.matches(&kernel.spec().await) {
                matched = true;
            }
            if !matched {
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
                | KernelStatus::Ready
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
        self.start(desired_selector, directory).await
    }

    /// Start a kernel for a selector
    async fn start(&mut self, selector: &KernelSelector, directory: &Path) -> Result<KernelId> {
        tracing::trace!("Starting kernel matching selector `{}`", selector);

        let mut kernel = MetaKernel::new(selector).await?;
        kernel.start(directory).await?;

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

        self.insert(kernel_id.clone(), (selector.clone(), kernel));

        Ok(kernel_id)
    }

    /// Stop one of the kernels and remove it from the kernel space
    async fn stop(&mut self, id: &str) -> Result<()> {
        tracing::trace!("Stopping kernel `{}`", id);

        self.get_mut(id)?.stop().await?;
        self.remove(id);
        Ok(())
    }

    /// Connect to a running kernel
    #[allow(unused_variables)]
    async fn connect(&mut self, id_or_path: &str) -> Result<KernelId> {
        tracing::trace!("Connecting to kernel `{}`", id_or_path);

        #[cfg(feature = "kernel-jupyter")]
        {
            let (kernel_id, kernel) = kernel_jupyter::JupyterKernel::connect(id_or_path).await?;
            self.insert(
                kernel_id.clone(),
                (KernelSelector::default(), MetaKernel::Jupyter(kernel)),
            );

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
        for (id, (.., kernel)) in self.iter() {
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
                    info.spec
                        .languages
                        .iter()
                        .map(|format| format.spec().title)
                        .join(", "),
                    info.interruptable,
                    info.forkable
                )
            })
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
    kind: Option<String>,

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
    pub fn new(kind: Option<String>, kernel_id: &str) -> Self {
        SymbolInfo {
            kind,
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
                symbol_info.kind.clone().unwrap_or_default(),
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
    pub parse_info: ParseInfo,

    /// The id of the kernel that the task was dispatched to
    pub kernel_id: Option<String>,

    /// Whether the task has been scheduled to run in a fork of the kernel
    pub is_fork: bool,

    /// Whether the task is asynchronous
    pub is_async: bool,

    /// Whether the task can be interrupted
    pub is_interruptable: bool,

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

    /// Was the task interrupted?
    pub fn was_interrupted(&self) -> bool {
        self.task.interrupted.is_some()
    }

    /// Time that the the task ended (either finished, or interrupted)
    pub fn ended(&self) -> Option<DateTime<Utc>> {
        self.task.finished.or(self.task.interrupted)
    }

    /// Calculate the task duration in microseconds
    pub fn duration(&self) -> Option<u64> {
        let duration = if let (Some(began), Some(ended)) = (self.task.started, self.ended()) {
            Some(ended - began)
        } else {
            self.task.started.map(|started| (Utc::now() - started))
        };
        duration.map(|duration| duration.to_std().unwrap_or(Duration::ZERO).as_micros() as u64)
    }
}

/// A list of [`Task`]s associated with a [`KernelSpace`]
///
/// The main purpose of maintaining this list of tasks is observability.
/// To avoid the list growing indefinitely, tasks are removed on a periodic basis,
/// if the list is greater than a certain size.
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
    /// Sort by time interrupted
    Interrupted,
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
    async fn put(
        &mut self,
        task: &Task,
        code: &str,
        parse_info: &ParseInfo,
        kernel_id: &str,
        is_fork: bool,
    ) -> TaskInfo {
        self.counter += 1;

        let task_info = TaskInfo {
            num: self.counter,
            code: code.to_string(),
            parse_info: parse_info.clone(),
            kernel_id: Some(kernel_id.to_string()),
            is_fork,
            is_async: task.is_async(),
            is_interruptable: task.is_interruptable(),
            task: task.clone(),
        };

        self.inner.push(task_info.clone());

        task_info
    }

    /// Display the tasks
    #[cfg(feature = "cli")]
    async fn display(
        &self,
        num: usize,
        sort: &KernelTaskSorting,
        desc: bool,
        kernel: Option<KernelId>,
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
            &KernelTaskSorting::Interrupted => {
                list.sort_by(|a, b| a.task.interrupted.cmp(&b.task.interrupted))
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
            "|-|-------|-------|-----------|--------|--------|------|------|-------------|----|";
        let head =
            "|#|Created|Started|Interrupted|Finished|Duration|Kernel|Forked|Interruptable|Code|";
        let align =
            "|-|------:|------:|----------:|-------:|-------:|:-----|-----:|------------:|:---|";
        let body = list
            .iter()
            .map(|task_info| {
                let task = &task_info.task;

                let kernel_id = task_info.kernel_id.clone().unwrap_or_default();

                let fork = if task_info.is_fork { "yes" } else { "no" };

                let interruptable = if task_info.is_interruptable {
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
                    "|{}|{}|{}|{}|{}|{}|{}|{}|{}|`{}`|",
                    task_info.num,
                    format_time(task.created),
                    task.started.map(format_time).unwrap_or_default(),
                    task.interrupted.map(format_time).unwrap_or_default(),
                    task.finished.map(format_time).unwrap_or_default(),
                    format_duration(task_info.duration()),
                    kernel_id,
                    fork,
                    interruptable,
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

#[derive(Debug, Default)]
pub struct KernelSpace {
    /// The working directory of the kernel space
    directory: PathBuf,

    /// The kernels in the kernel space
    kernels: Arc<Mutex<KernelMap>>,

    /// The symbols in the kernel space
    symbols: Arc<Mutex<KernelSymbols>>,

    /// The list of all tasks sent to this kernel space
    tasks: Arc<Mutex<KernelTasks>>,

    /// The monitoring task for the kernel space
    monitoring: Option<JoinHandle<()>>,
}

impl Drop for KernelSpace {
    fn drop(&mut self) {
        if let Some(monitoring) = &self.monitoring {
            tracing::trace!("Ending kernel space monitoring");
            monitoring.abort()
        }
    }
}

pub type KernelInfos = HashMap<KernelId, KernelInfo>;

impl KernelSpace {
    /// Create a new kernel space and start its monitoring task
    pub fn new(directory: Option<&Path>) -> Self {
        let mut new = Self::default();
        new.directory = directory
            .map(PathBuf::from)
            .unwrap_or_else(|| current_dir().expect("Should be able to get current dir"));

        new.monitor();
        new
    }

    /// Fork a kernel space
    ///
    /// A clone is made of each kernel in `kernels`. If the kernel is forkable, then the clone will be
    /// a forked process. Otherwise, it will be a new process.
    ///
    /// The symbols in this kernel space are also cloned but are then modified to that `mirrored`
    /// id removed for new kernels that are not processes forks and therefore do not have the
    /// value mirrored in them.
    ///
    /// Returns a new `KernelSpace` and a list of the names of the kernels in that kernel space
    /// that were restarted.
    pub async fn fork(&self) -> Result<(Self, Vec<String>)> {
        let mut new = Self::default();
        new.directory = self.directory.clone();

        // Fork each of the kernels and remove any `mirrored` entry for the kernel if unable to do
        // a proper fork.
        let mut new_kernels = KernelMap::default();
        let mut new_symbols = self.symbols.lock().await.clone();
        let mut kernels_restarted = Vec::new();
        for (kernel_name, (selector, kernel)) in self.kernels.lock().await.iter() {
            // Create the new kernel
            let (new_kernel, is_fork) = kernel.fork().await?;
            new_kernels.insert(kernel_name.clone(), (selector.clone(), new_kernel));

            // Special handling for those that are not true forks with identical state.
            if !is_fork {
                // Remove any symbols that
                new_symbols.retain(|_symbol, symbol_info| symbol_info.home != *kernel_name);
                // Remove any symbol `mirrored` entries for the kernel
                for (_symbol, symbol_info) in new_symbols.iter_mut() {
                    symbol_info
                        .mirrored
                        .retain(|mirrored_in, _| mirrored_in != kernel_name);
                }
                // Add to list of kernels that were restarted
                kernels_restarted.push(kernel_name.to_owned());
            }
        }
        new.kernels = Arc::new(Mutex::new(new_kernels));
        new.symbols = Arc::new(Mutex::new(new_symbols));

        new.monitor();
        Ok((new, kernels_restarted))
    }

    /// Monitor the kernel space
    ///
    /// Monitors the health of kernels and cleans up the task list to
    /// avoid it growing too large.
    fn monitor(&mut self) {
        const PERIOD: Duration = Duration::from_millis(1000);

        let tasks = self.tasks.clone();
        self.monitoring = Some(tokio::spawn(async move {
            tracing::trace!("Beginning kernel space monitoring");
            loop {
                KernelSpace::clean(&tasks).await;
                tokio::time::sleep(PERIOD).await;
            }
        }));
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

    /// Guess the language of some code based on syntax and symbols used
    ///
    /// Attempts to parse the code using each parser and will return the language
    /// which parses successfully and which has the most number of symbols (involved
    /// in relations) that are resident in the corresponding kernel.
    pub fn guess_language(
        &self,
        code: &str,
        fallback: Format,
        include: Option<&[Format]>,
        exclude: Option<&[Format]>,
    ) -> Format {
        // Languages in order of increasing permissiveness of parsers
        let include = include.unwrap_or(&[
            Format::Json5,
            Format::Json,
            Format::PrQL,
            Format::SQL,
            Format::Calc,
            Format::JavaScript,
            Format::Python,
            Format::Tailwind,
            Format::R, // The R parser seems to be very permissive (generates few (no?) errors)
            Format::Bash,
            Format::Zsh,
        ]);
        let exclude = exclude.map_or_else(Vec::new, Vec::from);
        let alternatives = include.iter().filter(|lang| !exclude.contains(lang));

        if code.is_empty() {
            return fallback;
        }

        // TODO check against existing kernels and variables in them
        for language in alternatives {
            if let Ok(parse_info) = parsers::parse(*language, code, None) {
                if !parse_info.syntax_errors {
                    return *language;
                }
            }
        }

        fallback
    }

    /// Start a kernel
    pub async fn start(&self, selector: &KernelSelector) -> Result<KernelId> {
        let kernels = &mut *self.kernels.lock().await;
        kernels.start(selector, &self.directory).await
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
            let selector = KernelSelector {
                name: Some(spec.name),
                ..Default::default()
            };

            kernels.stop(id).await?;
            purge_kernel_from_symbols(symbols, id);
            kernels.start(&selector, &self.directory).await?;
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
    pub async fn set(
        &self,
        name: &str,
        value: Node,
        selector: &KernelSelector,
    ) -> Result<KernelId> {
        let kernels = &mut *self.kernels.lock().await;

        let kernel_id = kernels.ensure(selector, &self.directory).await?;
        tracing::debug!("Setting symbol `{}` in kernel `{}`", name, kernel_id);

        let kernel = kernels.get_mut(&kernel_id)?;
        kernel.set(name, value).await?;

        let symbols = &mut *self.symbols.lock().await;
        match symbols.entry(name.to_string()) {
            Entry::Occupied(mut occupied) => {
                let info = occupied.get_mut();
                info.home = kernel_id.clone();
                info.modified = Utc::now();
            }
            Entry::Vacant(vacant) => {
                vacant.insert(SymbolInfo::new(None, &kernel_id));
            }
        }

        Ok(kernel_id)
    }

    /// Derive one or more nodes from a symbol in the kernel space
    ///
    /// Determine's the home kernel for the `from` symbol and dispatches a `derive()`
    /// call to that kernel.
    ///
    /// Returns a tuple of the kernel id and the derived nodes.
    pub async fn derive(&self, what: &str, from: &str) -> Result<(String, Vec<Node>)> {
        tracing::trace!("Deriving `{what}` from `{from}`");

        let parts: Vec<&str> = from.splitn(2, '.').collect();
        let symbol = parts[0];
        let symbols = &mut *self.symbols.lock().await;
        let symbol_info = symbols
            .get(symbol)
            .ok_or_else(|| eyre!("Unknown symbol `{}`. Perhaps it needs to be declared, or the code that assigns it needs to be executed?", symbol))?;
        let kernel_id = symbol_info.home.clone();

        let kernels = &mut *self.kernels.lock().await;
        let kernel = kernels.get_mut(&kernel_id)?;

        let nodes = kernel.derive(what, from).await?;
        Ok((kernel_id, nodes))
    }

    /// Execute some code in the kernel space
    pub async fn exec(
        &self,
        code: &str,
        parse_info: &ParseInfo,
        force_fork: bool,
        selector: &KernelSelector,
    ) -> Result<TaskInfo> {
        let kernels = &mut *self.kernels.lock().await;

        // Determine the kernel to execute in
        let kernel_id = kernels.ensure(selector, &self.directory).await?;
        tracing::trace!("Dispatching task to kernel `{}`", kernel_id);

        // Mirror symbols that are used in the code into the kernel
        let symbols = &mut *self.symbols.lock().await;
        for (name, ..) in parse_info.variables_used() {
            let symbol = match symbols.get_mut(&name) {
                Some(symbol) => symbol,
                // Skip if unknown symbol (e.g a package, or variable assigned elsewhere)
                None => continue,
            };

            // Skip if home is the target kernel
            if symbol.home == *kernel_id {
                continue;
            }

            // Skip if already mirrored since last assigned
            if let Some(mirrored) = symbol.mirrored.get(&kernel_id) {
                if mirrored >= &symbol.modified {
                    continue;
                }
            }

            tracing::trace!(
                "Mirroring symbol `{}` from kernel `{}` to kernel `{}`",
                name,
                symbol.home,
                kernel_id
            );

            let home_kernel = kernels.get_mut(&symbol.home)?;
            let value = home_kernel.get(&name).await?;

            let mirror_kernel = kernels.get_mut(&kernel_id)?;
            mirror_kernel.set(&name, value).await?;

            symbol
                .mirrored
                .entry(kernel_id.to_string())
                .and_modify(|datetime| *datetime = Utc::now())
                .or_insert_with(Utc::now);
        }

        // Execute the code in the kernel, or a fork
        let kernel = kernels.get_mut(&kernel_id)?;
        let lang = parse_info.language();
        let pure = parse_info.is_pure();
        let fork = force_fork || (pure && kernel.is_forkable().await);

        tracing::trace!(
            "Executing code for in kernel `{}`{}",
            kernel_id,
            if fork { " fork" } else { "" }
        );
        let tags = parse_info.tag_map();
        let task = if fork {
            kernel.exec_fork(code, lang, Some(&tags)).await?
        } else {
            kernel.exec_async(code, lang, Some(&tags)).await?
        };

        // Record symbols assigned in kernel (unless it was a fork)
        if !pure {
            for (name, kind) in parse_info.variables_modified() {
                symbols
                    .entry(name)
                    .and_modify(|info| {
                        info.home = kernel_id.to_string();
                        info.modified = Utc::now();
                    })
                    .or_insert_with(|| SymbolInfo::new(kind, &kernel_id));
            }
        }

        // Either way, store the task
        let task_info = self
            .store(&task, code, parse_info, &kernel_id, force_fork)
            .await;
        Ok(task_info)
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
        parse_info: &ParseInfo,
        kernel_id: &str,
        is_fork: bool,
    ) -> TaskInfo {
        if let (false, Ok(mut receiver)) = (task.is_ended(), task.subscribe()) {
            // When finished, update the tasks info stored in `tasks`
            let tasks = self.tasks.clone();
            let task_id = task.id.clone();
            tokio::spawn(async move {
                match receiver.recv().await {
                    Ok(result) => {
                        let mut tasks = tasks.lock().await;
                        match KernelTasks::get_mut(&mut tasks, &task_id) {
                            // Finish the task with the result
                            Some(task_info) => task_info.task.end(result),
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
        tasks.put(task, code, parse_info, kernel_id, is_fork).await
    }

    /// Cancel a task
    async fn cancel(&self, task_num_or_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task_info) = tasks.find_mut(task_num_or_id) {
            task_info.task.interrupt().await
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
            if !task.is_ended() {
                task.interrupt().await?;
            }
        }
        Ok(())
    }

    /// Remove old tasks to avoid the `tasks` list growing indefinitely in long running processes.
    async fn clean(tasks: &Arc<Mutex<KernelTasks>>) {
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
            if task_info.task.is_ended() {
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
            if !task.is_ended() {
                if let Err(error) = task.interrupt().await {
                    tracing::debug!("While cancelling unfinished task `{}`: {}", task.id, error)
                }
            }
            list.remove(index);
        }
    }

    /// A read-evaluate-print function
    ///
    /// Primarily intended for use in interactive mode to execute a line of code REPL Tailwind
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
        use common::regex::Regex;
        use events::{subscribe, unsubscribe, Subscriber};

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
            let format = language
                .as_ref()
                .map_or_else(|| Format::Unknown, |lang| formats::match_name(lang));
            let parse_info = match parsers::parse(format, &code, None) {
                Ok(parse_info) => parse_info,
                Err(..) => ParseInfo::default(),
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
                    Some(_) => KernelSelector::from_lang_and_tags(language.as_deref(), None),
                    None => {
                        let tasks = self.tasks.lock().await;
                        match tasks.inner.last() {
                            // Select for kernel used for the last task
                            Some(task_info) => KernelSelector {
                                id: task_info.kernel_id.clone(),
                                ..Default::default()
                            },
                            // Select anything (will select the first kernel)
                            None => KernelSelector::default(),
                        }
                    }
                },
            };

            // Execute the code
            let mut task_info = self.exec(&code, &parse_info, is_fork, &selector).await?;

            if background {
                // Indicate task is running in background
                tracing::info!("Task #{} is running in background", task_info.num);
                result::nothing()
            } else {
                // If cancellable, subscribe to user interrupt, cancelling the task on that event
                let subscription_id = if task_info.task.is_interruptable() {
                    let tasks = self.tasks.clone();
                    let task_id = task_info.task.id.clone();
                    let (sender, mut receiver) = mpsc::unbounded_channel();
                    tokio::spawn(async move {
                        // This ends on interrupt or on unsubscribe (when the channel sender is dropped)
                        if let Some(..) = receiver.recv().await {
                            let mut tasks = tasks.lock().await;
                            if let Some(TaskInfo { task, .. }) = tasks.find_mut(&task_id) {
                                if let Err(error) = task.interrupt().await {
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
                            use std::fmt::Write;
                            write!(err, "\n{}", trace)?;
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
///
/// This is a relatively expensive function, mainly because it involves searching PATH
/// for binaries. Therefore, as an optimization, the return value is memoized and
/// only updated after a certain duration (we still what updates so it a kernel is enabled
/// (by installing a language runtime) while the server is running, it will get reflected in the list)
static AVAILABLE_LIST: Lazy<RwLock<Vec<Kernel>>> = Lazy::new(|| RwLock::new(Vec::new()));
static AVAILABLE_UPDATED: Lazy<RwLock<Instant>> = Lazy::new(|| RwLock::new(Instant::now()));
#[allow(clippy::vec_init_then_push, unused_mut)]
pub async fn available() -> Vec<Kernel> {
    let available = AVAILABLE_LIST.read().await;
    if !available.is_empty()
        && Instant::now().saturating_duration_since(*AVAILABLE_UPDATED.read().await)
            < Duration::from_secs(60)
    {
        return available.clone();
    }
    drop(available);

    tracing::debug!("Updating list of available kernels");

    let mut available: Vec<Kernel> = Vec::new();

    #[cfg(feature = "kernel-store")]
    available.push(kernel_store::StoreKernel::new().spec().await);

    #[cfg(feature = "kernel-calc")]
    available.push(kernel_calc::CalcKernel::new().spec().await);

    #[cfg(feature = "kernel-tailwind")]
    available.push(kernel_tailwind::TailwindKernel::new().spec().await);

    #[cfg(feature = "kernel-sql")]
    available.push(
        kernel_sql::SqlKernel::new(&KernelSelector::default())
            .spec()
            .await,
    );

    #[cfg(feature = "kernel-prql")]
    available.push(
        kernel_prql::PrqlKernel::new(&KernelSelector::default(), None)
            .spec()
            .await,
    );

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
    available.append(
        &mut kernel_jupyter::JupyterKernel::available()
            .await
            .unwrap_or_default(),
    );

    *AVAILABLE_LIST.write().await = available.clone();
    *AVAILABLE_UPDATED.write().await = Instant::now();

    available
}

/// List the languages supported by the kernels available on this machine
///
/// Returns a list of unique language names across the available kernels.
#[allow(clippy::vec_init_then_push)]
pub async fn languages() -> Result<Vec<String>> {
    let mut languages: HashSet<String> = HashSet::new();
    let kernels = available().await;
    for kernel in kernels {
        for format in kernel.languages {
            languages.insert(format.to_string());
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
fn format_duration(micros: Option<u64>) -> String {
    match micros {
        Some(micros) => {
            let duration = Duration::from_micros(micros);
            humantime::format_duration(duration).to_string()
        }
        _ => "".to_string(),
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use cli_utils::{
        clap::{self, Parser},
        result, Result, Run,
    };
    use common::once_cell::sync::Lazy;
    use tokio::sync::Mutex;

    use super::*;

    /// Manage and use execution kernels
    #[derive(Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Parser)]
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
                        Action::Cancel(action) => action.run(kernel_space).await,
                        Action::Symbols(action) => action.run(kernel_space).await,
                        Action::Restart(action) => action.run(kernel_space).await,

                        _ => bail!("Unhandled action"),
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
    #[derive(Parser)]
    #[clap(alias = "avail")]
    pub struct Available {}
    #[async_trait]
    impl Run for Available {
        async fn run(&self) -> Result {
            result::value(available().await)
        }
    }

    /// List the languages supported by the kernels available on this machine
    ///
    /// Returns a unique list of languages across all kernels available.
    #[derive(Parser)]
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
    #[derive(Parser)]
    pub struct External {}
    #[async_trait]
    impl Run for External {
        async fn run(&self) -> Result {
            result::value(jupyter_running().await?)
        }
    }

    /// List the directories on this machine that will be searched for Jupyter kernel specs
    /// and running kernels
    #[derive(Parser)]
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
    static KERNEL_SPACE: Lazy<Mutex<KernelSpace>> =
        Lazy::new(|| Mutex::new(KernelSpace::new(None)));

    /// Execute code within a document kernel space
    ///
    /// Mainly intended for testing that Stencila is able to talk
    /// to Jupyter kernels and execute code within them.
    ///
    /// Use the `--kernel` option to specify, by name, language or type, which kernel the code
    /// should be executed in e.g.,
    ///
    /// ```stencila
    /// > kernels execute Math.PI --lang=javascript
    /// ```
    ///
    /// ```stencila
    /// > kernels execute Math.PI --lang javascript --kernel="type:jupyter"
    /// ```
    ///
    /// In interactive mode, you can set the command prefix to "stay" in a particular
    /// language and mimic a REPL in that language e.g.,
    ///
    /// ```stencila
    /// > kernels execute --lang=javascript
    /// > let r = 10
    /// > 2 * Math.PI * r
    /// ```
    ///
    /// If a kernel is not yet running for the language then one will be started
    /// (if installed on the machine).
    #[derive(Parser)]
    #[clap(alias = "exec", verbatim_doc_comment)]
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
    #[derive(Parser)]
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
            tasks
                .display(self.num, &self.sort, self.desc, self.kernel.clone())
                .await
        }
    }

    /// Show the code symbols in a document kernel space
    #[derive(Parser)]
    pub struct Symbols {}
    impl Symbols {
        pub async fn run(&self, kernel_space: &KernelSpace) -> Result {
            let symbols = kernel_space.symbols.lock().await;
            display_symbols(&symbols)
        }
    }

    /// Cancel a code execution task, or all tasks, in a document kernel space
    ///
    /// Use an integer to cancel a task by it's number.
    /// Use "all" to cancel all unfinished tasks.
    #[derive(Parser)]
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
    /// ```stencila
    /// > kernels start r
    /// ```
    ///
    /// or connected to from Stencila,
    ///  
    /// ```stencila
    /// > kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
    /// ```
    ///
    /// To get a list of externally started Jupyter kernels that can be connected to run,
    ///
    /// ```stencila
    /// > kernels external
    /// ```
    #[derive(Parser)]
    #[clap(alias = "kernels", verbatim_doc_comment)]
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
    #[derive(Parser)]
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
    #[derive(Parser)]
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
    #[derive(Parser)]
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
    /// ```stencila
    /// > kernels external
    /// ```
    ///
    /// and then connect to a kernel using its Jupyter id e.g.,
    ///
    /// ```stencila
    /// > kernels connect beaac32f-32a4-46bc-9940-186a14d9acc9
    /// ```
    ///
    /// Alternatively, use the path (relative or absolute) of the Jupyter notebook
    /// whose (already started) kernel you wish to connect to e.g.,
    ///
    /// ```stencila
    /// > kernels connect ../main.ipynb
    /// ```
    #[derive(Parser)]
    #[clap(verbatim_doc_comment)]
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
    #[derive(Parser)]
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
