use kernel_quickjs::{
    kernel::{common::eyre::Result, KernelInstance},
    QuickJsKernelInstance,
};
use rquickjs::Error;

mod document;
mod instruction;
mod kernels;
mod prelude;

// Export parts of context with renaming to avoid ambiguities
pub use document::Document as DocumentContext;
pub use instruction::Instruction as InstructionContext;
pub use kernels::Kernels as KernelsContext;

/// The execution context for a prompt
///
/// Note that all parts of the context are optional. This is for performance
/// reasons so that context is only generated or cloned when it is needed
#[derive(Default)]
pub struct PromptContext {
    /// The current instruction
    pub instruction: Option<instruction::Instruction>,

    /// The current document
    pub document: Option<document::Document>,

    /// The execution kernels associated with the document
    pub kernels: Option<kernels::Kernels>,
}

impl PromptContext {
    /// Create a QuickJS kernel instance for the context
    pub async fn into_kernel(self) -> Result<Box<dyn KernelInstance>> {
        let mut instance = QuickJsKernelInstance::new();
        instance.start_here().await?;
        instance
            .runtime_context()?
            .with(|ctx| {
                let globals = ctx.globals();
                if let Some(instruction) = self.instruction {
                    globals.set("instruction", instruction)?;
                }
                if let Some(document) = self.document {
                    globals.set("document", document)?;
                }
                if let Some(kernels) = self.kernels {
                    globals.set("kernels", kernels)?;
                }
                Ok::<(), Error>(())
            })
            .await?;

        Ok(Box::new(instance))
    }
}
