use std::{fmt, path::Path};

use common::{
    async_trait::async_trait,
    chrono::{DateTime, Utc},
    eyre::{bail, Result},
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    strum::Display,
    tokio::sync::{broadcast, mpsc},
    tracing,
};
use formats::Format;
pub use graph_triples::TagMap;
use stencila_schema::{CodeError, Node};
use utils::some_box_string;
use uuids::uuid_family;

// Re-export for the convenience of crates that implement `KernelTrait`
pub use common;
pub use formats;
pub use graph_triples;
pub use stencila_schema;

/// The type of kernel
///
/// At present this is mainly for informational purposes.
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(crate = "common::serde")]
pub enum KernelType {
    Builtin,
    Micro,
    Jupyter,
}

/// A specification for kernels
///
/// All kernels, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Kernel` instance from the
/// `spec` function of `KernelTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Kernel {
    /// The name of the kernel
    ///
    /// This is used for informational purposes and to allow the user to specify
    /// which kernel they want to use (e.g. in instances that they have more than one kernel that
    /// is capable of executing a language).
    pub name: String,

    /// The type of kernel
    pub r#type: KernelType,

    /// The languages supported by the kernel
    ///
    /// Used when generating execution plans to determine which tasks to
    /// run in each kernel.
    pub languages: Vec<Format>,

    /// Is the kernel available on the current machine?
    pub available: bool,

    /// Is the kernel interruptable on the current machine?
    pub interruptable: bool,

    /// Is the kernel fork-able on the current machine?
    ///
    /// Used when generating execution plans to determine which tasks
    /// can be conducted concurrently.
    pub forkable: bool,
}

impl Kernel {
    // Create a new kernel specification
    pub fn new(
        name: &str,
        r#type: KernelType,
        languages: &[Format],
        available: bool,
        interruptable: bool,
        forkable: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            r#type,
            languages: languages.into(),
            available,
            interruptable,
            forkable,
        }
    }

    // Does the kernel specification match against a kernel selector string?
    pub fn matches(&self, selector: &str) -> bool {
        KernelSelector::parse(selector).matches(self)
    }
}

/// An identifier for a kernel
///
/// This is *not* a UUID but rather a id that is unique to a
/// local kernel space. This allows more useful ids to be assigned
/// e.g. `python`, `r` etc.
pub type KernelId = String;

/// The status of a running kernel
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Display)]
#[serde(crate = "common::serde")]
#[allow(dead_code)]
pub enum KernelStatus {
    Pending,
    Starting,
    Idle,
    Busy,
    Unresponsive,
    Stopping,
    Finished,
    Failed,
    Unknown,
}

/// Information on a running kernel
///
/// Used when displaying information to the user about currently
/// running kernels.
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct KernelInfo {
    /// The id of the kernel instance
    pub id: KernelId,

    /// The status of the kernel
    pub status: KernelStatus,

    /// The kernel spec
    #[serde(flatten)]
    pub spec: Kernel,

    /// Whether the kernel is interruptable on the current machine
    pub interruptable: bool,

    /// Whether the kernel is forkable on the current machine
    pub forkable: bool,
}

/// A selector used to choose amongst alternative kernels
#[skip_serializing_none]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(crate = "common::serde")]
pub struct KernelSelector {
    /// A string that will match against the kernel `name` or any of its `languages`
    pub any: Option<String>,

    /// A string that will match against the kernel `name`
    pub name: Option<String>,

    /// A string that will match against any of a kernel's `languages`
    pub lang: Option<String>,

    /// A string that will match against the kernel `type`
    pub r#type: Option<String>,

    /// Extra configuration details which can be passed to the kernel's `new` function
    pub config: Option<String>,

    /// A string that will match against the kernel `id`
    pub id: Option<String>,
}

impl fmt::Display for KernelSelector {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut str = self.any.clone().unwrap_or_default();
        if let Some(name) = &self.name {
            str.push_str(" name:");
            str.push_str(name);
        }
        if let Some(lang) = &self.lang {
            str.push_str(" lang:");
            str.push_str(lang);
        }
        if let Some(r#type) = &self.r#type {
            str.push_str(" type:");
            str.push_str(r#type);
        }
        if let Some(config) = &self.config {
            str.push_str(" config:");
            str.push_str(config);
        }
        if let Some(id) = &self.id {
            str.push_str(" id:");
            str.push_str(id);
        }
        write!(formatter, "{}", str.trim())
    }
}

