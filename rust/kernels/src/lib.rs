use std::{
    fmt,
    path::{Path, PathBuf},
};

use context::KernelContext;
use kernel::{
    common::eyre::{bail, Result},
    format::Format,
    schema::{ExecutionMessage, Node},
    Kernel, KernelInstance,
};
use kernel_bash::BashKernel;
use kernel_node::NodeKernel;
use kernel_python::PythonKernel;
use kernel_r::RKernel;
use kernel_rhai::RhaiKernel;
use kernel_tex::TexKernel;

pub mod cli;

/// Get a list of available kernels
pub async fn list() -> Vec<Box<dyn Kernel>> {
    let mut kernels = vec![
        Box::<BashKernel>::default() as Box<dyn Kernel>,
        Box::<NodeKernel>::default() as Box<dyn Kernel>,
        Box::<PythonKernel>::default() as Box<dyn Kernel>,
        Box::<RKernel>::default() as Box<dyn Kernel>,
        Box::<RhaiKernel>::default() as Box<dyn Kernel>,
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

/// A collection of kernel instances associated with a document
pub struct Kernels {
    /// The home directory of the kernels
    ///
    /// Used to start each kernel in the home directory of the document
    home: PathBuf,

    /// The kernel instances
    instances: Vec<(Box<dyn Kernel>, Box<dyn KernelInstance>)>,
}

impl Default for Kernels {
    fn default() -> Self {
        Self {
            home: std::env::current_dir().expect("should always be a current dir"),
            instances: Vec::new(),
        }
    }
}

impl fmt::Debug for Kernels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kernels({} instances)", self.instances.len())
    }
}

impl Kernels {
    /// Create a new set of kernels
    pub fn new(home: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            instances: Vec::new(),
        }
    }

    /// Get the context of each kernel instance
    pub async fn kernel_contexts(&mut self) -> Vec<KernelContext> {
        let mut contexts = Vec::new();
        for (.., instance) in self.instances.iter_mut() {
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
    async fn create_instance(&mut self, language: Option<&str>) -> Result<&mut dyn KernelInstance> {
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
        instance.start(&self.home).await?;
        self.instances.push((kernel, instance));

        let instance = self
            .instances
            .last_mut()
            .expect("should be just pushed")
            .1
            .as_mut();

        Ok(instance)
    }

    /// Get a kernel instance
    ///
    /// The `language` argument can be the name of a programming language, or
    /// the id of an existing kernel instance.
    ///
    /// If no language specified, and there is at least one kernel instance, returns the
    /// first instance.
    fn get_instance(&mut self, language: Option<&str>) -> Result<Option<&mut dyn KernelInstance>> {
        let format = language.map(Format::from_name);

        for (kernel, instance) in self.instances.iter_mut() {
            let Some(language) = language else {
                return Ok(Some(instance.as_mut()));
            };

            if instance.name() == language {
                return Ok(Some(instance.as_mut()));
            }

            if let Some(format) = &format {
                if kernel.supports_language(format) {
                    return Ok(Some(instance.as_mut()));
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
        let instance = match self.get_instance(language)? {
            Some(instance) => instance,
            None => self.create_instance(language).await?,
        };

        instance.execute(code).await
    }

    /// Evaluate a code expression in a kernel instance
    pub async fn evaluate(
        &mut self,
        code: &str,
        language: Option<&str>,
    ) -> Result<(Node, Vec<ExecutionMessage>)> {
        let instance = match self.get_instance(language)? {
            Some(instance) => instance,
            None => self.create_instance(language).await?,
        };

        instance.evaluate(code).await
    }

    /// Set a variable in the first kernel instance
    pub async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        let instance = match self.get_instance(None)? {
            Some(instance) => instance,
            None => self.create_instance(None).await?,
        };

        instance.set(name, value).await
    }

    /// Remove a variable from the kernels
    pub async fn remove(&mut self, name: &str) -> Result<()> {
        // TODO: remove from all kernels that the variable has been mirrored to
        let Some(instance) = self.get_instance(None)? else {
            bail!("No kernel instances to remove variable from")
        };

        instance.remove(name).await
    }
}
