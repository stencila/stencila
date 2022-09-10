use std::path::Path;

use kernel::{
    common::{async_trait::async_trait, eyre::Result, serde::Serialize, tokio::sync::mpsc},
    formats::Format,
    graph_triples::ResourceChange,
    stencila_schema::Node,
    Kernel, KernelSelector, KernelStatus, KernelTrait, TagMap, Task, TaskResult,
};
use kernel_sql::SqlKernel;

/// A kernel that executes SQL
#[derive(Debug, Clone, Serialize)]
#[serde(transparent, crate = "kernel::common::serde")]
pub struct PrqlKernel(SqlKernel);

impl PrqlKernel {
    /// Create a new PrQL kernel
    pub fn new(
        selector: &KernelSelector,
        resource_changes_sender: Option<mpsc::Sender<ResourceChange>>,
    ) -> Self {
        Self(SqlKernel::new(selector, resource_changes_sender))
    }
}

#[async_trait]
impl KernelTrait for PrqlKernel {
    async fn spec(&self) -> Kernel {
        let mut spec = self.0.spec().await;
        spec.name = "prql".to_string();
        spec.languages = vec![Format::PrQL];
        spec
    }

    async fn status(&self) -> Result<KernelStatus> {
        self.0.status().await
    }

    async fn start(&mut self, directory: &Path) -> Result<()> {
        self.0.start(directory).await
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        self.0.get(name).await
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        self.0.set(name, value).await
    }

    async fn exec_sync(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task> {
        match prql_compiler::compile(code) {
            Ok(sql) => self.0.exec_sync(&sql, lang, tags).await,
            Err(error) => {
                let mut task = Task::begin_sync();
                task.end(TaskResult::syntax_error(&error.to_string()));
                Ok(task)
            }
        }
    }

    async fn exec_async(
        &mut self,
        code: &str,
        lang: Format,
        tags: Option<&TagMap>,
    ) -> Result<Task> {
        match prql_compiler::compile(code) {
            Ok(sql) => self.0.exec_async(&sql, lang, tags).await,
            Err(error) => {
                let mut task = Task::begin_sync();
                task.end(TaskResult::syntax_error(&error.to_string()));
                Ok(task)
            }
        }
    }

    async fn exec_fork(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task> {
        match prql_compiler::compile(code) {
            Ok(sql) => self.0.exec_fork(&sql, lang, tags).await,
            Err(error) => {
                let mut task = Task::begin_sync();
                task.end(TaskResult::syntax_error(&error.to_string()));
                Ok(task)
            }
        }
    }
}
