//! Implementations of `Executable` for generic types

use common::{async_trait::async_trait, eyre::Result};

use crate::executable::{CompileContext, Executable};

use super::ExecuteContext;

#[async_trait]
impl<T> Executable for Option<T>
where
    T: Executable + Send + Sync,
{
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        match self {
            Some(value) => value.compile(context).await,
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
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        (**self).compile(context).await
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
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        for (_index, item) in self.iter_mut().enumerate() {
            item.compile(context).await?;
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
