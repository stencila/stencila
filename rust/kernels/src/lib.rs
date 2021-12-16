use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut};
use graph_triples::{Relation, Resource};
#[allow(unused_imports)]
use kernel::{
    async_trait::async_trait,
    eyre::{bail, eyre, Result},
    stencila_schema::{CodeError, Node},
    Kernel, KernelInfo, KernelStatus, KernelTrait,
};
use serde::Serialize;
use std::collections::{hash_map::Entry, BTreeMap, HashMap, HashSet};
use validator::Contains;

// Re-exports
pub use kernel::KernelSelector;

/// An identifier for a kernel
///
/// This is *not* a UUID but rather a id that is unique to a
/// local kernel space. This allows more useful ids to be assigned
/// e.g. `python`, `r` etc.
type KernelId = String;

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
                    if selector.matches(&$kernel.spec()) && $kernel.available().await {
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

    async fn available(&self) -> bool {
        dispatch_variants!(self, available).await
    }

    async fn forkable(&self) -> bool {
        dispatch_variants!(self, forkable).await
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

    async fn exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)> {
        dispatch_variants!(self, exec, code).await
    }

    async fn fork_exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)> {
        dispatch_variants!(self, fork_exec, code).await
    }
}

/// A map of kernel ids to kernels.
///
/// A `newtype` that exists solely to provide a `Result` (rather than `<Option>`)
/// when getting a kernel by id.
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
}

#[derive(Debug, Default, Serialize)]
pub struct KernelSpace {
    /// The kernels in the document kernel space
    kernels: KernelMap,

    /// The symbols in the document kernel space
    symbols: HashMap<String, SymbolInfo>,
}

