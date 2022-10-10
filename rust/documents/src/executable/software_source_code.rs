use common::{async_trait::async_trait, eyre::Result};
use graph_triples::{
    resources::{self},
};

use node_address::Address;

use stencila_schema::{
    SoftwareSourceCode,
};

use crate::{
    assert_id,
    executable::{AssembleContext, CompileContext, Executable},
    register_id,
};

/// Compile a `SoftwareSourceCode` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
#[async_trait]
impl Executable for SoftwareSourceCode {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        register_id!("sc", self, address, context);
        Ok(())
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        if let (Some(code), Some(language)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let resource = resources::code(
                &context.path,
                id,
                "SoftwareSourceCode",
                formats::match_name(language),
            );
            let resource_info = parsers::parse(resource, code)?;
            context.resource_infos.push(resource_info);
        }

        Ok(())
    }
}
