use super::prelude::*;

#[async_trait]
impl<T> Executable for Option<T>
where
    T: Executable + Send + Sync,
{
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        match self {
            Some(value) => value.compile(address, context).await,
            None => Ok(()),
        }
    }

    async fn execute(&mut self, context: &mut ExecuteContext) -> Result<()> {
        match self {
            Some(value) => value.execute(context).await,
            None => Ok(()),
        }
    }
}

#[async_trait]
impl<T> Executable for Box<T>
where
    T: Executable + Send + Sync,
{
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        (**self).compile(address, context).await
    }

    async fn execute(&mut self, context: &mut ExecuteContext) -> Result<()> {
        (**self).execute(context).await
    }
}

#[async_trait]
impl<T> Executable for Vec<T>
where
    T: Executable + Send + Sync,
{
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        for (index, item) in self.iter().enumerate() {
            address.push_back(Slot::Index(index));
            item.compile(address, context).await?;
            address.pop_back();
        }
        Ok(())
    }

    async fn execute(&mut self, context: &mut ExecuteContext) -> Result<()> {
        for item in self {
            item.execute(context).await?;
        }
        Ok(())
    }
}
