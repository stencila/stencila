use common::{async_trait::async_trait};


use stencila_schema::SoftwareSourceCode;

use crate::executable::{Executable};

/// Compile a `SoftwareSourceCode` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
#[async_trait]
impl Executable for SoftwareSourceCode {
    #[cfg(ignore)]
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "sc", context);

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