impl KernelSelector {
    /// Create a new `KernelSelector` from a language string and tags
    pub fn from_lang_and_tags(lang: Option<&str>, tags: Option<&TagMap>) -> Self {
        let format = match lang {
            Some(lang) => formats::match_name(lang),
            None => Format::Unknown,
        };
        Self::from_format_and_tags(format, tags)
    }

    /// Create a new `KernelSelector` from a `Format` and tags
    pub fn from_format_and_tags(lang: Format, tags: Option<&TagMap>) -> Self {
        let (name, config) = if let Some(tags) = tags {
            let name = tags.get_value("kernel");
            let config = match lang {
                Format::SQL | Format::PrQL => tags.get_value("db"),
                _ => None,
            };
            (name, config)
        } else {
            (None, None)
        };

        Self {
            any: None,
            lang: Some(lang.to_string()),
            r#type: None,
            name,
            config,
            id: None,
        }
    }

    /// Parse a kernel selector string into a `KernelSelector`
    pub fn parse(selector: &str) -> Self {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(\b(name|lang|type)\s*:\s*([\w-]+)\b)|([\w-]+)")
                .expect("Unable to create regex")
        });

        let mut any = None;
        let mut name = None;
        let mut lang = None;
        let mut r#type = None;
        for captures in REGEX.captures_iter(selector) {
            if let Some(tag) = captures.get(2) {
                let value = Some(captures[3].to_string());
                match tag.as_str() {
                    "name" => {
                        if name.is_none() {
                            name = value
                        } else {
                            tracing::warn!("Ignoring additional kernel `name` selector");
                        }
                    }
                    "lang" => {
                        if lang.is_none() {
                            lang = value
                        } else {
                            tracing::warn!("Ignoring additional kernel `lang` selector");
                        }
                    }
                    "type" => {
                        if r#type.is_none() {
                            r#type = value
                        } else {
                            tracing::warn!("Ignoring additional kernel `type` selector");
                        }
                    }
                    _ => (),
                }
            } else if any.is_none() {
                any = Some(captures[4].to_string())
            } else {
                tracing::warn!(
                    "Ignoring extraneous kernel selector: {}",
                    captures[0].to_string()
                );
            }
        }

        Self {
            any,
            name,
            lang,
            r#type,
            config: None,
            id: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.any.is_none() && self.name.is_none() && self.lang.is_none() && self.r#type.is_none()
    }

    /// Does a `Kernel` match this selector
    pub fn matches(&self, kernel: &Kernel) -> bool {
        let mut matched = true;

        if let (true, Some(name)) = (matched, &self.name) {
            matched = name.to_lowercase() == kernel.name.to_lowercase();
        } else if let Some(name) = &self.any {
            matched = name.to_lowercase() == kernel.name.to_lowercase();
        }

        if let (true, Some(lang)) = (matched, &self.lang) {
            let format = formats::match_name(lang);
            let mut lang_matched = false;
            for kernel_format in &kernel.languages {
                if format != Format::Unknown && format == *kernel_format {
                    lang_matched = true;
                    break;
                }
            }
            matched &= lang_matched;
        } else if let (false, Some(lang)) = (matched, &self.any) {
            let format = formats::match_name(lang);
            for kernel_format in &kernel.languages {
                if format != Format::Unknown && format == *kernel_format {
                    matched = true;
                    break;
                }
            }
        }

        if let (true, Some(r#type)) = (matched, &self.r#type) {
            matched = r#type.to_lowercase() == kernel.r#type.to_string().to_lowercase();
        }

        matched
    }

    /// Select the first kernel that matches against this selector
    pub fn select<'lt>(&self, kernels: &'lt [Kernel]) -> Option<&'lt Kernel> {
        kernels.iter().find(|kernel| self.matches(kernel))
    }
}

// An id for tasks
uuid_family!(TaskId, "ta");

/// Output nodes from a task
pub type TaskOutputs = Vec<Node>;

/// Messages from a task
///
/// In the future this will likely be a [`CodeMessage`] vector,
/// rather than a [`CodeError`] vector.
pub type TaskMessages = Vec<CodeError>;

/// The result of a [`Task`]
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct TaskResult {
    /// Outputs from the task
    pub outputs: Vec<Node>,

    /// Messages from the task
    pub messages: Vec<CodeError>,
}