impl KernelSpace {
    /// Create a new kernel space
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a list of kernels in the kernel space
    pub async fn kernels(&self) -> Vec<KernelInfo> {
        let mut info = Vec::new();
        for (id, kernel) in self.kernels.iter() {
            let id = id.to_string();
            let spec = kernel.spec();
            let status = match kernel.status().await {
                Ok(status) => status,
                Err(error) => {
                    tracing::warn!("While getting kernel status: {}", error);
                    KernelStatus::Unknown
                }
            };
            let interruptable = kernel.interruptable().await;
            let forkable = kernel.forkable().await;
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

    /// Get a list of symbols in the kernel space
    ///
    /// Mainly for inspection, in the future may return a list with
    /// more information e.g. the type of symbol.
    pub fn symbols(&self) -> HashMap<String, SymbolInfo> {
        self.symbols.clone()
    }

    /// Get a symbol from the kernel space
    pub async fn get(&mut self, name: &str) -> Result<Node> {
        let symbol_info = self
            .symbols
            .get(name)
            .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

        let kernel = self.kernels.get_mut(&symbol_info.home)?;
        kernel.get(name).await
    }

    /// Set a symbol in the kernel space
    pub async fn set(&mut self, name: &str, value: Node, language: &str) -> Result<()> {
        let selector = KernelSelector::parse(language);
        let kernel_id = self.ensure(&selector).await?;
        tracing::debug!("Setting symbol `{}` in kernel `{}`", name, kernel_id);

        let kernel = self.kernels.get_mut(&kernel_id)?;
        kernel.set(name, value).await?;

        match self.symbols.entry(name.to_string()) {
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
    ///
    /// Symbols that the code uses, but have a different home kernel, are mirrored to the kernel.
    pub async fn exec(
        &mut self,
        code: &str,
        selector: &KernelSelector,
        relations: Option<Vec<(Relation, Resource)>>,
        fork: bool,
    ) -> Result<(Vec<Node>, Vec<CodeError>)> {
        // Determine the kernel to execute in
        let kernel_id = self.ensure(selector).await?;
        tracing::debug!("Executing code in kernel `{}`", kernel_id);

        // Mirror used symbols into the kernel
        if let Some(relations) = &relations {
            for relation in relations {
                let name = if let (Relation::Use(..), Resource::Symbol(symbol)) = relation {
                    if self.symbols.has_element(&symbol.name) {
                        &symbol.name
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let SymbolInfo {
                    home,
                    assigned,
                    mirrored,
                    ..
                } = self
                    .symbols
                    .get_mut(name)
                    .ok_or_else(|| eyre!("Unknown symbol `{}`", name))?;

                // Skip if home is the target kernel
                if *home == kernel_id {
                    continue;
                }

                // Skip if already mirrored since last assigned
                if let Some(mirrored) = mirrored.get(&kernel_id) {
                    if mirrored >= assigned {
                        continue;
                    }
                }

                tracing::debug!(
                    "Mirroring symbol `{}` from kernel `{}` to kernel `{}`",
                    name,
                    home,
                    kernel_id
                );

                let home_kernel = self.kernels.get_mut(home)?;
                let value = home_kernel.get(name).await?;

                let mirror_kernel = self.kernels.get_mut(&kernel_id)?;
                mirror_kernel.set(name, value).await?;

                match mirrored.entry(kernel_id.clone()) {
                    Entry::Occupied(mut occupied) => {
                        let datetime = occupied.get_mut();
                        *datetime = Utc::now();
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(Utc::now());
                    }
                }
            }
        }

        // Execute the code
        let kernel = self.kernels.get_mut(&kernel_id)?;
        let results = match fork {
            true => kernel.fork_exec(code),
            false => kernel.exec(code),
        }
        .await?;

        // Record symbols assigned in kernel (unless it was a fork)
        if let (false, Some(relations)) = (fork, relations) {
            for relation in relations {
                let (name, kind) =
                    if let (Relation::Assign(..), Resource::Symbol(symbol)) = relation {
                        (symbol.name, symbol.kind)
                    } else {
                        continue;
                    };

                match self.symbols.entry(name) {
                    Entry::Occupied(mut occupied) => {
                        let info = occupied.get_mut();
                        info.home = kernel_id.clone();
                        info.assigned = Utc::now();
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(SymbolInfo::new(&kind, &kernel_id));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Ensure that a kernel exists for a selector
    ///
    /// Returns the kernel's id.
    async fn ensure(&mut self, selector: &KernelSelector) -> Result<KernelId> {
        // Is there already a running kernel that matches the selector?
        for (kernel_id, kernel) in self.kernels.iter_mut() {
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
            .kernels
            .keys()
            .filter(|key| key.starts_with(&kernel_id))
            .count();
        let kernel_id = if count == 0 {
            kernel_id
        } else {
            [kernel_id, count.to_string()].concat()
        };

        self.kernels.insert(kernel_id.clone(), kernel);

        Ok(kernel_id)
    }

    /// Connect to a running kernel
    #[allow(unused_variables)]
    async fn connect(&mut self, id_or_path: &str) -> Result<KernelId> {
        #[cfg(feature = "kernel-jupyter")]
        {
            let (kernel_id, kernel) = kernel_jupyter::JupyterKernel::connect(id_or_path).await?;
            self.kernels
                .insert(kernel_id.clone(), MetaKernel::Jupyter(kernel));

            Ok(kernel_id)
        }

        #[cfg(not(feature = "kernel-jupyter"))]
        kernel::eyre::bail!(
            "Unable to connect to running kernel because support for Jupyter kernels is not enabled",
        )
    }

    /// Stop one of the kernels and remove it from the kernel space
    async fn stop(&mut self, id: &str) -> Result<()> {
        self.kernels.get_mut(id)?.stop().await?;
        self.kernels.remove(id);
        Ok(())
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
        fork: bool,
    ) -> cli_utils::Result {
        use cli_utils::result;
        use once_cell::sync::Lazy;
        use regex::Regex;

        static SYMBOL: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^\w+$").expect("Unable to create regex"));

        if code.is_empty() {
            result::nothing()
        } else if code == "%symbols" {
            let symbols = self.symbols();
            result::value(symbols)
        } else if code == "%kernels" {
            let kernels = self.kernels().await;
            result::value(kernels)
        } else if language.is_none() && kernel.is_none() && !fork && SYMBOL.is_match(code) {
            match self.get(code).await {
                Ok(node) => result::value(node),
                Err(err) => {
                    tracing::error!("{}", err);
                    result::nothing()
                }
            }
        } else {
            let code = code.replace("\\n", "\n");

            let language = language.unwrap_or_else(|| "calc".to_string());

            // If possible, parse the code so that we can use the relations to determine variables that
            // are assigned or used (needed for variable mirroring).
            let relations = match parsers::parse("<cli>", &code, &language) {
                Ok(pairs) => pairs,
                Err(..) => Vec::new(),
            };
            let selector = match kernel {
                Some(kernel) => {
                    let mut selector = KernelSelector::parse(&kernel);
                    selector.lang = Some(language);
                    selector
                }
                None => KernelSelector::new(None, Some(language), None),
            };
            let (nodes, errors) = self.exec(&code, &selector, Some(relations), fork).await?;
            if !errors.is_empty() {
                for error in errors {
                    let mut err = error.error_message;
                    if let Some(trace) = error.stack_trace {
                        err += &format!("\n{}", trace);
                    }
                    tracing::error!("{}", err)
                }
            }
            match nodes.len() {
                0 => result::nothing(),
                1 => result::value(nodes[0].clone()),
                _ => result::value(nodes),
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
                if kernel.available().await {
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

        /// Fork the kernel before executing (mainly for testing)
        #[structopt(short, long)]
        fork: bool,
    }
    #[async_trait]
    impl Run for Execute {
        async fn run(&self) -> Result {
            KERNEL_SPACE
                .lock()
                .await
                .repl(
                    &self.code.join(" "),
                    self.lang.clone(),
                    self.kernel.clone(),
                    self.fork,
                )
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
            let mut kernels = KERNEL_SPACE.lock().await;
            let selector = KernelSelector::parse(&self.selector);
            let kernel_id = kernels.start(&selector).await?;
            let kernel = kernels.kernels.get(&kernel_id)?;
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
            KERNEL_SPACE.lock().await.stop(&self.id).await?;
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
            let mut kernels = KERNEL_SPACE.lock().await;
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
            let status = KERNEL_SPACE.lock().await.kernels().await;
            result::value(status)
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
            let kernels = KERNEL_SPACE.lock().await;
            let kernel = kernels.kernels.get(&self.id)?;
            result::value(kernel)
        }
    }
}
