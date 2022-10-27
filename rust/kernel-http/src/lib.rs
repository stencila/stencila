use std::{collections::HashMap, sync::Arc};

use http_utils::reqwest::header::CONTENT_TYPE;
use kernel::{
    common::{
        async_trait::async_trait,
        eyre::{bail, Result},
        itertools::Itertools,
        regex::Captures,
        serde::Serialize,
        serde_json,
        tokio::{
            self,
            sync::{broadcast, mpsc, RwLock},
        },
        tracing,
    },
    formats::Format,
    stencila_schema::{CodeError, Node, Null, Primitive},
    Kernel, KernelStatus, KernelTrait, KernelType, TagMap, Task, TaskResult,
};
use node_transform::Transform;
use parser::utils::VAR_INTERP_REGEX;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(crate = "kernel::common::serde")]
pub struct HttpKernel {
    #[serde(skip)]
    symbols: Arc<RwLock<HashMap<String, Node>>>,
}

impl HttpKernel {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KernelTrait for HttpKernel {
    async fn spec(&self) -> Kernel {
        Kernel::new(
            "http",
            KernelType::Builtin,
            &[Format::Http],
            true,
            true,
            true,
        )
    }

    async fn status(&self) -> Result<KernelStatus> {
        Ok(KernelStatus::Idle)
    }

    async fn get(&mut self, name: &str) -> Result<Node> {
        match self.symbols.read().await.get(name) {
            Some(node) => Ok(node.clone()),
            None => bail!("Symbol `{}` does not exist in this HTTP kernel", name),
        }
    }

    async fn set(&mut self, name: &str, value: Node) -> Result<()> {
        self.symbols.write().await.insert(name.to_string(), value);

        Ok(())
    }

    async fn derive(&mut self, _what: &str, _from: &str) -> Result<Vec<Node>> {
        bail!("Method `derive()` is not supported by HTTP kernel")
    }

    async fn exec_async(
        &mut self,
        code: &str,
        lang: Format,
        tags: Option<&TagMap>,
    ) -> Result<Task> {
        if !matches!(lang, Format::Http) {
            bail!("The `HttpKernel` can only execute HTTP requests")
        }

        if code.trim().is_empty() {
            return Ok(Task::begin_and_end(None));
        }

        // Split the request into lines for parsing, returning if there are none
        let mut lines = code.lines();

        // Ignore starting lines that are comments or are blank
        let mut request = String::new();
        for line in lines.by_ref() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            } else {
                request = line.to_string();
                break;
            }
        }

        // Setup channels and create async task
        let (result_sender, ..) = broadcast::channel(1);
        let (interrupt_sender, mut interrupt_receiver) = mpsc::channel(1);
        let mut task = Task::begin(Some(result_sender.clone()), Some(interrupt_sender));

        let mut messages = Vec::new();

        // Do variable interpolation
        // This is a re-implementation of `parser::perform_var_interps` to interpolate strings
        // as double quoted strings in the body of the request.
        let symbols = self.symbols.read().await;
        let mut var_interp = |code, as_json| {
            VAR_INTERP_REGEX
                .replace_all(code, |captures: &Captures| {
                    let symbol = captures
                        .get(1)
                        .or_else(|| captures.get(2))
                        .expect("Should always have one group")
                        .as_str();
                    match symbols.get(symbol) {
                        Some(node) => {
                            if as_json {
                                serde_json::to_string(node).unwrap_or_default()
                            } else {
                                match node {
                                    Node::String(string) => string.clone(),
                                    _ => serde_json::to_string(node).unwrap_or_default(),
                                }
                            }
                        }
                        None => {
                            messages.push(CodeError {
                                error_type: Some(Box::new("UnknownSymbol".to_string())),
                                error_message: format!("Symbol `{}` is not available", symbol),
                                ..Default::default()
                            });
                            captures[0].to_string()
                        }
                    }
                })
                .to_string()
        };

        enum Method {
            Get,
            Head,
            Post,
            Put,
            Patch,
            Delete,
        }
        use Method::*;

        // Split the request line into three parts: method, URL, protocol version
        // Method defaults to GET, and protocol version to HTTP/1.1 (effectively, its ignored)
        let request = var_interp(&request, false);
        let parts = request.split_whitespace().collect_vec();
        let (method, url) = if parts.is_empty() {
            return Ok(Task::begin_and_end(None));
        } else if parts.len() == 1 {
            (Get, parts[0])
        } else {
            let method = match parts[0].to_lowercase().as_str() {
                "get" => Get,
                "head" => Head,
                "post" => Post,
                "put" => Put,
                "patch" => Patch,
                "delete" => Delete,
                _ => {
                    task.end(TaskResult::syntax_error(&format!(
                        "HTTP method unknown or not handled: {}",
                        parts[0]
                    )));
                    return Ok(task);
                }
            };
            (method, parts[1])
        };

