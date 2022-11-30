use super::prelude::*;

#[async_trait]
impl Executable for CodeBlock {
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        if self.id.is_none() {
            context.push_patch(produce_address(self, address, |draft| {
                draft.id = generate_id("cb");
            }))
        }

        Ok(())
    }
}

#[async_trait]
impl Executable for CodeFragment {
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        if self.id.is_none() {
            context.push_patch(produce_address(self, address, |draft| {
                draft.id = generate_id("cf");
            }))
        }

        Ok(())
    }
}
