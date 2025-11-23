//! Generation of a JSON Schema from Stencila Schema

use std::path::PathBuf;

use eyre::{ErrReport, Result};
use futures::future::try_join_all;
use glob::glob;
use schemars::generate::SchemaSettings;
use tokio::{
    fs::{File, remove_file},
    io::AsyncWriteExt,
};

use crate::{schema::Schema, schemas::Schemas};

impl Schemas {
    /// Generate a JSON Schema meta-schema and a JSON Schema for each schema
    #[allow(clippy::print_stderr)]
    pub async fn json_schema(&self) -> Result<()> {
        eprintln!("Generating JSON Schema");

        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../json/");

        // Remove all existing *.schema.json files
        let futures = glob(&dir.join("*.schema.json").to_string_lossy())?
            .flatten()
            .filter(|file| file != &PathBuf::from("stencila-config.schema.json"))
            .map(|file| async { remove_file(file).await });
        try_join_all(futures).await?;

        // Generate the meta schema
        let path = dir.join("meta.schema.json");
        let mut file = File::create(path).await?;

        let settings = SchemaSettings::draft07();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<Schema>();

        let json = serde_json::to_string_pretty(&schema)?;
        file.write_all(json.as_bytes()).await?;

        // Generate a schema for each schema
        let futures = self.schemas.iter().map(|(title, schema)| {
            let dir = dir.clone();
            async move {
                let path = dir.join(format!("{title}.schema.json"));
                let mut file = File::create(path).await?;

                let json = serde_json::to_string_pretty(schema)?;
                file.write_all(json.as_bytes()).await?;

                Ok::<(), ErrReport>(())
            }
        });
        try_join_all(futures).await?;

        Ok(())
    }
}
