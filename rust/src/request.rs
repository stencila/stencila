use crate::jwt;
use crate::methods::Method;
use crate::nodes::Node;
use crate::rpc::Response;
use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use std::env;
use strum::VariantNames;

/// Make a JSON-RPC request to a plugin or a peer
pub async fn request(
    url: String,
    method: Method,
    params: serde_json::Value,
    key: Option<String>,
) -> Result<Node> {
    // Ensure that url is fully formed
    let url = if url.starts_with(':') {
        format!("http://127.0.0.1{}", url)
    } else {
        let re = Regex::new("https?|wss?").unwrap();
        match re.captures(&url) {
            Some(_) => url,
            None => format!("http://{}", url),
        }
    };

    // Construct a JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    });

    // Dispatch to functions based on URL scheme
    let parsed = url::Url::parse(&url)?;
    let scheme = parsed.scheme();
    let response = match scheme {
        #[cfg(feature = "request-http")]
        "http" | "https" => request_http(&url, &request, key).await?,
        #[cfg(feature = "request-ws")]
        "ws" | "wss" => request_ws(&url, &request).await?,
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

/// Make a request over HTTP
#[cfg(feature = "request-http")]
async fn request_http(
    url: &str,
    request: &serde_json::Value,
    key: Option<String>,
) -> Result<Response> {
    let client = reqwest::Client::new();
    let request = client
        .post(url)
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
            let jwt = jwt::encode(key)?;
            request.header("authorization", jwt::to_auth_header(jwt))
        }
        None => request,
    };
    let response = request.send().await?.json::<Response>().await?;
    Ok(response)
}

/// Make a request over Websocket
#[cfg(feature = "request-ws")]
async fn request_ws(url: &str, request: &serde_json::Value) -> Result<Response> {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let (ws, _) = tokio_tungstenite::connect_async(url)
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

    pub async fn request(args: Args) -> Result<Node> {
        let Args {
            url,
            method,
            params,
            key,
        } = args;

        let re = Regex::new(r"(\w+)(:?=)(.+)").unwrap();
        let mut object = serde_json::json!({});
        for param in params {
            if let Some(captures) = re.captures(param.as_str()) {
                let (name, kind, value) = (&captures[1], &captures[2], &captures[3]);
                if kind == ":=" {
                    object[name] = match serde_json::from_str(value) {
                        Ok(value) => value,
                        Err(_) => serde_json::Value::String(value.to_string()),
                    };
                } else {
                    object[name] = serde_json::Value::from(value);
                }
            }
        }

        super::request(url, method, object, key).await
    }
}
