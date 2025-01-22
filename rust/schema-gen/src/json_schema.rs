//! Generation of a JSON Schema from Stencila Schema

use std::path::PathBuf;

use common::{
    eyre::{ErrReport, Result},
    futures::future::try_join_all,
    glob::glob,
    serde_json,
    tokio::{
        fs::{remove_file, File},
        io::AsyncWriteExt,
    },
};
use schemars::gen::SchemaSettings;

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
            .map(|file| async { remove_file(file).await });
        try_join_all(futures).await?;

        // Generate the meta schema
        let path = dir.join("meta.schema.json");
        let mut file = File::create(path).await?;

        let settings = SchemaSettings::draft07().with(|s| {
            s.option_add_null_type = false;
        });
        let gen = settings.into_generator();
        let schema = gen.into_root_schema_for::<Schema>();

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
