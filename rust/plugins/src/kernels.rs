use std::path::Path;

use serde::{Deserialize, Serialize};

use codec::schema::ExecutionBounds;
use common::{
    async_trait::async_trait,
    eyre::{Result, bail},
};
use kernel::{
    Kernel, KernelAvailability, KernelInstance, KernelInterrupt, KernelKill, KernelProvider,
    KernelTerminate,
    format::Format,
    schema::{ExecutionMessage, Node, SoftwareApplication, SoftwareSourceCode, Variable},
};

use crate::{Plugin, PluginEnabled, PluginInstance, PluginStatus, plugins};

/// A kernel provided by a plugin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginKernel {
    /// The name of the kernel
    name: String,

    /// The languages that the kernel supports
    #[serde(default)]
    languages: Vec<Format>,

    /// Does the kernel support the interrupt signal?
    #[serde(default)]
    interrupt: KernelInterrupt,

    /// Does the kernel support the terminate signal?
    #[serde(default)]
    terminate: KernelTerminate,

    /// Does the kernel support the kill signal?
    #[serde(default)]
    kill: KernelKill,

    /// Execution bounds supported by the kernel
    #[serde(default)]
    bounds: Vec<ExecutionBounds>,

    /// The plugin that provides this kernel
    ///
    /// Used to be able to create a plugin instance, which in
    /// turn is used to create a kernel instance.
    #[serde(skip)]
    plugin: Option<Plugin>,
}

impl PluginKernel {
    /// Bind a plugin to this kernel so that it can be started (by starting the plugin first)
    pub fn bind(&mut self, plugin: &Plugin) {
        self.plugin = Some(plugin.clone());
    }
}

impl Kernel for PluginKernel {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn provider(&self) -> KernelProvider {
        match &self.plugin {
            Some(plugin) => {
                let mut name = plugin.name.clone();
                if plugin.linked {
                    name += " (linked)";
                }
                KernelProvider::Plugin(name)
            }
            None => KernelProvider::Plugin("unknown".to_string()),
        }
    }

    fn availability(&self) -> KernelAvailability {
        match &self.plugin {
            Some(plugin) => match plugin.availability() {
                (
                    PluginStatus::InstalledLatest(..) | PluginStatus::InstalledOutdated(..),
                    PluginEnabled::Yes,
                ) => KernelAvailability::Available,

                (
                    PluginStatus::InstalledLatest(..) | PluginStatus::InstalledOutdated(..),
                    PluginEnabled::No,
                ) => KernelAvailability::Disabled,

                (PluginStatus::Installable, _) => KernelAvailability::Installable,

                _ => KernelAvailability::Unavailable,
            },
            None => KernelAvailability::Unavailable,
        }
    }

    fn supports_languages(&self) -> Vec<Format> {
        self.languages.clone()
    }

    fn supports_interrupt(&self) -> KernelInterrupt {
        self.interrupt
    }

    fn supports_terminate(&self) -> KernelTerminate {
        self.terminate
    }

    fn supports_kill(&self) -> KernelKill {
        self.kill
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        self.bounds.clone()
    }

    fn create_instance(&self, bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        let Some(plugin) = self.plugin.clone() else {
            bail!("No plugin associated with this plugin kernel!")
        };

        if !self.bounds.contains(&bounds) {
            bail!("Execution bounds `{bounds}` are not supported by this plugin kernel")
        };

        Ok(Box::new(PluginKernelInstance::new(self.clone(), plugin)))
    }
}

/// An instance of a plugin kernel
pub struct PluginKernelInstance {
    /// The kernel specification for this instance
    kernel: PluginKernel,

    /// The plugin that provides the kernel
    plugin: Plugin,

    /// The plugin instance started when this kernel is started
    plugin_instance: Option<PluginInstance>,

    /// The id of the kernel instance on the plugin instance
    kernel_instance: Option<String>,
}

impl PluginKernelInstance {
    fn new(kernel: PluginKernel, plugin: Plugin) -> Self {
        Self {
            kernel,
            plugin,
            plugin_instance: None,
            kernel_instance: None,
        }
    }

    /// Get the plugin instance and the name of the kernel instance in that plugin instance
    fn details(&mut self) -> Result<(&mut PluginInstance, String)> {
        match (self.plugin_instance.as_mut(), self.kernel_instance.as_ref()) {
            (Some(instance), Some(name)) => Ok((instance, name.clone())),
            _ => bail!("Kernel instance has no plugin instance, "),
        }
    }
}

