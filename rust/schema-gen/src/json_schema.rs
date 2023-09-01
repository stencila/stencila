//! Generation of a JSON Schema from Stencila Schema

use common::eyre::Result;

use crate::{schema::Schema, schemas::Schemas};

impl Schemas {
    /// Generate JSON Schema for the schemas
    pub async fn json_schema(&self) -> Result<()> {
        // Generate the meta schema
        Schema::meta_schema().await?;

        Ok(())
    }
}
