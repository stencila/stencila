//! Generation of a JSON-LD context from Stencila Schema

use common::eyre::Result;

use crate::schemas::Schemas;

impl Schemas {
    /// Generate JSON-LD context for the schemas
    pub async fn json_ld(&self) -> Result<()> {
        eprintln!("Generating JSON-LD: not yet implemented");

        // TODO

        Ok(())
    }
}
