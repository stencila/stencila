//! Generation of reference document from Stencila Schema

use common::eyre::Result;

use crate::schemas::Schemas;

impl Schemas {
    /// Generate reference documentation for the schemas
    pub async fn docs(&self) -> Result<()> {
        eprintln!("Generating documentation: not yet implemented");

        // TODO

        Ok(())
    }
}
