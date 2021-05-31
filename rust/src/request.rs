use crate::jwt;
use crate::methods::prelude::Method;
use crate::rpc::{Error, Response};
use async_trait::async_trait;
use enum_dispatch::enum_dispatch;
use eyre::{bail, eyre, Context, Result};
use serde::Serialize;
use std::env;
use strum::VariantNames;
use tokio::io::AsyncWriteExt;

/// Trait for request clients. This allows us to use `enum_dispatch` to
/// dispatch these methods based on the type of client.
#[async_trait]
#[enum_dispatch]
trait ClientTrait {
    async fn send(&mut self, request: &serde_json::Value) -> Result<Response>;
}

/// A request client
#[enum_dispatch(ClientTrait)]
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Client {
    #[cfg(feature = "request-stdio")]
    Stdio(ClientStdio),
    #[cfg(feature = "request-docker")]
    Docker(ClientDocker),
    #[cfg(feature = "request-http")]
    Http(ClientHttp),
    #[cfg(feature = "request-ws")]
    Ws(ClientWs),
}

impl Client {
    /// Create a new client for a server at a URL
    pub fn new(url: &str, key: Option<String>) -> Result<Client> {
        let parts = url.split("://").collect::<Vec<_>>();
        let scheme = parts[0];
        let rest = parts[1];
        let client = match scheme {
            #[cfg(feature = "request-stdio")]
            "stdio" => {
                let command = rest.split_whitespace().map(str::to_string).collect();
                Client::Stdio(ClientStdio::new(command)?)
            }

            #[cfg(feature = "request-docker")]
            "docker" => Client::Docker(ClientDocker::new(parts[1])),

            #[cfg(feature = "request-http")]
            "http" | "https" => Client::Http(ClientHttp::new(url, key)),

            #[cfg(feature = "request-ws")]
            "ws" | "wss" => Client::Ws(ClientWs::new(url)),

            _ => bail!(
                "Unsupported request protocol '{}'; is 'request-{}' features enabled?",
                scheme,
                scheme
            ),
        };
        Ok(client)
    }

    /// Make a JSON-RPC method call to a plugin or peer.
    ///
    /// ```
    /// use stencila::methods::Method;
    /// use stencila::request::Client;
    /// use serde_json::json;
    ///
    /// let mut client = Client::new("docker://stencila/rasta", None).unwrap();
    /// client.call(
    ///     Method::Execute,
    ///     &json!({
    ///         "type": "CodeChunk",
    ///         "programmingLanguage": "r",
    ///         "text": "plot(mtcars)",
    ///     })
    /// );
    /// ```
    pub async fn call(
        &mut self,
        method: Method,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Construct a JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        // Send request to client
        let response = self.send(&request).await?;

        // Handle the response
        let Response { result, error, .. } = response;
        match result {
            Some(result) => Ok(result),
            None => match error {
                Some(error) => Err(eyre!(error.message)),
                None => bail!("Response has neither a result nor an error"),
            },
        }
    }
}

#[cfg(feature = "request-stdio")]
use tokio::{
    io::{BufReader, BufWriter},
    process::{ChildStderr, ChildStdin, ChildStdout, Command},
};

#[cfg(feature = "request-stdio")]
/// A standard input / output client
#[derive(Debug, Default, Serialize)]
pub struct ClientStdio {
    command: Vec<String>,

    #[serde(skip)]
    writer: Option<BufWriter<ChildStdin>>,
    #[serde(skip)]
    reader: Option<BufReader<ChildStdout>>,
    #[serde(skip)]
    logger: Option<BufReader<ChildStderr>>,
}

#[cfg(feature = "request-stdio")]
impl ClientStdio {
    // Create a stdio client
    pub fn new(command: Vec<String>) -> Result<Self> {
        let mut client = Self {
            command,
            ..Default::default()
        };
        client.start()?;
        Ok(client)
    }

    #[tracing::instrument]
    pub fn start(&mut self) -> Result<()> {
        tracing::debug!("Starting Stdio client");

        let mut child = Command::new(&self.command[0])
            .args(&self.command[1..])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| eyre!("Child has no stdin handle"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| eyre!("Child has no stdout handle"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| eyre!("Child has no stderr handle"))?;

        self.writer = Some(BufWriter::new(stdin));
        self.reader = Some(BufReader::new(stdout));
        // TODO send logger lines to tracing
        self.logger = Some(BufReader::new(stderr));

        let _child_thread = tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("Child process encountered an error");

            println!("Child status was: {}", status);
        });

        Ok(())
    }
}

