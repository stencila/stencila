use std::{
    env, fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use kernel::{
    common::{
        eyre::{bail, Result},
        tokio::{
            self,
            sync::{broadcast, mpsc, Mutex, RwLock},
        },
        tracing,
    },
    format::Format,
    schema::{ExecutionMessage, Node},
    Kernel, KernelForks, KernelInstance, KernelVariableRequest, KernelVariableRequester,
    KernelVariableResponse,
};
use kernel_asciimath::AsciiMathKernel;
use kernel_bash::BashKernel;
use kernel_graphviz::GraphvizKernel;
use kernel_jinja::JinjaKernel;
use kernel_mermaid::MermaidKernel;
use kernel_nodejs::NodeJsKernel;
use kernel_python::PythonKernel;
use kernel_quickjs::QuickJsKernel;
use kernel_r::RKernel;
use kernel_style::StyleKernel;
use kernel_tex::TexKernel;

#[cfg(feature = "kernel-rhai")]
use kernel_rhai::RhaiKernel;

pub use kernel::{KernelAvailability, KernelProvider, KernelSpecification, KernelType};

pub mod cli;

/// Get a list of available kernels
pub async fn list() -> Vec<Box<dyn Kernel>> {
    let mut kernels = vec![
        // First so that it gets used for `js` rather than `NodeJsKernel`
        Box::<QuickJsKernel>::default() as Box<dyn Kernel>,
        Box::<AsciiMathKernel>::default() as Box<dyn Kernel>,
        Box::<BashKernel>::default() as Box<dyn Kernel>,
        Box::<GraphvizKernel>::default() as Box<dyn Kernel>,
        Box::<JinjaKernel>::default() as Box<dyn Kernel>,
        Box::<MermaidKernel>::default() as Box<dyn Kernel>,
        Box::<NodeJsKernel>::default() as Box<dyn Kernel>,
        Box::<PythonKernel>::default() as Box<dyn Kernel>,
        Box::<RKernel>::default() as Box<dyn Kernel>,
        Box::<StyleKernel>::default() as Box<dyn Kernel>,
        Box::<TexKernel>::default() as Box<dyn Kernel>,
    ];

    #[cfg(feature = "kernel-rhai")]
    kernels.push(Box::<RhaiKernel>::default() as Box<dyn Kernel>);

    let provided_by_plugins = &mut plugins::kernels::list().await;
    kernels.append(provided_by_plugins);

    kernels
}

/// Get a kernel by name
pub async fn get(name: &str) -> Option<Box<dyn Kernel>> {
    let name = name.to_lowercase();

    for kernel in list().await {
        if kernel.name().to_lowercase() == name {
            return Some(kernel);
        }
    }

    None
}

/// Get the default kernel (used when no language is specified)
pub fn default() -> Box<dyn Kernel> {
    Box::<QuickJsKernel>::default() as Box<dyn Kernel>
}

/// An entry for a kernel instance
struct KernelInstanceEntry {
    /// The kernel that the instance is an instance of
    kernel: Arc<Box<dyn Kernel>>,

    /// The id of the kernel instance. Used to avoid needing to take
    /// a lock on the instance just to get its id (which is constant)
    id: String,

    /// The instance itself
    instance: Arc<Mutex<Box<dyn KernelInstance>>>,
}

type KernelInstances = Arc<RwLock<Vec<KernelInstanceEntry>>>;

/// A collection of kernel instances associated with a document
pub struct Kernels {
    /// The home directory of the kernels
    ///
    /// Used to start each kernel in the home directory of the document
    home: PathBuf,

    /// The kernel instances
    instances: KernelInstances,

    /// A sender of requests from kernels for variables
    variable_request_sender: KernelVariableRequester,

    /// A sender for responses to kernels for variables
    variable_response_sender: broadcast::Sender<KernelVariableResponse>,
}

impl fmt::Debug for Kernels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kernels")
    }
}