#[async_trait]
impl KernelInstance for PluginKernelInstance {
    fn id(&self) -> &str {
        // This should only be called once the kernel has stated and
        // has a id. But in case it has not, and because this method
        // is infallible, default to "noid".
        match &self.kernel_instance {
            Some(id) => id.as_str(),
            None => "noid",
        }
    }

    async fn start(&mut self, _directory: &Path) -> Result<()> {
        #[derive(Serialize)]
        struct Params {
            kernel: String,
        }

        #[derive(Deserialize)]
        struct Result {
            instance: String,
        }

        // TODO: consider starting in directory
        let mut plugin_instance = self.plugin.start(None).await?;

        let Result {
            instance: kernel_instance,
        } = plugin_instance
            .call(
                "kernel_start",
                Params {
                    kernel: self.kernel.name(),
                },
            )
            .await?;

        self.plugin_instance = Some(plugin_instance);
        self.kernel_instance = Some(kernel_instance);

        Ok(())
    }

    #[allow(dependency_on_unit_never_type_fallback)]
    async fn stop(&mut self) -> Result<()> {
        #[derive(Serialize)]
        struct Params {
            instance: String,
        }

        if let Some(plugin_instance) = self.plugin_instance.as_mut() {
            if let Some(instance) = self.kernel_instance.take() {
                plugin_instance
                    .call::<_, ()>("kernel_stop", Params { instance })
                    .await?;
            }
            plugin_instance.stop().await?;
        };

        self.plugin_instance = None;
        self.kernel_instance = None;

        Ok(())
    }

    // In the following methods, for brevity, we use:
    //  - `plugin` to refer to `self.plugin_instance`
    //  - `instance` to refer to `self.kernel_instance`

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        #[derive(Serialize)]
        struct Params {
            code: String,
            instance: String,
        }

        #[derive(Deserialize)]
        struct Result {
            outputs: Vec<Node>,
            messages: Vec<ExecutionMessage>,
        }

        let (plugin, instance) = self.details()?;
        let result: Result = plugin
            .call(
                "kernel_execute",
                Params {
                    code: code.to_string(),
                    instance,
                },
            )
            .await?;

        Ok((result.outputs, result.messages))
    }

    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)> {
        #[derive(Serialize)]
        struct Params {
            code: String,
            instance: String,
        }

        #[derive(Deserialize)]
        struct Result {
            output: Node,
            messages: Vec<ExecutionMessage>,
        }

        let (plugin, instance) = self.details()?;
        let result: Result = plugin
            .call(
                "kernel_evaluate",
                Params {
                    code: code.to_string(),
                    instance,
                },
            )
            .await?;

        Ok((result.output, result.messages))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        #[derive(Serialize)]
        struct Params {
            instance: String,
        }

        let (plugin, instance) = self.details()?;
        plugin.call("kernel_info", Params { instance }).await
    }

    async fn packages(&mut self) -> Result<Vec<SoftwareSourceCode>> {
        #[derive(Serialize)]
        struct Params {
            instance: String,
        }

        let (plugin, instance) = self.details()?;
        plugin.call("kernel_packages", Params { instance }).await
    }

    async fn list(&mut self) -> Result<Vec<Variable>> {
        #[derive(Serialize)]
        struct Params {
            instance: String,
        }

        let (plugin, instance) = self.details()?;
        plugin.call("kernel_list", Params { instance }).await
    }

    async fn get(&mut self, name: &str) -> Result<Option<Node>> {
        #[derive(Serialize)]
        struct Params {
            name: String,
            instance: String,
        }

        let (plugin, instance) = self.details()?;
        plugin
            .call(
                "kernel_get",
                Params {
                    name: name.to_string(),
                    instance,
                },
            )
            .await
    }

    async fn set(&mut self, name: &str, value: &Node) -> Result<()> {
        #[derive(Serialize)]
        struct Params {
            name: String,
            value: Node,
            instance: String,
        }

        let (plugin, instance) = self.details()?;
        plugin
            .call(
                "kernel_set",
                Params {
                    name: name.to_string(),
                    value: value.clone(),
                    instance,
                },
            )
            .await
    }

    async fn remove(&mut self, name: &str) -> Result<()> {
        #[derive(Serialize)]
        struct Params {
            name: String,
            instance: String,
        }

        let (plugin, instance) = self.details()?;
        plugin
            .call(
                "kernel_remove",
                Params {
                    name: name.to_string(),
                    instance,
                },
            )
            .await
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        bail!("Replicating plugin kernel is not supported")
    }
}

/// List all the kernels provided by plugins
pub async fn list() -> Vec<Box<dyn Kernel>> {
    plugins()
        .await
        .into_iter()
        .flat_map(|plugin| plugin.kernels())
        .collect()
}