#[async_trait]
impl ClientTrait for ClientStdio {
    /// Send a JSON-RPC request
    #[tracing::instrument(skip(self))]
    async fn send(&mut self, request: &serde_json::Value) -> Result<Response> {
        use tokio::io::AsyncBufReadExt;

        if let Some(writer) = &mut self.writer {
            tracing::debug!("Sending request: {:?}", request);
            let json = serde_json::to_string(&request)? + "\n";
            writer.write_all(json.as_bytes()).await?;
            writer.flush().await?
        }

        if let Some(reader) = &mut self.reader {
            if let Some(line) = reader.lines().next_line().await? {
                tracing::debug!("Received response: {:?}", line);
                let response: Response = serde_json::from_str(&line)?;
                return Ok(response);
            }
        }

        if let Some(logger) = &mut self.logger {
            if let Some(line) = logger.lines().next_line().await? {
                tracing::debug!("Received log entry: {:?}", line);
            }
        }

        let response = Response {
            error: Some(Error::server_error(
                "Did not get a response from the stdio server",
            )),
            ..Default::default()
        };
        Ok(response)
    }
}

#[cfg(feature = "request-docker")]
/// A Docker container client
#[derive(Debug, Serialize)]
pub struct ClientDocker {}

#[cfg(feature = "request-docker")]
impl ClientDocker {
    // Create a Docker client
    pub fn new(_image: &str) -> Self {
        Self {}
    }
}

#[async_trait]
impl ClientTrait for ClientDocker {
    /// Send a JSON-RPC request
    async fn send(&mut self, _request: &serde_json::Value) -> Result<Response> {
        todo!();
    }
}

#[cfg(feature = "request-http")]
/// A HTTP client
#[derive(Debug, Serialize)]
pub struct ClientHttp {
    url: String,

    key: Option<String>,

    #[serde(skip)]
    client: reqwest::Client,
}

#[cfg(feature = "request-http")]
impl ClientHttp {
    // Create a HTTP client
    pub fn new(url: &str, key: Option<String>) -> Self {
        Self {
            url: url.to_string(),
            key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl ClientTrait for ClientHttp {
    /// Send a JSON-RPC request
    async fn send(&mut self, request: &serde_json::Value) -> Result<Response> {
        let request = self
            .client
            .post(&self.url)
            .header(
                "user-agent",
                format!(
                    "Stencila/{} ({})",
                    env!("CARGO_PKG_VERSION"),
                    env::consts::OS
                ),
            )
            .json(request);
        let request = match self.key.clone() {
            Some(key) => {
                let jwt = jwt::encode(key, Some(60))?;
                request.header("authorization", jwt::to_auth_header(jwt))
            }
            None => request,
        };
        let response = request.send().await?.json::<Response>().await?;
        Ok(response)
    }
}

#[cfg(feature = "request-ws")]
/// A WebSocket client
#[derive(Debug, Serialize)]
pub struct ClientWs {
    url: String,
}

#[cfg(feature = "request-ws")]
impl ClientWs {
    // Create a WebSocket client
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

#[async_trait]
impl ClientTrait for ClientWs {
    /// Send a JSON-RPC request
    async fn send(&mut self, request: &serde_json::Value) -> Result<Response> {
        use futures::{SinkExt, StreamExt};
        use tokio_tungstenite::tungstenite::Message;

        let (ws, _) = tokio_tungstenite::connect_async(&self.url)
            .await
            .context("Connecting to server")?;

        let (mut write, read) = ws.split();

        let json = serde_json::to_string(request)?;
        write.send(Message::Text(json)).await?;

        read.for_each(|message| async {
            let data = message.unwrap().into_data();
            println!("{}", String::from_utf8(data).unwrap());
            // TODO: Look up the id and resolve that future
        })
        .await;

        let response: Response = Default::default();
        Ok(response)
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Request a method call on a plugin or peer",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Args {
        /// URL of the peer (e.g. ws://example.org:9001, :9000)
        url: String,

        /// Method name (e.g. `convert`)
        #[structopt(possible_values = Method::VARIANTS, case_insensitive = true)]
        method: Method,

        /// Method parameters (after `--`) as strings (e.g. `format=json`) or JSON (e.g. `node:='{"type":...}'`)
        #[structopt(raw(true))]
        params: Vec<String>,

        /// Secret key to use for signing JSON Web Tokens
        #[structopt(short, long)]
        key: Option<String>,
    }

    pub async fn run(args: Args) -> Result<()> {
        let Args {
            url,
            method,
            params,
            key,
        } = args;

        let mut client = Client::new(&url, key)?;
        let params = crate::cli::args::params(&params);
        let result = client.call(method, &params).await?;
        println!("{}", serde_json::to_string_pretty(&result)?);

        Ok(())
    }
}