        // Remaining lines before blank line are headers
        let mut headers: HashMap<String, String> = HashMap::new();
        for line in lines.by_ref() {
            if line.starts_with('#') {
                continue;
            }
            if line.trim().is_empty() {
                break;
            }
            let line = var_interp(line, false);
            if let Some((key, value)) = line.splitn(2, ':').collect_tuple() {
                let key = key.trim().to_lowercase();
                let value = value.trim().to_string();
                headers.insert(key, value);
            }
        }

        // Remaining lines after first blank line is body of request
        let body = lines.join("\n");
        let body = var_interp(&body, true);

        // Drop symbols guard now that interp has been done
        drop(symbols);

        // Return now if any errors related to interpolation
        if !messages.is_empty() {
            return Ok(Task::begin_and_end(Some(TaskResult::new(
                Vec::new(),
                messages,
            ))));
        }

        let tags = tags.cloned().unwrap_or_default();

        // Resolve the host: already in URL, in the `Host` header, or in the `@host` tag
        let url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else if let Some(host) = headers
            .get("host")
            .or_else(|| tags.get("host").map(|tag| &tag.value))
        {
            let sep = (!(host.ends_with('/') || url.starts_with('/')))
                .then_some("/")
                .unwrap_or_default();
            [host, sep, url].concat()
        } else {
            task.end(TaskResult::syntax_error(
                "Unable to resolve a host for the request, add one to URL, or use a Host header or @host tag",  
            ));
            return Ok(task);
        };

        // Add headers for any other tags
        // TODO

        // Should the response be assigned?
        let assign_to = tags.get("assigns").map(|tag| tag.value.clone());

        // Spawn the task to run in the background
        let symbols = self.symbols.clone();
        let join_handle = tokio::spawn(async move {
            let mut outputs = Vec::new();
            let mut messages = Vec::new();

            let request = match method {
                Get => http_utils::CLIENT.get(url),
                Head => http_utils::CLIENT.head(url),
                Post => http_utils::CLIENT.post(url).body(body),
                Put => http_utils::CLIENT.put(url).body(body),
                Patch => http_utils::CLIENT.patch(url).body(body),
                Delete => http_utils::CLIENT.delete(url),
            };

            match request.send().await {
                Ok(response) => match response.error_for_status() {
                    Ok(response) => {
                        if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
                            if content_type
                                .to_str()
                                .unwrap_or_default()
                                .ends_with("json")
                            {
                                match response.json::<Primitive>().await {
                                    Ok(primitive) => outputs.push(primitive.to_node()),
                                    Err(error) => messages.push(CodeError {
                                        error_message: error.to_string(),
                                        ..Default::default()
                                    }),
                                }
                            } else {
                                match response.text().await {
                                    Ok(string) => outputs.push(Node::String(string)),
                                    Err(error) => messages.push(CodeError {
                                        error_message: error.to_string(),
                                        ..Default::default()
                                    }),
                                }
                            }
                        }
                    }
                    Err(error) => messages.push(CodeError {
                        error_message: error.to_string(),
                        ..Default::default()
                    }),
                },
                Err(error) => messages.push(CodeError {
                    error_message: error.to_string(),
                    ..Default::default()
                }),
            }

            if let Some(assign_to) = assign_to {
                let node = outputs.first().cloned().unwrap_or(Node::Null(Null {}));
                symbols.write().await.insert(assign_to, node);
                outputs.clear();
            }

            let result = TaskResult::new(outputs, messages);
            if let Err(error) = result_sender.send(result) {
                tracing::debug!(
                    "When sending result for `HttpKernel::exec_async` task: {}",
                    error
                );
            }
        });

        // Spawn a task to listen for interruption message
        // This should finish when the `interrupter` is either triggered or dropped
        let task_id = task.id.clone();
        tokio::spawn(async move {
            if let Some(..) = interrupt_receiver.recv().await {
                tracing::debug!("Interrupting `HttpKernel::exec_async task` `{}`", task_id);
                join_handle.abort()
            }
        });

        Ok(task)
    }

    async fn exec_fork(&mut self, code: &str, lang: Format, tags: Option<&TagMap>) -> Result<Task> {
        // Fork execution can just delegate to `exec_async`
        self.exec_async(code, lang, tags).await
    }
}
