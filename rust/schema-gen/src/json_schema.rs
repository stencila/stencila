//! Generation of a JSON Schema from Stencila Schema

use common::eyre::Result;

use crate::schemas::Schemas;

impl Schemas {
    /// Generate JSON Schema for the schemas
    pub async fn json_schema(&self) -> Result<()> {
        eprintln!("Generating JSON Schema: not yet implemented");

        // TODO

        Ok(())
    }
}
