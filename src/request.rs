use crate::methods::Method;
use crate::nodes::Node;
use crate::rpc::Response;
use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use strum::VariantNames;

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Request a method call on a peer",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub struct Args {
        /// URL of the peer (e.g. ws://example.org:9001, :9000)
        url: String,

        /// Method name
        #[structopt(possible_values = Method::VARIANTS, case_insensitive = true)]
        method: Method,

        /// Method parameters
        #[structopt(raw(true))]
        params: Vec<String>,
    }

    pub async fn request(args: Args) -> Result<Node> {
        let Args {
            url,
            method,
            params,
        } = args;

        let mut object = serde_json::json!({});
        for param in params {
            let parts: Vec<&str> = param.split('=').collect();
            let (name, value) = (parts[0], parts[1]);
            object[name] = match serde_json::from_str(value) {
                Ok(value) => value,
                Err(_) => serde_json::Value::String(value.to_string()),
            };
        }

        super::request(url, method, object).await
    }
}

pub async fn request(url: String, method: Method, params: serde_json::Value) -> Result<Node> {
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
        "http" | "https" => request_http(&url, &request).await?,
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

#[cfg(feature = "request-http")]
async fn request_http(url: &str, request: &serde_json::Value) -> Result<Response> {
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(request)
        .send()
        .await?
        .json::<Response>()
        .await?;
    Ok(response)
}

#[cfg(feature = "request-http")]
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
