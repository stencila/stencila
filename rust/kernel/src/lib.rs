// use crate::utils::uuids;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use eyre::{bail, Result};
use formats::Format;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use stencila_schema::{CodeError, Node};
use strum::Display;
use tokio::sync::{broadcast, mpsc};
use utils::some_box_string;
use uuids::uuid_family;

// Re-export for the convenience of crates that implement `KernelTrait`
pub use ::async_trait;
pub use eyre;
pub use serde;
pub use stencila_schema;
pub use tokio;

/// The type of kernel
///
/// At present this is mainly for informational purposes.
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
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
#[serde(rename_all = "camelCase")]
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
    /// These should be the `name` of one of the `Format`s defined in
    /// the `formats` crate. Many kernels only support one language.
    pub languages: Vec<String>,
}

impl Kernel {
    // Create a new kernel specification
    pub fn new(name: &str, r#type: KernelType, languages: &[&str]) -> Self {
        let languages = languages
            .iter()
            .map(|language| language.to_string())
            .collect();
        Self {
            name: name.to_string(),
            r#type,
            languages,
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
#[derive(Debug, PartialEq, Clone, Serialize, Display)]
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
pub struct KernelSelector {
    /// A string that will match against the kernel `name` or any of its `languages`
    pub any: Option<String>,

    /// A string that will match against the kernel `name`
    pub name: Option<String>,

    /// A string that will match against any of a kernel's `languages`
    pub lang: Option<String>,

    /// A string that will match against the kernel `type`
    pub r#type: Option<String>,
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
        write!(formatter, "{}", str.trim())
    }
}

impl KernelSelector {
    /// Create a new `KernelSelector`
    pub fn new(name: Option<String>, lang: Option<String>, r#type: Option<String>) -> Self {
        Self {
            any: None,
            name,
            lang,
            r#type,
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
            for kernel_lang in &kernel.languages {
                if lang.to_lowercase() == kernel_lang.to_lowercase()
                    || (format != Format::Unknown && format == formats::match_name(kernel_lang))
                {
                    lang_matched = true;
                    break;
                }
            }
            matched &= lang_matched;
        } else if let (false, Some(lang)) = (matched, &self.any) {
            let format = formats::match_name(lang);
            for kernel_lang in &kernel.languages {
                if lang.to_lowercase() == kernel_lang.to_lowercase()
                    || (format != Format::Unknown && format == formats::match_name(kernel_lang))
                {
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

/// A [`mpsc::channel`] sender to cancel the task
///
/// Used to send a cancellation request to the the kernel that is running the task.
pub type TaskCanceller = mpsc::Sender<()>;

/// A task running in a [`Kernel`]
#[derive(Debug, Clone, Serialize)]
pub struct Task {
    /// The uuid of the task
    pub id: TaskId,

    /// The time that the task was created by the kernel
    pub created: DateTime<Utc>,

    /// The time that the task was started by the kernel
    ///
    /// If the task was placed on a queue then this is expected to be after `created`.
    /// In this case, the task may also be cancelled before it is started.
    pub started: Option<DateTime<Utc>>,

    /// The time that the task ended (if it has)
    pub finished: Option<DateTime<Utc>>,

    /// The time that the task was cancelled
    pub cancelled: Option<DateTime<Utc>>,

    /// The result of the task (if it was completed immediately)
    pub result: Option<TaskResult>,

    /// The result sender for the task (if it was started asynchronously)
    #[serde(skip)]
    pub sender: Option<TaskSender>,

    /// The canceller for the task (may be set after the task is started)
    #[serde(skip)]
    pub canceller: Option<TaskCanceller>,
}

impl Task {
    /// Create a task
    pub fn create(sender: Option<TaskSender>, canceller: Option<TaskCanceller>) -> Self {
        Self {
            id: TaskId::new(),
            created: Utc::now(),
            started: None,
            finished: None,
            cancelled: None,
            result: None,
            sender,
            canceller,
        }
    }

    /// Start a task
    pub fn start(sender: Option<TaskSender>, canceller: Option<TaskCanceller>) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            created: now,
            started: Some(now),
            finished: None,
            cancelled: None,
            result: None,
            sender,
            canceller,
        }
    }

    /// Start a synchronous task
    pub fn start_sync() -> Self {
        Self::start(None, None)
    }

    /// Start an asynchronous task
    pub fn start_async(sender: TaskSender) -> Self {
        Self::start(Some(sender), None)
    }

    /// Is the task an async task?
    pub fn is_async(&self) -> bool {
        self.sender.is_some()
    }

    /// Is the task able to be cancelled?
    pub fn is_cancellable(&self) -> bool {
        self.canceller.is_some()
    }

    /// Is the task finished or cancelled?
    pub fn is_done(&self) -> bool {
        self.finished.is_some() || self.cancelled.is_some()
    }

    /// Subscribe to the task
    pub fn subscribe(&self) -> Result<TaskReceiver> {
        match self.sender.as_ref() {
            Some(sender) => Ok(sender.subscribe()),
            None => bail!("Task is sync, so can not be subscribed to"),
        }
    }

    /// The task did finish
    pub fn finished(&mut self, result: TaskResult) {
        if self.cancelled.is_some() {
            // The task can be cancelled but we still get a partial result
            // (because, for example the stdout and stderr have been captured).
            // In that case, store the result, but don't update anything else.
            self.result = Some(result);
        } else if let Some(finished) = self.finished {
            // Log an error because this shouldn't really ever happen but
            // don't returns an error because its not fatal if it does
            tracing::error!("Task already finished at `{}`", finished);
        } else {
            self.result = Some(result);
            self.finished = Some(Utc::now());
            self.canceller = None; // Ensure finish of the async cancellation task
        }
    }

    /// The task was cancelled (potentially with partial results)
    pub fn cancelled(&mut self, result: Option<TaskResult>) {
        if let Some(cancelled) = self.cancelled {
            // Log an error because this shouldn't really ever happen but
            // don't returns an error because its not fatal if it does
            tracing::error!("Task was already cancelled at `{}`", cancelled);
        } else {
            self.result = result;
            self.cancelled = Some(Utc::now());
            self.canceller = None; // Ensure finish of the async cancellation task
        }
    }

    /// Cancel the task
    pub async fn cancel(&mut self) -> Result<()> {
        if let Some(finished) = self.finished {
            // Warn if the task already finished
            tracing::warn!("Task already finished at `{}`", finished);
            Ok(())
        } else if let Some(cancelled) = self.cancelled {
            // Just debug here since this is not really an error or warning
            // (e.g. another client cancelled)
            tracing::debug!("Task was already cancelled at `{}`", cancelled);
            Ok(())
        } else if let Some(canceller) = &mut self.canceller {
            if let Err(..) = canceller.send(()).await {
                tracing::debug!("Cancellation receiver for task `{}` dropped", self.id);
            };
            self.cancelled = Some(Utc::now());
            self.canceller = None; // Ensure finish of the async cancellation task
            Ok(())
        } else {
            bail!("Task `{}` is not cancellable", self.id)
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
            self.finished(result.clone());
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
    fn spec(&self) -> Kernel;

    /// Is the kernel available on the current machine?
    async fn is_available(&self) -> bool {
        true
    }

    /// Can the kernel interrupt tasks?
    ///
    /// Some kernels listen for an interrupt signal (`SINGIT` on POSIX) to
    /// cancel long running tasks.
    async fn is_interruptable(&self) -> bool {
        false
    }

    /// Can the kernel be forked?
    ///
    /// Some kernels can be "forked" to support parallel execution. On POSIX
    /// this is generally implemented using the `fork` system call.
    async fn is_forkable(&self) -> bool {
        false
    }

    /// Start the kernel
    ///
    /// Will usually be overridden by [`KernelTrait`] implementations.
    async fn start(&mut self) -> Result<()> {
        Ok(())
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
    /// outputs and messages (and are not interested in task duration or cancellation).
    async fn exec(&mut self, code: &str) -> Result<(TaskOutputs, TaskMessages)> {
        let mut task = self.exec_sync(code).await?;
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
    async fn exec_sync(&mut self, code: &str) -> Result<Task>;

    /// Execute code in the kernel asynchronously
    ///
    /// Should be overridden by [`KernelTrait`] implementations that are cancellable.
    /// The default implementation simply calls `exec_sync`.
    async fn exec_async(&mut self, code: &str) -> Result<Task> {
        self.exec_sync(code).await
    }

    /// Fork the kernel and execute code in the fork
    ///
    /// Should be overridden by [`KernelTrait`] implementations that are forkable.
    /// The default implementation errors because code marked as `@pure` should not
    /// be executed in the main kernel in case it has side-effects (e.g. assigning
    /// temporary variables) which are intended to be ignored.
    async fn exec_fork(&mut self, _code: &str) -> Result<Task> {
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
        let k = Kernel::new("foo", KernelType::Builtin, &["bar", "baz"]);

        assert!(KernelSelector::parse("foo").matches(&k));
        assert!(KernelSelector::parse("bar").matches(&k));
        assert!(KernelSelector::parse("baz").matches(&k));
        assert!(KernelSelector::parse("name:foo").matches(&k));
        assert!(KernelSelector::parse("lang:bar").matches(&k));
        assert!(KernelSelector::parse("lang:baz").matches(&k));
        assert!(KernelSelector::parse("name:foo lang:bar type:builtin").matches(&k));
        assert!(KernelSelector::parse("foo type:builtin").matches(&k));

        assert!(!KernelSelector::parse("quax").matches(&k));
        assert!(!KernelSelector::parse("name:quax").matches(&k));
        assert!(!KernelSelector::parse("lang:quax").matches(&k));
        assert!(!KernelSelector::parse("name:foo lang:quax").matches(&k));
        assert!(!KernelSelector::parse("name:foo lang:bar type:quax").matches(&k));
        assert!(!KernelSelector::parse("foo type:quax").matches(&k));
    }
}
