use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use context::KernelContext;
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
    Kernel, KernelInstance, KernelVariableRequest, KernelVariableRequester, KernelVariableResponse,
};
use kernel_asciimath::AsciiMathKernel;
use kernel_bash::BashKernel;
use kernel_jinja::JinjaKernel;
use kernel_node::NodeKernel;
use kernel_python::PythonKernel;
use kernel_r::RKernel;
use kernel_rhai::RhaiKernel;
use kernel_style::StyleKernel;
use kernel_tex::TexKernel;

pub mod cli;

/// Get a list of available kernels
pub async fn list() -> Vec<Box<dyn Kernel>> {
    let mut kernels = vec![
        Box::<AsciiMathKernel>::default() as Box<dyn Kernel>,
        Box::<BashKernel>::default() as Box<dyn Kernel>,
        Box::<JinjaKernel>::default() as Box<dyn Kernel>,
        Box::<NodeKernel>::default() as Box<dyn Kernel>,
        Box::<PythonKernel>::default() as Box<dyn Kernel>,
        Box::<RKernel>::default() as Box<dyn Kernel>,
        Box::<RhaiKernel>::default() as Box<dyn Kernel>,
        Box::<StyleKernel>::default() as Box<dyn Kernel>,
        Box::<TexKernel>::default() as Box<dyn Kernel>,
    ];

    let provided_by_plugins = &mut plugins::kernels::list().await;
    kernels.append(provided_by_plugins);

    kernels
}

/// Get the default kernel (used when no language is specified)
pub fn default() -> Box<dyn Kernel> {
    Box::<RhaiKernel>::default() as Box<dyn Kernel>
}

/// An entry for a kernel instance
struct KernelInstanceEntry {
    /// The kernel that the instance is an instance of
    kernel: Box<dyn Kernel>,

    /// The name of the instance. Used to avoid needing to take
    /// a lock on the instance just to get its name (which is constant)
    name: String,

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

        Self {
            home: home.to_path_buf(),
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
                if entry.name == request.instance {
                    continue;
                }

                let mut instance = entry.instance.lock().await;
                if let Ok(Some(value)) = instance.get(&response.variable).await {
                    response.value = Some(value);
                    response.instance = Some(entry.name.clone());
                    break;
                }
            }

            if let Err(error) = sender.send(response) {
                tracing::debug!("Error sending variable response: {error}");
            }
        }

        tracing::debug!("Kernels variable request task stopped");
    }

    /// Get the context of each kernel instance
    pub async fn kernel_contexts(&mut self) -> Vec<KernelContext> {
        let mut contexts = Vec::new();
        for entry in self.instances.read().await.iter() {
            let mut instance = entry.instance.lock().await;
            contexts.push(KernelContext {
                info: instance.info().await.unwrap_or_default(),
                packages: instance.packages().await.unwrap_or_default(),
                variables: instance.list().await.unwrap_or_default(),
            })
        }
        contexts
    }

    /// Create a kernel instance
    ///
    /// The `language` argument can be the name of a kernel or a programming language
    async fn create_instance(
        &mut self,
        language: Option<&str>,
    ) -> Result<Arc<Mutex<Box<dyn KernelInstance>>>> {
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
        let name = instance.name();
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
            kernel,
            name,
            instance: instance.clone(),
        });

        Ok(instance)
    }

    /// Get a kernel instance
    ///
    /// The `language` argument can be the name of a programming language, or
    /// the id of an existing kernel instance.
    ///
    /// If no language specified, and there is at least one kernel instance, returns the
    /// first instance.
    async fn get_instance(
        &mut self,
        language: Option<&str>,
    ) -> Result<Option<Arc<Mutex<Box<dyn KernelInstance>>>>> {
        let format = language.map(Format::from_name);

        for entry in self.instances.read().await.iter() {
            let Some(language) = language else {
                return Ok(Some(entry.instance.clone()));
            };

            if entry.name == language {
                return Ok(Some(entry.instance.clone()));
            }

            if let Some(format) = &format {
                if entry.kernel.supports_language(format) {
                    return Ok(Some(entry.instance.clone()));
                }
            }
        }

        Ok(None)
    }

    /// Execute some code in a kernel instance
    pub async fn execute(
        &mut self,
        code: &str,
        language: Option<&str>,
    ) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        let instance = match self.get_instance(language).await? {
            Some(instance) => instance,
            None => self.create_instance(language).await?,
        };

        let mut instance = instance.lock().await;
        instance.execute(code).await
    }

    /// Evaluate a code expression in a kernel instance
    pub async fn evaluate(
        &mut self,
        code: &str,
        language: Option<&str>,
    ) -> Result<(Node, Vec<ExecutionMessage>)> {
        let instance = match self.get_instance(language).await? {
            Some(instance) => instance,
            None => self.create_instance(language).await?,
        };

        let mut instance = instance.lock().await;
        instance.evaluate(code).await
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
        let instance = match self.get_instance(None).await? {
            Some(instance) => instance,
            None => self.create_instance(None).await?,
        };

        let mut instance = instance.lock().await;
        instance.set(name, value).await
    }

    /// Remove a variable from the kernels
    pub async fn remove(&mut self, name: &str) -> Result<()> {
        // TODO: remove from all kernels that the variable has been mirrored to
        let Some(instance) = self.get_instance(None).await? else {
            bail!("No kernel instances to remove variable from")
        };

        let mut instance = instance.lock().await;
        instance.remove(name).await
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

    // Test on-demand variable requests from to Jinja kernel
    #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
    async fn variables_to_jinja() -> Result<()> {
        let mut kernels = Kernels::new_here();

        let (.., messages) = kernels.execute("let a = 123", Some("rhai")).await?;
        assert_eq!(messages, vec![]);

        let (node, messages) = kernels.evaluate("a * 2", Some("jinja")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, Node::Integer(246));

        let (node, messages) = kernels.execute("{{a * 3}}", Some("jinja")).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(node, vec![Node::String("369".to_string())]);

        let (node, messages) = kernels.execute("{{foo + 4}}", Some("jinja")).await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].level, MessageLevel::Exception);
        assert_eq!(messages[0].message, "invalid operation: tried to use + operator on unsupported types undefined and number (in <string>:1)");
        assert_eq!(node, vec![Node::String("{{foo + 4}}".to_string())]);

        Ok(())
    }
}
