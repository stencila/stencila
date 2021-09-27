use crate::utils::uuids;
use enum_dispatch::enum_dispatch;
use eyre::{eyre, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use stencila_schema::Node;

type KernelId = String;

#[enum_dispatch]
pub trait KernelTrait {
    /// Get the name of the kernel's programming language, and/or
    /// check that it is able to execute a given language.
    ///
    /// If a `language` identifier is supplied, e.g. `Some("py")`, and the kernel
    /// can execute that language, should return the canonical name of the language
    /// e.g. `Ok("python3")`. If the language can not execute the language should
    /// return a `IncompatibleLanguage` error.
    fn language(&self, language: Option<String>) -> Result<String>;

    /// Get a variable from the kernel
    fn get(&self, name: &str) -> Result<Node>;

    /// Set a variable in the kernel
    fn set(&mut self, name: &str, value: Node) -> Result<()>;

    /// Execute some code in the kernel
    fn exec(&mut self, code: &str) -> Result<Vec<Node>>;
}

mod default;
use default::*;

mod calc;
use calc::*;

#[enum_dispatch(KernelTrait)]
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(tag = "type")]
pub enum Kernel {
    Default(DefaultKernel),
    Calc(CalcKernel),
}

type VariableName = String;

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct KernelSpace {
    /// The kernels in the document kernel space
    kernels: BTreeMap<KernelId, Kernel>,

    /// The variables in the document kernel space
    variables: HashMap<VariableName, KernelId>,
}

impl KernelSpace {
    /// Get a variable from the kernel space
    pub fn get(&self, name: &str) -> Result<Node> {
        let kernel_id = self
            .variables
            .get(name)
            .ok_or_else(|| eyre!("Unknown variable `{}`", name))?;

        let kernel = self.get_kernel(kernel_id)?;
        kernel.get(name)
    }

    /// Set a variable in the kernel space
    pub fn set(&mut self, name: &str, value: Node, language: &str) -> Result<()> {
        tracing::debug!("Setting variable `{}`", name);

        let kernel_id = self.ensure_kernel(language)?;
        let kernel = self.get_kernel_mut(&kernel_id)?;
        kernel.set(name, value)?;

        self.variables.insert(name.to_string(), kernel_id);

        Ok(())
    }

    /// Execute some code in the kernel space
    pub fn exec(&mut self, code: &str, language: &str) -> Result<Vec<Node>> {
        tracing::debug!("Executing code");

        let kernel_id = self.ensure_kernel(language)?;

        let kernel = self.get_kernel_mut(&kernel_id)?;
        kernel.exec(code)
    }

    /// Ensure that a kernel exists for a language
    ///
    /// Returns a tuple of the kernel's canonical language name and id.
    fn ensure_kernel(&mut self, language: &str) -> Result<KernelId> {
        // Is there already a kernel capable of executing the language?
        for (kernel_id, kernel) in self.kernels.iter_mut() {
            if kernel.language(Some(language.to_string())).is_ok() {
                return Ok(kernel_id.clone());
            }
        }

        // If unable to set in an existing kernel then start a new kernel
        // for the language.
        let kernel = match language {
            "calc" => Kernel::Calc(CalcKernel::new()),
            _ => Kernel::Default(DefaultKernel::new()),
        };
        let kernel_id = uuids::generate(uuids::Family::Kernel);
        self.kernels.insert(kernel_id.clone(), kernel);

        Ok(kernel_id)
    }

    /// Get a kernel using its id
    fn get_kernel(&self, kernel_id: &KernelId) -> Result<&Kernel> {
        self.kernels
            .get(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Get a mutable kernel using its id
    fn get_kernel_mut(&mut self, kernel_id: &KernelId) -> Result<&mut Kernel> {
        self.kernels
            .get_mut(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }
}
