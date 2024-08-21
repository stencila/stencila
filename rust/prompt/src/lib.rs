use kernel_quickjs::{
    kernel::{common::eyre::Result, KernelInstance},
    QuickJsKernelInstance,
};
use rquickjs::Class;

mod document;
mod kernels;
mod prelude;

/// The execution context for a prompt
#[derive(Default)]
pub struct Context {
    /// The current document
    pub document: document::Document,

    /// The execution kernels associated with the document
    pub kernels: kernels::Kernels,
}

impl Context {
    /// Create a QuickJS kernel for the context
    pub async fn into_kernel(self) -> Result<Box<dyn KernelInstance>> {
        let mut instance = QuickJsKernelInstance::new("prompt".to_string());
        instance.start_here().await?;
        instance
            .runtime_context()?
            .with(|ctx| {
                let Context { document, kernels } = self;
                let document = Class::instance(ctx.clone(), document)?;
                ctx.globals().set("document", document)?;
                ctx.globals().set("kernels", kernels)
            })
            .await?;

        Ok(Box::new(instance))
    }
}
