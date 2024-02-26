use common::{
    eyre::Result,
    serde::{Deserialize, Serialize},
};
use kernel::{
    format::Format, Kernel, KernelAvailability, KernelForks, KernelInstance, KernelInterrupt,
    KernelKill, KernelTerminate,
};

use crate::plugins;

/// A kernel provided by a plugin
#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
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

    /// Does the kernel support forks?
    #[serde(default)]
    forks: KernelForks,
}

impl Kernel for PluginKernel {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn availability(&self) -> KernelAvailability {
        // Assume that the kernel is available if the plugin if available
        KernelAvailability::Available
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

    fn supports_forks(&self) -> KernelForks {
        self.forks
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        todo!()
    }
}

/// List all the kernels provided by plugins
pub async fn list() -> Vec<Box<dyn Kernel>> {
    plugins()
        .await
        .into_iter()
        .flat_map(|plugin| plugin.kernels)
        .map(|kernel| Box::new(kernel) as Box<dyn Kernel>)
        .collect()
}
