use crate::prelude::*;

mod package;
mod variable;

use package::Package;
use variable::Variable;

#[cfg(test)]
mod tests;

/// The kernels associated with a document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Kernels {
    items: Vec<Kernel>,
}

impl Kernels {
    #[cfg(test)]
    pub fn new(items: Vec<Kernel>) -> Self {
        Self { items }
    }
}

#[rquickjs::methods]
impl Kernels {
    /// Get the count of all kernels
    #[qjs(get)]
    fn count(&self) -> usize {
        self.items.len()
    }

    /// Get all kernels
    #[qjs(get)]
    fn all(&self) -> Vec<Kernel> {
        self.items.clone()
    }

    /// Get the first kernel (if any)
    #[qjs(get)]
    fn first(&self) -> Option<Kernel> {
        self.items.first().cloned()
    }

    /// Get the last kernel (if any)
    #[qjs(get)]
    fn last(&self) -> Option<Kernel> {
        self.items.last().cloned()
    }

    /// Get the first kernel with a matching name
    ///
    /// Note that the name argument is matched case-insensitively and partially.
    #[qjs()]
    fn find(&self, name: String) -> Option<Kernel> {
        let name = name.to_lowercase();
        self.items
            .iter()
            .find(|kernel| kernel.name.to_lowercase().contains(&name))
            .cloned()
    }
}

impl Kernels {
    /// Create a set of kernel contexts from a set of kernel instances
    pub async fn from_kernels(kernels: &kernels::Kernels) -> Result<Self> {
        let mut items = Vec::new();
        for kernel in kernels.instances().await {
            let mut kernel_instance = kernel.lock().await;
            let kernel_context = Kernel::from_kernel_instance(kernel_instance.as_mut()).await?;
            items.push(kernel_context);
        }
        Ok(Self { items })
    }
}

/// A kernel associated with a document
///
/// This encapsulates the information that can be obtained from
/// a `KernelInstance` at runtime.
///
/// Note that `info` and `packages` probably only need to be
/// obtained from a kernel instance once, whereas `variables`
/// needs to be updated whenever a variable is declared or
/// updated in a kernel.
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Kernel {
    /// The name of the kernel e.g. Python or R
    #[qjs(get, enumerable)]
    name: String,

    /// The version of the kernel runtime e.g. 1.2.3
    #[qjs(get, enumerable)]
    version: Option<String>,

    /// The operating system of the kernel runtime
    #[qjs(get, enumerable)]
    os: Option<String>,

    /// A list of packages available in the kernel instance
    #[qjs(get, enumerable)]
    packages: Vec<Package>,

    // A list of variables available in the kernel instance
    #[qjs(get, enumerable)]
    variables: Vec<Variable>,
}

impl Kernel {
    #[cfg(test)]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.into(),
            version: Some(version.into()),
            os: None,
            ..Default::default()
        }
    }

    /// Create a new context [`Kernel`] from a [`KernelInstance`]
    pub async fn from_kernel_instance(instance: &mut dyn KernelInstance) -> Result<Self> {
        let app = instance.info().await?;
        let name = app.name.clone();
        let version = app.options.software_version.clone();
        let os = app.options.operating_system.clone();

        let packages = instance
            .packages()
            .await?
            .into_iter()
            .map(Package::from)
            .collect();

        let variables = instance
            .list()
            .await?
            .into_iter()
            .map(Variable::from)
            .collect();

        Ok(Self {
            name,
            version,
            os,
            packages,
            variables,
        })
    }
}

#[rquickjs::methods]
impl Kernel {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    pub fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;

        obj.set("name", self.name.clone())?;
        obj.set("version", self.version.clone())?;
        obj.set("os", self.os.clone())?;
        obj.set("packages", self.packages.clone())?;
        obj.set("variables", self.variables.clone())?;

        Ok(obj)
    }
}
