//! Implementations of `Executable` for generic types

use common::{async_trait::async_trait, eyre::Result};
use node_address::{Address, Slot};

use crate::executable::{AssembleContext, Executable};

use super::ExecuteContext;

#[async_trait]
impl<T> Executable for Option<T>
where
    T: Executable + Send + Sync,
{
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        match self {
            Some(value) => value.assemble(address, context).await,
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
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        (**self).assemble(address, context).await
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
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            address.push_back(Slot::Index(index));
            item.assemble(address, context).await?;
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
