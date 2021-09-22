use eyre::Result;
use schemars::JsonSchema;
use serde::Serialize;
use stencila_schema::Node;

trait KernelTrait {
    fn get(&mut self, name: &str) -> Result<Node>;

    fn set(&mut self, name: &str, value: Node) -> Result<()>;

    fn exec(&mut self, code: &str) -> Result<Vec<Node>>;
}

mod calc;
mod default;

#[derive(Debug, Clone, JsonSchema, Serialize)]
pub enum Kernel {
    Default(default::DefaultKernel),
    Calc(calc::CalcKernel),
}
