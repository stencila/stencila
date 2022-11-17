use common::{async_trait::async_trait, eyre::Result};
use graph_triples::{
    resources::{self},
    Relation, ResourceInfo,
};

use path_utils::merge;
use stencila_schema::Link;

use crate::executable::{CompileContext, Executable};

#[async_trait]
impl Executable for Link {
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "li", context);

        let resource = resources::node(&context.path, id, "Link");

        let target = &self.target;
        let object = if target.starts_with("http://") || target.starts_with("https://") {
            resources::url(target)
        } else {
            resources::file(&merge(&context.path, target))
        };
        let relations = vec![(Relation::Links, object)];

        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        context.resource_infos.push(resource_info);

        Ok(())
    }
}