impl TaskResult {
    /// Create a new task result
    pub fn new(outputs: Vec<Node>, messages: Vec<CodeError>) -> Self {
        Self { outputs, messages }
    }

    /// A syntax error occurred
    pub fn syntax_error(message: &str) -> Self {
        Self::new(
            vec![],
            vec![CodeError {
                error_type: some_box_string!("SyntaxError"),
                error_message: message.to_string(),
                ..Default::default()
            }],
        )
    }

    /// Used to indicate an internal (to Stencila code) error rather than user error
    ///
    /// Usually used in conjunction with a `tracing::error` to expose error details on the
    /// server side rather than client side.
    pub fn internal_error(message: &str) -> Self {
        Self::new(
            vec![],
            vec![CodeError {
                error_type: some_box_string!("InternalError"),
                error_message: message.to_string(),
                ..Default::default()
            }],
        )
    }
}

/// A [`broadcast::channel`] sender of a [`TaskResult`]
///
/// Subscribe to this to receive the result of a task. This needs to be a `broadcast`
/// channel (single-producer-multiple-receiver) so that multiple async
/// tasks can receive the result of the execution task (e.g. both the
/// original caller, and the `KernelsTasks` that keeps a track of tasks and
/// when they are finished).
pub type TaskSender = broadcast::Sender<TaskResult>;

/// A [`broadcast::channel`] receiver of a [`TaskResult`]
pub type TaskReceiver = broadcast::Receiver<TaskResult>;

/// A [`mpsc::channel`] sender to interrupt the task
pub type TaskInterrupter = mpsc::Sender<()>;

/// A task running in a [`Kernel`]
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "common::serde")]
pub struct Task {
    /// The uuid of the task
    pub id: TaskId,

    /// The time that the task was created by the kernel
    pub created: DateTime<Utc>,

    /// The time that the task was started by the kernel
    ///
    /// If the task was placed on a queue then this is expected to be after `created`.
    /// In this case, the task may also be interrupted before it is started.
    pub started: Option<DateTime<Utc>>,

    /// The time that the task finished (successfully or not)
    pub finished: Option<DateTime<Utc>>,

    /// The time that the task was interrupted
    pub interrupted: Option<DateTime<Utc>>,

    /// The result of the task (if it was completed immediately)
    pub result: Option<TaskResult>,

    /// The result sender for the task (if it was started asynchronously)
    #[serde(skip)]
    pub sender: Option<TaskSender>,

    /// The interrupter for the task (may be set after the task is started)
    #[serde(skip)]
    pub interrupter: Option<TaskInterrupter>,
}

