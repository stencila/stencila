use crate::utils::uuids;
use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut};
use enum_dispatch::enum_dispatch;
use eyre::{eyre, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::{hash_map::Entry, BTreeMap, HashMap};
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

#[derive(Debug, Clone, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct VariableInfo {
    /// The home kernel of the variable
    home: KernelId,

    /// The time that the variable was last set in the home kernel
    set_last: DateTime<Utc>,

    /// The time that the variable was last mirrored to other kernels
    mirrors: HashMap<KernelId, DateTime<Utc>>,
}

impl VariableInfo {
    pub fn new(kernel_id: &KernelId) -> Self {
        VariableInfo {
            home: kernel_id.into(),
            set_last: Utc::now(),
            mirrors: HashMap::new(),
        }
    }
}

/// A map of [`KernelId`] to [`Kernel`]
///
/// A `newtype` that exists solely to provide a `Result` (rather than `<Option>`
/// when getting a kernel by id.
#[derive(Debug, Clone, Default, Deref, DerefMut, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
struct KernelMap(BTreeMap<KernelId, Kernel>);

impl KernelMap {
    /// Get a reference to a kernel
    fn get(&self, kernel_id: &KernelId) -> Result<&Kernel> {
        (**self)
            .get(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }

    /// Get a mutable reference to a kernel
    fn get_mut(&mut self, kernel_id: &KernelId) -> Result<&mut Kernel> {
        (**self)
            .get_mut(kernel_id)
            .ok_or_else(|| eyre!("Unknown kernel `{}`", kernel_id))
    }
}

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct KernelSpace {
    /// The kernels in the document kernel space
    kernels: KernelMap,

    /// The variables in the document kernel space
    variables: HashMap<String, VariableInfo>,
}

impl KernelSpace {
    /// Get a variable from the kernel space
    pub fn get(&self, name: &str) -> Result<Node> {
        let variable_info = self
            .variables
            .get(name)
            .ok_or_else(|| eyre!("Unknown variable `{}`", name))?;

        let kernel = self.kernels.get(&variable_info.home)?;
        kernel.get(name)
    }

    /// Set a variable in the kernel space
    pub fn set(&mut self, name: &str, value: Node, language: &str) -> Result<()> {
        tracing::debug!("Setting variable `{}`", name);

        let kernel_id = self.ensure_kernel(language)?;
        let kernel = self.kernels.get_mut(&kernel_id)?;
        kernel.set(name, value)?;

        match self.variables.entry(name.to_string()) {
            Entry::Occupied(mut occupied) => {
                let info = occupied.get_mut();
                info.set_last = Utc::now();
            }
            Entry::Vacant(vacant) => {
                vacant.insert(VariableInfo::new(&kernel_id));
            }
        }

        Ok(())
    }

    /// Execute some code in the kernel space
    ///
    /// Variables that the code uses, but have a different home kernel, are mirrored to the kernel.
    pub fn exec(&mut self, code: &str, language: &str) -> Result<Vec<Node>> {
        tracing::debug!("Executing code");

        let kernel_id = self.ensure_kernel(language)?;

        // TODO: Pass the list of used variables to this function
        let uses = self.variables.clone();
        for name in uses.keys() {
            let variable_info = self
                .variables
                .get_mut(name)
                .ok_or_else(|| eyre!("Unknown variable `{}`", name))?;

            if variable_info.home != *kernel_id {
                let home_kernel = self.kernels.get(&variable_info.home)?;
                let value = home_kernel.get(name)?;

                let kernel = self.kernels.get_mut(&kernel_id)?;
                kernel.set(name, value)?;

                match variable_info.mirrors.entry(kernel_id.clone()) {
                    Entry::Occupied(mut occupied) => {
                        let datetime = occupied.get_mut();
                        *datetime = Utc::now();
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(Utc::now());
                    }
                }
            }
        }

        let kernel = self.kernels.get_mut(&kernel_id)?;
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
}