impl Kernels {
    /// Create a new set of kernels
    pub fn new(home: &Path) -> Self {
        let instances = KernelInstances::default();

        let (variable_request_sender, variable_request_receiver) = mpsc::unbounded_channel();
        let (variable_response_sender, ..) = broadcast::channel(32);

        let instances_clone = instances.clone();
        let variable_response_sender_clone = variable_response_sender.clone();
        tokio::spawn(async move {
            Self::variable_requests_task(
                instances_clone,
                variable_request_receiver,
                variable_response_sender_clone,
            )
            .await
        });

        let home = if home.to_string_lossy() == "" {
            match env::current_dir() {
                Ok(dir) => dir,
                Err(error) => {
                    tracing::error!("Unable to get current working dir: {error}");
                    home.to_path_buf()
                }
            }
        } else {
            home.to_path_buf()
        };

        Self {
            home,
            instances,
            variable_request_sender,
            variable_response_sender,
        }
    }

    /// Create a new set of kernels in the current working directory
    pub fn new_here() -> Self {
        let path = std::env::current_dir().expect("should always be a current dir");
        Self::new(&path)
    }

    /// A task to handle requests from kernels for variables in other contexts
    async fn variable_requests_task(
        instances: KernelInstances,
        mut receiver: mpsc::UnboundedReceiver<KernelVariableRequest>,
        sender: broadcast::Sender<KernelVariableResponse>,
    ) {
        tracing::debug!("Kernels variable request task started");

        while let Some(request) = receiver.recv().await {
            tracing::trace!("Received request for variable `{}`", request.variable);

            let mut response = KernelVariableResponse {
                variable: request.variable,
                ..Default::default()
            };
            for entry in instances.read().await.iter() {
                // If the candidate instance is the same as the request instance then
                // skip - because unnecessary because likely to cause deadlock in
                // next step.
                if entry.id == request.instance {
                    continue;
                }

                let mut instance = entry.instance.lock().await;
                if let Ok(Some(value)) = instance.get(&response.variable).await {
                    response.value = Some(value);
                    response.instance = Some(entry.id.clone());
                    break;
                }
            }

            if let Err(error) = sender.send(response) {
                tracing::debug!("Error sending variable response: {error}");
            }
        }

        tracing::debug!("Kernels variable request task stopped");
    }

    /// Create a kernel instance
    ///
    /// The `language` argument can be the name of a kernel or a programming language.
    /// If `language` is `None` then the default language is used.
    pub async fn create_instance(
        &mut self,
        language: Option<&str>,
    ) -> Result<Arc<Mutex<Box<dyn KernelInstance>>>> {
        tracing::debug!(
            "Creating kernel instance for language {:?}",
            language.unwrap_or_default()
        );

        let kernel = match language {
            Some(language) => 'block: {
                let format = Format::from_name(language);

                for kernel in list().await {
                    if kernel.name() == language {
                        break 'block kernel;
                    }

                    if kernel.supports_language(&format) && kernel.is_available() {
                        break 'block kernel;
                    }
                }

                bail!("No kernel available with name, or that supports language, `{language}`")
            }
            None => default(),
        };

        let mut instance = kernel.create_instance()?;
        let id = instance.id().to_string();
        if kernel.supports_variable_requests() {
            instance.variable_channel(
                self.variable_request_sender.clone(),
                self.variable_response_sender.subscribe(),
            );
        }
        instance.start(&self.home).await?;
        let instance = Arc::new(Mutex::new(instance));

        let mut instances = self.instances.write().await;
        instances.push(KernelInstanceEntry {
            kernel: Arc::new(kernel),
            id,
            instance: instance.clone(),
        });

