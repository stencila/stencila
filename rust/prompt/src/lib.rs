use kernel_quickjs::{
    kernel::{common::eyre::Result, KernelInstance},
    QuickJsKernelInstance,
};
use rquickjs::Class;

mod document;
mod instruction;
mod kernels;
mod prelude;

/// The execution context for a prompt
#[derive(Default)]
pub struct Context {
    /// The current instruction
    instruction: instruction::Instruction,

    /// The current document
    document: document::Document,

    /// The execution kernels associated with the document
    kernels: kernels::Kernels,
}

impl Context {
    /// Create a QuickJS kernel instance for the context
    pub async fn into_kernel(self) -> Result<Box<dyn KernelInstance>> {
        let mut instance = QuickJsKernelInstance::new("prompt".to_string());
        instance.start_here().await?;
        instance
            .runtime_context()?
            .with(|ctx| {
                let Context {
                    instruction,
                    document,
                    kernels,
                } = self;
                let document = Class::instance(ctx.clone(), document)?;
                let globals = ctx.globals();
                globals.set("instruction", instruction)?;
                globals.set("document", document)?;
                globals.set("kernels", kernels)
            })
            .await?;

        Ok(Box::new(instance))
    }
}
