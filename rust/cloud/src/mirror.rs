use std::{env, fmt};

use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

use crate::{api_token, base_url, client};

const LOCAL_MIRROR_URL: &str = "STENCILA_MIRROR_URL";

fn endpoint() -> String {
    env::var(LOCAL_MIRROR_URL).unwrap_or_else(|_| format!("{}/mirror/rpc", base_url()))
}

#[derive(Debug, Deserialize)]
pub struct Error {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl Error {
    fn transport(error: impl fmt::Display) -> Self {
        Self {
            code: -32000,
            message: error.to_string(),
            data: None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for Error {}

#[derive(Serialize)]
struct JsonRpcRequest<'a, Params> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: Params,
}

#[derive(Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// Call the hosted enterprise mirror JSON-RPC endpoint.
///
/// Set `STENCILA_MIRROR_URL` to call a locally running mirror directly. When
/// unset, this calls the Stencila Cloud API at `/v1/mirror/rpc` using the
/// configured Stencila API token.
pub async fn call<Params, Output, RemoteError>(
    method: &str,
    params: Params,
) -> std::result::Result<Output, RemoteError>
where
    Params: Serialize,
    Output: DeserializeOwned,
    RemoteError: DeserializeOwned + From<Error>,
{
    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method,
        params,
    };

    let endpoint = endpoint();
    let response = if env::var(LOCAL_MIRROR_URL).is_ok() {
        let mut request_builder = Client::new().post(endpoint).json(&request);
        if let Some(token) = api_token() {
            request_builder = request_builder.header("Authorization", format!("Bearer {token}"));
        }
        request_builder.send().await
    } else {
        client()
            .await
            .map_err(Error::transport)
            .map_err(RemoteError::from)?
            .post(endpoint)
            .json(&request)
            .send()
            .await
    }
    .map_err(Error::transport)
    .map_err(RemoteError::from)?;

    let status = response.status();
    if !status.is_success() {
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| format!("mirror returned HTTP status {status}"));
        return Err(RemoteError::from(Error::transport(format!(
            "mirror returned HTTP status {status}: {message}"
        ))));
    }

    let mut response = response
        .json::<Map<String, Value>>()
        .await
        .map_err(Error::transport)
        .map_err(RemoteError::from)?;

    if let Some(error) = response.remove("error").filter(|error| !error.is_null()) {
        let error = serde_json::from_value::<JsonRpcError>(error)
            .map_err(Error::transport)
            .map_err(RemoteError::from)?;
        let error = Error {
            code: error.code,
            message: error.message,
            data: error.data,
        };

        if let Some(data) = error.data.clone()
            && let Ok(error) = serde_json::from_value(data)
        {
            return Err(error);
        }

        return Err(RemoteError::from(error));
    }

    let result = response
        .remove("result")
        .ok_or_else(|| Error::transport("mirror response did not include result or error"))
        .map_err(RemoteError::from)?;

    serde_json::from_value(result)
        .map_err(Error::transport)
        .map_err(RemoteError::from)
}
