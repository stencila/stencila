use common::{async_trait::async_trait, eyre::Result};
use graph_triples::{
    resources::{self},
    Relation, ResourceInfo,
};

use node_address::Address;
use path_utils::merge;
use stencila_schema::{
    Link,
};

use crate::{
    assert_id,
    executable::{AssembleContext, CompileContext, Executable},
    register_id,
};

#[async_trait]
impl Executable for Link {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        register_id!("li", self, address, context);
        Ok(())
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;
        let resource = resources::node(&context.path, id, "Link");

        let target = &self.target;
        let object = if target.starts_with("http://") || target.starts_with("https://") {
            resources::url(target)
        } else {
            resources::file(&merge(&context.path, target))
        };
        let relations = vec![(Relation::Links, object)];

        let resource_info =
            ResourceInfo::new(resource, Some(relations), None, None, None, None, None);
        context.resource_infos.push(resource_info);

        Ok(())
    }
}
