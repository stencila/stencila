use std::fs::remove_file;
use std::path::PathBuf;
use std::{env, path::Path, process::Stdio, sync::Arc};

use binary_postgrest::BinaryTrait;
use binary_postgrest::PostgrestBinary;
use kernel::{
    common::{
        async_trait::async_trait,
        dirs,
        eyre::{bail, Result, WrapErr},
        serde::Serialize,
        tokio::{
            self,
            fs::{create_dir_all, write},
            io::{AsyncBufReadExt, BufReader},
            sync::RwLock,
        },
        tracing::{self},
    },
    formats::Format,
    stencila_schema::Node,
    uuids, Kernel, KernelStatus, KernelTrait, KernelType, TagMap, Task,
};
use kernel_http::HttpKernel;

/// A kernel that executes PostgREST statements
///
/// This kernel transpiles statements into PostgREST URLS (and
/// JSON bodies for insert, update etc statement) and sends them
/// over HTTP to a running `postgres` server
#[derive(Debug, Clone, Serialize)]
#[serde(crate = "kernel::common::serde")]
pub struct PostgrestKernel {
    /// The id of the kernel
    ///
    /// Used to ensure that PostgREST config files are unique to instances
    /// of `PostgrestKernel`
    id: String,

    /// The path to the PostgREST configuration file for this kernelcd ..
    config_file: Option<PathBuf>,

    /// The status of the kernel
    ///
    /// Reflects the status of the spawned `postgrest` process
    #[serde(skip)]
    status: Arc<RwLock<KernelStatus>>,

    /// The HTTP kernel that this kernel delegates to
    http_kernel: HttpKernel,
}

impl PostgrestKernel {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for PostgrestKernel {
    fn default() -> Self {
        Self {
            id: uuids::generate("pr").to_string(),
            config_file: None,
            status: Arc::new(RwLock::new(KernelStatus::Pending)),
            http_kernel: HttpKernel::default(),
        }
    }
}

impl Drop for PostgrestKernel {
    fn drop(&mut self) {
        if let Some(config_file) = &self.config_file {
            if config_file.exists() {
                remove_file(config_file).ok();
            }
        }
    }
}

#[async_trait]
impl KernelTrait for PostgrestKernel {
    async fn spec(&self) -> Kernel {
        Kernel::new(
            "postgrest",
            KernelType::Builtin,
            &[Format::Postgrest],
            true,
            true,
            true,
        )
    }

    /// Get the status of the kernel
    async fn status(&self) -> Result<KernelStatus> {
        return Ok(self.status.read().await.clone());
    }

    /// Start the kernel
    ///
    /// Spawns `postgrest` in a background task. PostgREST prints logs to stderr, including errors.
    /// We capture these logs, detect which are errors, add them to Stencila's own tracing logs as
    /// set the kernel status to `Failed`. We also detect the 'Listening on port...' message and
    /// set kernel status to `Started` in that case.
    async fn start(&mut self, _directory: &Path) -> Result<()> {
        *self.status.write().await = KernelStatus::Starting;

        // Ensure PostgREST is installed
        let install = match (PostgrestBinary {}).ensure().await {
            Ok(install) => Ok(install),
            Err(error) => {
                *self.status.write().await = KernelStatus::Failed;
                Err(error).wrap_err("While attempting to ensure `postgres` installation")
            }
        }?;

        // Resolve, and create if necessary, the directory where the config file will be created
        let dir = dirs::data_dir()
            .unwrap_or_else(|| {
                env::current_dir().expect("Should always be able to get current dir")
            })
            .join("stencila")
            .join("kernels")
            .join("postgrest");
        create_dir_all(&dir).await?;

        // Write the config file for this kernel
        // TODO: Work out how best to setup/specify/pass-through etc these settings
        let config = r#"
            db-uri = "postgres://authenticator:authenticator@localhost:5432/postgres"
            db-schemas = "public"
            db-anon-role = "anon"
        "#;
        let config_file = dir.join([&self.id, ".config"].concat());
        write(&config_file, config).await?;

        // Spawn the process
        let mut child = match install
            .command()
            .args([config_file.to_string_lossy().to_string()])
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => Ok(child),
            Err(error) => {
                *self.status.write().await = KernelStatus::Failed;
                Err(error).wrap_err("While attempting to spawn `postgres` binary")
            }
        }?;

        // Record the path of the config file so it can be cleaned up later
        self.config_file = Some(config_file);

        // Monitor stderr for errors / success
        let status = self.status.clone();
        tokio::spawn(async move {
            let stderr = child.stderr.take().expect("stderr should be piped");
            let mut stderr_reader = BufReader::new(stderr).lines();

            while let Ok(Some(line)) = stderr_reader.next_line().await {
                if line.contains("Listening on port") {
                    *status.write().await = KernelStatus::Ready;
                    tracing::debug!("PostgREST has successfully started")
                } else if line.contains("Error")
                    || line.contains("error")
                    || line.contains("{\"code\":")
                {
                    *status.write().await = KernelStatus::Failed;
                    tracing::error!("[PostgREST] {}", line);
                } else {
                    tracing::trace!("[PostgREST] {}", line);
                }
            }
        });

        Ok(())
    }

    /// Get a symbol from the kernel
    ///
    /// Simply delegates to HTTP kernel.
    async fn get(&mut self, name: &str) -> Result<Node> {
        return self.http_kernel.get(name).await;
    }

    /// Set a symbol in the kernel
    ///
    /// Simply delegates to HTTP kernel.
    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        return self.http_kernel.set(name, value).await;
    }

    /// Derive one or more `Node`s from an object in the kernel
    ///
    /// Currently errors. In the future we may use PostgREST's OpenAPI endpoint to
    /// get meta data about tables etc and derive parameters or forms.
    async fn derive(&mut self, _what: &str, _from: &str) -> Result<Vec<Node>> {
        bail!("Method `derive()` is not supported by Postgrest kernel")
    }

    /// Execute code in the kernel asynchronously
    /// 
    /// Transpiles the PostqREST to a HTTP request. No variable interpolation is done here since
    /// that is delegated to `http_kernel`.
    async fn exec_async(
        &mut self,
        code: &str,
        lang: Format,
        tags: Option<&TagMap>,
    ) -> Result<Task> {
        if !matches!(lang, Format::Postgrest) {
            bail!("The `PostgrestKernel` can only execute PostgREST requests")
        }

        let http = match parser_postgrest::transpile(code) {
            Ok(code) => code,
            Err(error) => {
                tracing::error!("{}", error.to_string()); // TODO: turn this into an error for the task
                "".to_string()
            }
        };

        // Insert the PostgREST server as the host header
        let host = "\nHost: http://127.0.0.1:3000\n";
        let http = match http.contains('\n') {
            true => http.replacen('\n', host, 1),
            false => [&http, host].concat()
        };

        return self.http_kernel.exec_async(&http, Format::Http, tags).await;
    }

    /// Fork the kernel and execute code in the fork
    ///
    /// At present simply calls `exec_async` but it may be necessary to clone this kernel
    /// given that statefulness is possible with @assign tag.
    async fn exec_fork(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task> {
        self.exec_async(code, lang, tags).await
    }
}