impl Task {
    /// Create and immediately begin a kernel task
    pub fn begin(sender: Option<TaskSender>, interrupter: Option<TaskInterrupter>) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            created: now,
            started: Some(now),
            finished: None,
            interrupted: None,
            result: None,
            sender,
            interrupter,
        }
    }

    /// Create and immediately begin a synchronous task
    pub fn begin_sync() -> Self {
        Self::begin(None, None)
    }

    /// Create and immediately begin an asynchronous task
    pub fn begin_async(sender: TaskSender) -> Self {
        Self::begin(Some(sender), None)
    }

    /// Is the task an async task?
    pub fn is_async(&self) -> bool {
        self.sender.is_some()
    }

    /// Is the task able to be interrupted?
    pub fn is_interruptable(&self) -> bool {
        self.interrupter.is_some()
    }

    /// Is the task ended (i.e. finished or interrupted)?
    pub fn is_ended(&self) -> bool {
        self.finished.is_some() || self.interrupted.is_some()
    }

    /// Subscribe to the task
    pub fn subscribe(&self) -> Result<TaskReceiver> {
        match self.sender.as_ref() {
            Some(sender) => Ok(sender.subscribe()),
            None => bail!("Task is sync, so can not be subscribed to"),
        }
    }

    /// End the tasks by passing a result
    ///
    /// This function determines if the result has a message with `error_type: Interrupt`
    /// to determine if it was `interrupted`.
    #[tracing::instrument]
    pub fn end(&mut self, result: TaskResult) {
        // Log an error because these shouldn't really ever happen; but
        // if they do don't return an error because its not fatal if it does
        if let Some(finished) = self.finished {
            tracing::error!("Task already finished at `{}`", finished);
        } else if let Some(interrupted) = self.interrupted {
            tracing::error!("Task already interrupted at `{}`", interrupted);
        };

        let was_interrupted = result.messages.iter().any(|msg| match &msg.error_type {
            Some(type_) => type_.as_str() == "Interrupt",
            None => false,
        });
        if was_interrupted {
            self.interrupted = Some(Utc::now());
        } else {
            self.finished = Some(Utc::now());
        }

        self.result = Some(result);
        self.interrupter = None; // Ensure the async interrupt task ends by closing the channel
    }

    /// Interrupt the task by sending a message on its interrupt channel
    ///
    /// Note that this is not the only way to interrupt a task. Some code
    /// elsewhere may send a message directly to the channel.
    #[tracing::instrument]
    pub async fn interrupt(&mut self) -> Result<()> {
        if let Some(finished) = self.finished {
            // Warn if the task already finished
            tracing::warn!("Task already finished at `{}`", finished);
            Ok(())
        } else if let Some(interrupted) = self.interrupted {
            // Just debug here since this is not really an error or warning
            // (e.g. another client interrupted)
            tracing::debug!("Task was already interrupted at `{}`", interrupted);
            Ok(())
        } else if let Some(interrupter) = &mut self.interrupter {
            if let Err(..) = interrupter.send(()).await {
                tracing::debug!("Interruption receiver for task `{}` dropped", self.id);
            };
            tracing::info!("Task `{}` was interrupted", self.id);
            self.interrupted = Some(Utc::now());
            self.interrupter = None; // Ensure finish of the async interruption task
            Ok(())
        } else {
            bail!("Task `{}` is not interruptable", self.id)
        }
    }

    /// Wait for a result from the task
    ///
    /// For synchronous tasks, returns the result immediately. Errors if the
    /// result sender has already dropped, or if for some reason the task has neither
    /// a `result`, nor a `receiver`.
    pub async fn result(&mut self) -> Result<TaskResult> {
        if let Some(result) = &self.result {
            Ok(result.clone())
        } else if let Ok(mut receiver) = self.subscribe() {
            let result = match receiver.recv().await {
                Ok(result) => result,
                Err(..) => bail!("Result sender for task `{}` dropped", self.id),
            };
            self.end(result.clone());
            Ok(result)
        } else {
            bail!(
                "Task `{}` has neither a `result`, nor a `receiver`!",
                self.id
            )
        }
    }
}

/// A trait for kernels
///
/// This trait can be used by Rust implementations of kernels, allowing them to
/// be compiled into the Stencila binaries.
#[async_trait]
pub trait KernelTrait {
    /// Get the [`Kernel`] specification for this implementation
    ///
    /// Must be implemented by [`KernelTrait`] implementations.
    async fn spec(&self) -> Kernel;

    /// Is the kernel available on the current machine?
    async fn is_available(&self) -> bool {
        self.spec().await.available
    }

    /// Is the kernel interruptable on the current machine?
    async fn is_interruptable(&self) -> bool {
        self.spec().await.interruptable
    }

    /// Is the kernel forkable on the current machine?
    async fn is_forkable(&self) -> bool {
        self.spec().await.forkable
    }

    /// Start the kernel
    ///
    /// Will usually be overridden by [`KernelTrait`] implementations.
    /// Should set the working directory to `directory`.
    async fn start(&mut self, _directory: &Path) -> Result<()> {
        Ok(())
    }

    /// Start the kernel in the current working directory
    async fn start_here(&mut self) -> Result<()> {
        let cwd = std::env::current_dir()?;
        self.start(&cwd).await
    }

    /// Stop the kernel
    ///
    /// Will usually be overridden by [`KernelTrait`] implementations.
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the status of the kernel
    ///
    /// Must be implemented by [`KernelTrait`] implementations.
    async fn status(&self) -> Result<KernelStatus>;

    /// Is the kernel busy?
    async fn is_busy(&self) -> Result<bool> {
        Ok(self.status().await? == KernelStatus::Busy)
    }

    /// Get a symbol from the kernel
    ///
    /// Must be implemented by [`KernelTrait`] implementations.
    async fn get(&mut self, name: &str) -> Result<Node>;

    /// Set a symbol in the kernel
    ///
    /// Must be implemented by [`KernelTrait`] implementations.
    async fn set(&mut self, name: &str, value: Node) -> Result<()>;

