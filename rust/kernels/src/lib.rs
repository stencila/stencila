use std::fmt;

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

pub mod cli;

/// Get a list of available kernels
pub fn list() -> Vec<Box<dyn Kernel>> {
    vec![
        Box::<BashKernel>::default() as Box<dyn Kernel>,
        Box::<NodeKernel>::default() as Box<dyn Kernel>,
        Box::<PythonKernel>::default() as Box<dyn Kernel>,
        Box::<RKernel>::default() as Box<dyn Kernel>,
        Box::<RhaiKernel>::default() as Box<dyn Kernel>,
    ]
}

/// Get the default kernel (used when no language is specified)
pub fn default() -> Box<dyn Kernel> {
    Box::<RhaiKernel>::default() as Box<dyn Kernel>
}

/// A collection of kernel instances associated with a document
#[derive(Default)]
pub struct Kernels {
    /// The kernel instances
    instances: Vec<(Box<dyn Kernel>, Box<dyn KernelInstance>)>,
}

impl fmt::Debug for Kernels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kernels({} instances)", self.instances.len())
    }
}

impl Kernels {
    /// Create a kernel instance
    ///
    /// The `language` argument can be the name of a programming language, or
    /// the id of an existing kernel.
    async fn create_instance(&mut self, language: Option<&str>) -> Result<&mut dyn KernelInstance> {
        let kernel = match language {
            Some(language) => 'block: {
                let format = Format::from_name(language).ok();

                for kernel in list() {
                    if kernel.id() == language {
                        break 'block kernel;
                    }

                    if let Some(format) = format {
                        if kernel.supports_language(&format) && kernel.is_available() {
                            break 'block kernel;
                        }
                    }
                }

                bail!("No kernel available with id, or that supports language, `{language}`")
            }
            None => default(),
        };

        let mut instance = kernel.create_instance()?;
        instance.start_here().await?; // TODO: start elsewhere?
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
        let format = language.and_then(|lang| Format::from_name(lang).ok());

        for (kernel, instance) in self.instances.iter_mut() {
            let Some(language) = language else {
                return Ok(Some(instance.as_mut()));
            };

            if instance.id() == language {
                return Ok(Some(instance.as_mut()));
            }

            if let Some(format) = format {
                if kernel.supports_language(&format) {
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