        Ok(instance)
    }

    /// Add a kernel to the set of instances
    ///
    /// It is assumed that the instance is already started (e.g. is a fork).
    /// If the kernel supports variable requests then it will connected to the
    /// variable channel.
    pub async fn add_instance(
        &mut self,
        kernel: Arc<Box<dyn Kernel>>,
        mut instance: Box<dyn KernelInstance>,
    ) -> Result<()> {
        if kernel.supports_variable_requests() {
            instance.variable_channel(
                self.variable_request_sender.clone(),
                self.variable_response_sender.subscribe(),
            );
        }

        let id = instance.id().to_string();
        let instance = Arc::new(Mutex::new(instance));

        let mut instances = self.instances.write().await;
        instances.push(KernelInstanceEntry {
            kernel,
            id,
            instance,
        });

        Ok(())
    }

    /// Determine if the kernels set has an instance with the given id
    pub async fn has_instance(&mut self, id: &str) -> bool {
        self.instances
            .read()
            .await
            .iter()
            .any(|entry| entry.id == id)
    }

    /// Get the instance with the given id
    pub async fn get_instance(&mut self, id: &str) -> Option<Arc<Mutex<Box<dyn KernelInstance>>>> {
        self.instances
            .read()
            .await
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.instance.clone())
    }

    /// Get a kernel instance for a language
    ///
    /// The `language` argument can be the name of a programming language, or
    /// the id of an existing kernel instance.
    async fn get_instance_for(
        &mut self,
        language: &str,
    ) -> Result<Option<Arc<Mutex<Box<dyn KernelInstance>>>>> {
        let format = Format::from_name(language);

        for entry in self.instances.read().await.iter() {
            if entry.id == language {
                return Ok(Some(entry.instance.clone()));
            }

            if entry.kernel.supports_language(&format) {
                return Ok(Some(entry.instance.clone()));
            }
        }

        Ok(None)
    }

    /// Get the first kernel instance of [`KernelType::Programming`]
    ///
    /// If there is not yet a kernel instance for an executable, programming language
    /// then falls back to creating an instance of the default kernel.
    async fn get_instance_programming(&mut self) -> Result<Arc<Mutex<Box<dyn KernelInstance>>>> {
        for entry in self.instances.read().await.iter() {
            if matches!(entry.kernel.r#type(), KernelType::Programming) {
                return Ok(entry.instance.clone());
            }
        }

        self.create_instance(None).await
    }

    /// Get a reference to each of the kernel instances
    pub async fn instances(&self) -> Vec<Arc<Mutex<Box<dyn KernelInstance>>>> {
        self.instances
            .read()
            .await
            .iter()
            .map(|entry| entry.instance.clone())
            .collect()
    }

    /// Execute some code in a kernel instance
    pub async fn execute(
        &mut self,
        code: &str,
        language: Option<&str>,
    ) -> Result<(Vec<Node>, Vec<ExecutionMessage>, String)> {
        let instance = match language {
            Some(language) => match self.get_instance_for(language).await? {
                Some(instance) => instance,
                None => self.create_instance(Some(language)).await?,
            },
            None => self.get_instance_programming().await?,
        };

        let mut instance = instance.lock().await;
        let (nodes, messages) = instance.execute(code).await?;
        let id = instance.id().to_string();

        Ok((nodes, messages, id))
    }

    /// Evaluate a code expression in a kernel instance
    pub async fn evaluate(
        &mut self,
        code: &str,
        language: Option<&str>,
    ) -> Result<(Node, Vec<ExecutionMessage>, String)> {
        let instance = match language {
            Some(language) => match self.get_instance_for(language).await? {
                Some(instance) => instance,
                None => self.create_instance(Some(language)).await?,
            },
            None => self.get_instance_programming().await?,
        };

        let mut instance = instance.lock().await;
        let (node, messages) = instance.evaluate(code).await?;
        let id = instance.id().to_string();

        Ok((node, messages, id))
    }

    /// Get a variable from the kernels
    ///
    /// Currently just iterates over kernels until the variable is found (if at all).
    pub async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        for entry in self.instances.read().await.iter() {
            let mut instance = entry.instance.lock().await;
            if let Some(value) = instance.get(name).await? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    /// Set a variable in the first kernel instance
    pub async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        let instance = self.get_instance_programming().await?;

        let mut instance = instance.lock().await;
        instance.set(name, value).await
    }

    /// Remove a variable from the kernels
    pub async fn remove(&mut self, name: &str) -> Result<()> {
        let instance = self.get_instance_programming().await?;

        let mut instance = instance.lock().await;
        instance.remove(name).await
    }

    /// Whether all kernels in the set support forking
    pub async fn supports_forks(&self) -> bool {
        self.instances
            .read()
            .await
            .iter()
            .all(|entry| matches!(entry.kernel.supports_forks(), KernelForks::Yes))
    }

    /// Fork the kernels
    ///
    /// Creates a new [`Kernels`] set with a fork of each current instance.
    /// Errors if any of the forks fails (i.e. a complete fork is not possible).
    pub async fn fork(&self) -> Result<Self> {
        let mut kernels = Self::new(&self.home);
        for entry in self.instances.read().await.iter() {
            let kernel = entry.kernel.clone();
            let instance = entry.instance.lock().await.fork().await?;
            kernels.add_instance(kernel, instance).await?;
        }
        Ok(kernels)
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::{
        common::tokio,
        schema::{MessageLevel, Node},
    };

    use super::*;

    /// Test on-demand variable requests from Rhai to Jinja kernel
    ///
    /// Multithreaded test needed so that variable request does not hang.
    #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
    async fn variables_to_jinja() -> Result<()> {
        let mut kernels = Kernels::new_here();

        let (_, messages, ..) = kernels.execute("var a = 123", Some("js")).await?;
        assert_eq!(messages, vec![]);

        let (node, messages, ..) = kernels.evaluate("a * 2", Some("jinja")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, Node::Integer(246));

        let (node, messages, ..) = kernels.execute("{{a * 3}}", Some("jinja")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, vec![Node::String("369".to_string())]);

        let (node, messages, ..) = kernels.execute("{{foo + 4}}", Some("jinja")).await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].level, MessageLevel::Exception);
        assert_eq!(messages[0].message, "invalid operation: tried to use + operator on unsupported types undefined and number (in <string>:1)");
        assert_eq!(node, vec![Node::String("{{foo + 4}}".to_string())]);

        Ok(())
    }

    /// Test forking a set of kernels
    ///
    /// The `evaluate` calls using Jinja test variable connections
    /// are also "forked".
    #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
    async fn fork() -> Result<()> {
        let mut kernels = Kernels::new_here();
        kernels.execute("var a = 1", Some("js")).await?;
        kernels.execute("var b = 2", Some("js")).await?;

        let mut fork = kernels.fork().await?;
        fork.execute("a = 11", Some("js")).await?;
        fork.execute("b = 22", Some("js")).await?;
        fork.execute("var c = 33", Some("js")).await?;

        // In original kernels post forking

        let (node, messages, ..) = kernels.evaluate("a", Some("js")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, Node::Integer(1));

        let node = kernels.get("b").await?;
        assert_eq!(node, Some(Node::Integer(2)));

        let node = kernels.get("c").await?;
        assert_eq!(node, None);

        let (node, messages, ..) = kernels.evaluate("a + b", Some("jinja")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, Node::Integer(3));

        let (nodes, messages, ..) = kernels.execute("{{ c }}", Some("jinja")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(nodes[0], Node::String(String::new()));

        // In fork

        let (node, messages, ..) = fork.evaluate("a", Some("js")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, Node::Integer(11));

        let node = fork.get("b").await?;
        assert_eq!(node, Some(Node::Integer(22)));

        let node = fork.get("c").await?;
        assert_eq!(node, Some(Node::Integer(33)));

        let (node, messages, ..) = fork.evaluate("a + b + c", Some("jinja")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, Node::Integer(66));

        Ok(())
    }
}