    /// Execute code in the kernel and get outputs and messages
    ///
    /// This is a convenience method when all you want to do is get [`Task`]
    /// outputs and messages (and are not interested in task duration or interruption).
    async fn exec(
        &mut self,
        code: &str,
        lang: Format,
        tags: Option<&TagMap>,
    ) -> Result<(TaskOutputs, TaskMessages)> {
        let mut task = self.exec_sync(code, lang, tags).await?;
        let TaskResult { outputs, messages } = task.result().await?;
        Ok((outputs, messages))
    }

    /// Execute code in the kernel synchronously
    ///
    /// Use the method instead of `exec` when you want to know details of the execution
    /// such as its duration.
    ///
    /// Note that although this method is called `exec_sync` it is Rust `async`. This
    /// is necessary because some kernels may need to call other `async` functions as
    /// part of their implementation.
    ///
    /// Must be implemented by [`KernelTrait`] implementations.
    async fn exec_sync(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task>;

    /// Execute code in the kernel asynchronously
    ///
    /// Should be overridden by [`KernelTrait`] implementations that are interruptable.
    /// The default implementation simply calls `exec_sync`.
    async fn exec_async(
        &mut self,
        code: &str,
        lang: Format,
        tags: Option<&TagMap>,
    ) -> Result<Task> {
        self.exec_sync(code, lang, tags).await
    }

    /// Fork the kernel and execute code in the fork
    ///
    /// Should be overridden by [`KernelTrait`] implementations that are forkable.
    /// The default implementation errors because code marked as `@pure` should not
    /// be executed in the main kernel in case it has side-effects (e.g. assigning
    /// temporary variables) which are intended to be ignored.
    async fn exec_fork(
        &mut self,
        _code: &str,
        _lang: Format,
        _tags: Option<&TagMap>,
    ) -> Result<Task> {
        bail!("Kernel is not forkable")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use utils::some_string;

    #[test]
    fn kernel_selector_new() {
        let ks = KernelSelector::parse("name_lang");
        assert_eq!(ks.any, some_string!("name_lang"));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, None);
        assert_eq!(ks.to_string(), "name_lang");

        let ks = KernelSelector::parse("name_lang foo bar");
        assert_eq!(ks.any, some_string!("name_lang"));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, None);

        let ks = KernelSelector::parse("type:micro name_lang");
        assert_eq!(ks.any, some_string!("name_lang"));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, some_string!("micro"));

        let ks = KernelSelector::parse("name_lang type:jupyter lang:py");
        assert_eq!(ks.any, some_string!("name_lang"));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, some_string!("py"));
        assert_eq!(ks.r#type, some_string!("jupyter"));

        let ks = KernelSelector::parse("name:node-micro");
        assert_eq!(ks.any, None);
        assert_eq!(ks.name, some_string!("node-micro"));
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, None);

        let ks = KernelSelector::parse("type:jupyter lang:r name:ir");
        assert_eq!(ks.any, None);
        assert_eq!(ks.name, some_string!("ir"));
        assert_eq!(ks.lang, some_string!("r"));
        assert_eq!(ks.r#type, some_string!("jupyter"));
        assert_eq!(ks.to_string(), "name:ir lang:r type:jupyter");
    }

    #[test]
    fn kernel_selector_matches() {
        let k = Kernel::new(
            "foo",
            KernelType::Builtin,
            &[Format::Bash, Format::Zsh],
            true,
            false,
            false,
        );

        assert!(KernelSelector::parse("foo").matches(&k));
        assert!(KernelSelector::parse("bash").matches(&k));
        assert!(KernelSelector::parse("zsh").matches(&k));
        assert!(KernelSelector::parse("name:foo").matches(&k));
        assert!(KernelSelector::parse("lang:bash").matches(&k));
        assert!(KernelSelector::parse("lang:zsh").matches(&k));
        assert!(KernelSelector::parse("name:foo lang:bash type:builtin").matches(&k));
        assert!(KernelSelector::parse("foo type:builtin").matches(&k));

        assert!(!KernelSelector::parse("quax").matches(&k));
        assert!(!KernelSelector::parse("name:quax").matches(&k));
        assert!(!KernelSelector::parse("lang:quax").matches(&k));
        assert!(!KernelSelector::parse("name:foo lang:quax").matches(&k));
        assert!(!KernelSelector::parse("name:foo lang:bar type:quax").matches(&k));
        assert!(!KernelSelector::parse("foo type:quax").matches(&k));
    }
}
