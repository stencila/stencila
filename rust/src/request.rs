use crate::jwt;
use crate::methods::Method;
use crate::nodes::Node;
use crate::rpc::{Error, Response};
use anyhow::{anyhow, bail, Context, Result};
use std::env;
use strum::VariantNames;
use tokio::io::AsyncWriteExt;

pub enum Client {
    #[cfg(feature = "request-stdio")]
    Stdio(Stdio),
    #[cfg(feature = "request-docker")]
    Docker(Docker),
    #[cfg(feature = "request-http")]
    Http(Http),
    #[cfg(feature = "request-ws")]
    Ws(Ws),
}

#[cfg(feature = "request-stdio")]
use tokio::{
    io::{BufReader, BufWriter},
    process::{ChildStderr, ChildStdin, ChildStdout, Command},
};

#[cfg(feature = "request-stdio")]
/// A standard input / output client
#[derive(Debug, Default)]
pub struct Stdio {
    command: Vec<String>,

    writer: Option<BufWriter<ChildStdin>>,
    reader: Option<BufReader<ChildStdout>>,
    logger: Option<BufReader<ChildStderr>>,
}

#[cfg(feature = "request-stdio")]
impl Stdio {
    // Create a stdio client
    pub fn new(command: Vec<String>) -> Self {
        Self {
            command,
            ..Default::default()
        }
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
            .ok_or_else(|| anyhow!("Child has no stdin handle"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Child has no stdout handle"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Child has no stderr handle"))?;

        self.writer = Some(BufWriter::new(stdin));
        self.reader = Some(BufReader::new(stdout));
        self.logger = Some(BufReader::new(stderr));

        let _child_thread = tokio::spawn(async move {
            let status = child
                .wait()
                .await
                .expect("Child process encountered an error");

            println!("child status was: {}", status);
        });

        Ok(())
    }

    /// Send a JSON-RPC request
    #[tracing::instrument(skip(self))]
    pub async fn send(&mut self, request: &serde_json::Value) -> Result<Response> {
        use tokio::io::AsyncBufReadExt;

        self.start()?;

        if let Some(writer) = &mut self.writer {
            tracing::debug!("Sending request: {:?}", request);
            let json = serde_json::to_string(&request)? + "\n";
            writer.write_all(json.as_bytes()).await?;
            writer.flush().await?
        }

        if let Some(reader) = &mut self.reader {
            if let Some(line) = reader.lines().next_line().await? {
                let response: Response = serde_json::from_str(&line)?;
                tracing::debug!("Got response: {:?}", response);
                return Ok(response);
            }
        }

        let response = Response {
            error: Some(Error::server_error("Errored")),
            ..Default::default()
        };
        Ok(response)
    }
}

#[cfg(feature = "request-docker")]
/// A Docker container client
pub struct Docker {}

#[cfg(feature = "request-docker")]
impl Docker {
    // Create a Docker client
    pub fn new(_image: &str) -> Self {
        Self {}
    }

    /// Send a JSON-RPC request
    pub async fn send(&self, _request: &serde_json::Value) -> Result<Response> {
        todo!();
    }
}

#[cfg(feature = "request-http")]
/// A HTTP client
pub struct Http {
    url: String,
    client: reqwest::Client,
}

#[cfg(feature = "request-http")]
impl Http {
    // Create a HTTP client
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Send a JSON-RPC request
    pub async fn send(&self, request: &serde_json::Value, key: Option<String>) -> Result<Response> {
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
        let request = match key {
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
pub struct Ws {
    url: String,
}

#[cfg(feature = "request-ws")]
impl Ws {
    // Create a WebSocket client
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }

    /// Send a JSON-RPC request
    pub async fn send(&self, request: &serde_json::Value) -> Result<Response> {
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

/// Make a JSON-RPC request to a plugin or peer.
///
/// This function is mostly useful for testing. It will construct a new client for each request.
/// For `stdio://` URLs that will involves spawning a new process, for `docker://` URLS
/// it will involve starting a new container. e.g.
///
/// ```
/// use stencila::request::request_url;
/// use stencila::methods::Method;
/// use serde_json::json;
///
/// request_url("docker://stencila/jesta", Method::Execute, &json!({
///     "type": "CodeExpression",
///     "programmingLanguage": "js",
///     "text": "6*7",
/// }), None);
/// ```
///
/// If you are making multiple requests it is preferable to construct a
/// client and use that e.g.
///
/// ```
/// use stencila::request::Docker;
/// use stencila::methods::Method;
/// use serde_json::json;
///
/// let client = Docker::new("stencila/rasta");
/// client.send(&json!({
///     "type": "CodeExpression",
///     "programmingLanguage": "r",
///     "text": "plot(mtcars)",
/// }));
/// ```
#[tracing::instrument]
pub async fn request_url(
    url: &str,
    method: Method,
    params: &serde_json::Value,
    key: Option<String>,
) -> Result<Node> {
    // Construct a JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    // Dispatch to functions based on URL scheme
    let parts = url.split("://").collect::<Vec<_>>();
    let scheme = parts[0];
    let rest = parts[1];
    let response = match scheme {
        #[cfg(feature = "request-stdio")]
        "stdio" => {
            let command = rest.split_whitespace().map(str::to_string).collect();
            Stdio::new(command).send(&request).await?
        }

        #[cfg(feature = "request-docker")]
        "docker" => Docker::new(parts[1]).send(&request).await?,

        #[cfg(feature = "request-http")]
        "http" | "https" => Http::new(url).send(&request, key).await?,

        #[cfg(feature = "request-ws")]
        "ws" | "wss" => Ws::new(url).send(&request).await?,

        _ => bail!(
            "Unsupported request protocol '{}'; is 'request-{}' features enabled?",
            scheme,
            scheme
        ),
    };

    // Handle the response
    let Response { result, error, .. } = response;
    match result {
        Some(result) => Ok(result),
        None => match error {
            Some(error) => Err(anyhow!(error.message)),
            None => bail!("Response has neither a result nor an error"),
        },
    }
}

/// CLI options for the `request` command
///
/// This command is mainly for testing, particularly during development.
/// It allows you to check that you can make a JSON-RPC request to a
/// plugin or peer.
#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Request a method call on a plugin or peer (mainly for testing)",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
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

        let params = crate::cli::parse_params(params);
        let result = super::request_url(&url, method, &params, key).await?;
        println!("{}", serde_json::to_string_pretty(&result)?);

        Ok(())
    }
}
