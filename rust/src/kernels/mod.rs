use enum_dispatch::enum_dispatch;
use eyre::Result;
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use stencila_schema::Node;

type KernelId = String;

#[enum_dispatch]
pub trait KernelTrait {
    /// Get a variable from the kernel
    fn get(&mut self, name: &str) -> Result<Node>;

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
pub enum Kernel {
    Default(DefaultKernel),
    Calc(CalcKernel),
}

type VariableName = String;

#[derive(Debug, Clone, Default, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct KernelSpace {
    kernels: BTreeMap<KernelId, Kernel>,
    variables: HashMap<VariableName, KernelId>,
}

impl KernelSpace {
    /// Get a variable from the kernels
    pub fn get(&mut self, _name: &str) -> Result<Node> {
        todo!()
    }

    /// Set a variable in the kernel
    pub fn set(&mut self, name: &str, value: Node, _language: &str) -> Result<()> {
        let mut kernel_id: String = "".to_string();

        // Attempt to set in one of the existing kernels
        //for (id, kernel) in self.kernels.iter_mut() {
        //if kernel.language(language) {
        //    kernel.set(name, value.clone())?;
        //    kernel_id = id.clone();
        //    break;
        //}
        //}

        // If was unable to set in any kernel then start a kernel
        if kernel_id.is_empty() {
            let mut kernel = Kernel::Default(DefaultKernel::new());
            kernel.set(name, value)?;
            kernel_id = "dodhhd".to_string();
            self.kernels.insert(kernel_id.clone(), kernel);
        }

        self.variables.insert(name.to_string(), kernel_id);

        Ok(())
    }

    /// Execute some code in the kernel
    pub fn exec(&mut self, _code: &str, _language: &str) -> Result<Vec<Node>> {
        todo!()
    }
}
