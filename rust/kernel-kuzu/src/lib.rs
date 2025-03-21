use std::path::Path;

use kuzu::{Connection, Database, LogicalType, SystemConfig};

use kernel::{
    Kernel, KernelInstance, KernelType,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
        tracing,
    },
    format::Format,
    generate_id,
    schema::{
        ExecutionBounds, ExecutionMessage, Node, SoftwareApplication, SoftwareApplicationOptions,
        StringOrNumber,
    },
};

mod from_kuzu;
pub use from_kuzu::*;

mod to_kuzu;
pub use to_kuzu::*;

pub use kuzu;

const NAME: &str = "kuzu";

/// A kernel for querying Kuzu databases.
#[derive(Default)]
pub struct KuzuKernel;

impl Kernel for KuzuKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn r#type(&self) -> KernelType {
        KernelType::Database
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Cypher]
    }

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![
            ExecutionBounds::Main,
            // Fork is possible by creating a read-only connection
            // but Box is not possible because statements such as `COPY .. TO ..`
            // will write to the filesystem
            ExecutionBounds::Fork,
        ]
    }

    fn create_instance(&self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(KuzuKernelInstance::new()))
    }
}

#[derive(Default)]
pub struct KuzuKernelInstance {
    /// The unique id of the kernel instance
    id: String,

    /// The path to the database
    path: Option<String>,

    /// The database instance
    db: Option<Database>,
}

impl KuzuKernelInstance {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            id: generate_id(NAME),
            path: None,
            db: None,
        }
    }
}

#[async_trait]
impl KernelInstance for KuzuKernelInstance {
    fn id(&self) -> &str {
        &self.id
    }

    async fn start(&mut self, _directory: &Path) -> Result<()> {
        let config = SystemConfig::default();
        let db = Database::in_memory(config)?;

        self.db = Some(db);

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::trace!("Executing Kuzu statements");

        let Some(db) = &self.db else {
            bail!("Database has not been started");
        };
        let connection = Connection::new(&db)?;

        let result = match connection.query(code) {
            Ok(result) => result,
            Err(error) => {
                return Ok((Vec::new(), vec![execution_message_from_error(error)]));
            }
        };

        let output = if result
            .get_column_data_types()
            .iter()
            .all(|data_type| matches!(data_type, LogicalType::Node))
        {
            Node::ImageObject(cytoscape_from_query_result(result)?)
        } else {
            Node::Datatable(datatable_from_query_result(result)?)
        };

        Ok((vec![output], Vec::new()))
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Kuzu runtime info");

        Ok(SoftwareApplication {
            name: "Kuzu".to_string(),
            version: Some(StringOrNumber::String(kuzu::VERSION.into())),
            options: Box::new(SoftwareApplicationOptions {
                operating_system: Some(std::env::consts::OS.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }

    async fn replicate(&mut self, _bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(Self::new()))
    }
}
