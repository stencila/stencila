use std::fs::remove_file;
use std::path::PathBuf;
use std::{env, path::Path, process::Stdio, sync::Arc};

use binary_postgrest::BinaryTrait;
use binary_postgrest::PostgrestBinary;
use kernel::common::serde::Deserialize;
use kernel::stencila_schema::CodeError;
use kernel::TaskResult;
use kernel::{
    common::{
        async_trait::async_trait,
        dirs,
        eyre::{bail, Result, WrapErr},
        serde::Serialize,
        serde_json,
        tokio::{
            self,
            fs::{create_dir_all, write},
            io::{AsyncBufReadExt, BufReader},
            sync::RwLock,
        },
        tracing,
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

    /// Custom error handler to improve usability of errors displayed to users
    ///
    /// See https://postgrest.org/en/stable/errors.html
    pub fn error_handler(error_type: &str, error_body: &str) -> CodeError {
        #[derive(Deserialize)]
        #[serde(crate = "kernel::common::serde")]
        struct Error {
            code: String,
            details: Option<String>,
            hint: Option<String>,
            message: Option<String>,
        }

        // Attempt to parse body as JSON (may not be JSON if this was for example a HTTP connection error)
        let error = match serde_json::from_str::<Error>(error_body) {
            Ok(error) => error,
            Err(..) => {
                return CodeError {
                    error_type: Some(Box::new(error_type.to_string())),
                    error_message: error_body.to_string(),
                    ..Default::default()
                }
            }
        };

        // Map the error code to the error type (the `error_type` that this function receives
        // is the HTTP status code number and string, which we can ignore if we got JSON).
        let (error_type, message) = match error.code.as_str() {
            "PGRST000" => ("Connection error", "Could not connect with the database due to an incorrect db-uri or the PostgreSQL service not running."),
            "PGRST001" => ("Connection error", "Could not connect with the database due to an internal error."),
            "PGRST002" => ("Connection error", "Could not connect with the database when building the Schema Cache due to the PostgreSQL service not running."),
            "PGRST100" => ("Request error", "Parsing error in the query string."),
            "PGRST101" => ("Request error", "Only GET and POST verbs are allowed"),
            "PGRST102" => ("Request error", "An invalid request body was sent(e.g. an empty body or malformed JSON)."),
            "PGRST103" => ("Request error", "An invalid range was specified for Limits and Pagination."),
            "PGRST104" => ("Request error", "Either the filter operator is missing or it doesn’t exist."),
            "PGRST105" => ("Request error", "An invalid PUT request was done"),
            "PGRST106" => ("Request error", "The schema specified when switching schemas is not present in the db-schemas configuration variable."),
            "PGRST107" => ("Request error", "The Content-Type sent in the request is invalid."),
            "PGRST108" => ("Request error", "The filter is applied to a embedded resource that is not specified in the select part of the query string. See Embedded Filters."),
            "PGRST109" => ("Request error", "Restricting a Deletion or an Update using limits must include the ordering of a unique column. See Limited Updates/Deletions."),
            "PGRST110" => ("Request error", "When restricting a Deletion or an Update using limits modifies more rows than the maximum specified in the limit. See Limited Updates/Deletions."),
            "PGRST111" => ("Request error", "An invalid response.headers was set. See Setting Response Headers."),
            "PGRST112" => ("Request error", "The status code must be a positive integer. See Setting Response Status Code."),
            "PGRST113" => ("Request error", "More than one column was returned for a scalar result. See Response Formats For Scalar Responses."),
            "PGRST114" => ("Request error", "For an UPSERT using PUT, when limits and offsets are used."),
            "PGRST115" => ("Request error", "For an UPSERT using PUT, when the primary key in the query string and the body are different."),
            "PGRST116" => ("Request error", "More than 1 or no items where returned when requesting a singular response. See Singular or Plural."),
            "PGRST117" => ("Request error", "The HTTP verb used in the request in not supported."),
            "PGRST200" => ("Schema cache error", "Caused by Stale Foreign Key Relationships, otherwise any of the embedding resources or the relationship itself may not exist in the database."),
            "PGRST201" => ("Schema cache error", "An ambiguous embedding request was made. See Embedding Disambiguation."),
            "PGRST202" => ("Schema cache error", "Caused by a Stale Function Signature, otherwise the function may not exist in the database."),
            "PGRST203" => ("Schema cache error", "Caused by requesting overloaded functions with the same argument names but different types, or by using a POST verb to request overloaded functions with a JSON or JSONB type unnamed parameter. The solution is to rename the function or add/modify the names of the arguments."),
            "PGRST300" => ("Authentication error", "A JWT secret is missing from the configuration."),
            "PGRST301" => ("Authentication error", "Any error related to the verification of the JWT, which means that the JWT provided is invalid in some way."),
            "PGRST302" => ("Authentication error", "Attempted to do a request without authentication when the anonymous role is disabled by not setting it in db-anon-role."),
            _ => ("Error", "Unknown error")
        };

        let mut error_message = error.message.unwrap_or_else(|| message.to_string());
        if let Some(hint) = error.hint {
            error_message.push_str("\n\n");
            error_message.push_str(&hint);
        }
        if let Some(details) = error.details {
            error_message.push_str("\n\n");
            error_message.push_str(&details);
        }

        CodeError {
            error_type: Some(Box::new(error_type.to_string())),
            error_message: error_message.to_string(),
            ..Default::default()
        }
    }
}

impl Default for PostgrestKernel {
    fn default() -> Self {
        Self {
            id: uuids::generate("pr").to_string(),
            config_file: None,
            status: Arc::new(RwLock::new(KernelStatus::Pending)),
            http_kernel: HttpKernel::with_error_handler(Box::new(PostgrestKernel::error_handler)),
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
    async fn start(&mut self, directory: &Path) -> Result<()> {
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
            db-uri = "postgres://authenticator:mysecretpassword@localhost:5433/postgres"
            db-schemas = "api"
            db-anon-role = "web_anon"
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

        self.http_kernel.start(directory).await
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
                return Ok(Task::begin_and_end(Some(TaskResult::syntax_error(
                    &error.to_string(),
                ))))
            }
        };

        // Insert the PostgREST server as the host header
        let host = "\nHost: http://127.0.0.1:3000\n";
        let http = match http.contains('\n') {
            true => http.replacen('\n', host, 1),
            false => [&http, host].concat(),
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
