use std::{collections::HashMap, str::FromStr};

use codecs::EncodeOptions;
use common::{
    defaults::Defaults,
    eyre::{bail, Result},
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    serde_with::skip_serializing_none,
    tracing,
};
use documents::{When, DOCUMENTS};
use graph::{PlanOrdering, PlanScope};
use node_patch::Patch;

type Params = HashMap<String, serde_json::Value>;

/// A JSON-RPC 2.0 request
///
/// See <https://www.jsonrpc.org/specification#request_object>.
#[skip_serializing_none]
#[derive(Debug, Clone, Defaults, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Request {
    /// A string specifying the version of the JSON-RPC protocol.
    #[def = "Some(\"2.0\".to_string())"]
    pub jsonrpc: Option<String>,

    /// An identifier for the request established by the client
    /// The standard allows this to be a number or a string but here we use
    /// `u64` because it proved to require less code and is probably more efficient.
    /// May be `None` for notifications or for requests over TTP (where there is not
    /// a need to correlate the request with the response message).
    pub id: Option<u64>,

    /// The method being called
    pub method: String,

    /// Parameters of the method
    pub params: Params,
}

impl Request {
    pub fn new(method: &str, params: Params) -> Self {
        Request {
            method: method.to_string(),
            params,
            ..Default::default()
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn dispatch(self, client: &str) -> (Response, Subscription) {
        tracing::trace!("Dispatching request for client `{}`", client);

        let result: Result<(serde_json::Value, Subscription)> = match self.method.as_str() {
            "documents.create" => documents_create(&self.params).await,
            "documents.open" => documents_open(&self.params).await,
            "documents.close" => documents_close(&self.params).await,
            "documents.write" => documents_write(&self.params).await,
            "documents.load" => documents_load(&self.params).await,
            "documents.dump" => documents_dump(&self.params).await,
            "documents.patch" => documents_patch(&self.params).await,
            "documents.compile" => documents_compile(&self.params).await,
            "documents.execute" => documents_execute(&self.params).await,
            "documents.cancel" => documents_cancel(&self.params).await,
            "documents.restart" => documents_restart(&self.params).await,
            "documents.kernels" => documents_kernels(&self.params).await,
            "documents.symbols" => documents_symbols(&self.params).await,
            "documents.subscribe" => documents_subscribe(&self.params, client).await,
            "documents.unsubscribe" => documents_unsubscribe(&self.params, client).await,
            _ => {
                let error = Error::method_not_found_error(&self.method);
                return (
                    Response::new(self.id, None, Some(error)),
                    Subscription::None,
                );
            }
        };
        match result {
            Ok((value, subscription)) => (Response::new(self.id, Some(value), None), subscription),
            Err(error) => {
                // If the error is JSON-RPC error from this module, just return that.
                // Otherwise, convert it into a generic `server_error` and log it.
                let message = error.to_string();
                let error = match error.downcast::<Error>() {
                    Ok(error) => error,
                    Err(_) => {
                        tracing::error!("{}", message);
                        Error::server_error(&message)
                    }
                };
                (
                    Response::new(self.id, None, Some(error)),
                    Subscription::None,
                )
            }
        }
    }
}

/// A JSON-RPC 2.0 notification
///
/// A `Notification` is just a `Request` with `id: None`.
///
/// See <https://www.jsonrpc.org/specification#notification>.
pub type Notification = Request;

/// A JSON-RPC 2.0 response
///
/// See <https://www.jsonrpc.org/specification#response_object>.
#[skip_serializing_none]
#[derive(Debug, Defaults, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Response {
    /// A string specifying the version of the JSON-RPC protocol.
    #[def = "Some(\"2.0\".to_string())"]
    pub jsonrpc: Option<String>,

    /// The id of the request that this response is for
    pub id: Option<u64>,

    /// The result of the method call
    pub result: Option<serde_json::Value>,

    /// Any error that may have occurred
    pub error: Option<Error>,
}

impl Response {
    pub fn new(id: Option<u64>, result: Option<serde_json::Value>, error: Option<Error>) -> Self {
        Response {
            id,
            result,
            error,
            ..Default::default()
        }
    }
}

pub enum Subscription {
    None,
    Subscribe(String),
    Unsubscribe(String),
}

/// A JSON-RPC 2.0 error
///
/// See <https://www.jsonrpc.org/specification#error_object>.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Error {
    /// A number that indicates the error type that ocurred
    pub code: i16,

    /// A string providing a short description of the error
    pub message: String,

    /// A value that contains additional information about the error
    pub data: Option<serde_json::Value>,
}

// Implement necessary traits to treat JSON-RPC errors as Rust errors
// so they can be part of a `Result`.

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC Error: {} ({})", self.message, self.code)
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Create an error
    pub fn new(code: i16, message: &str, data: Option<serde_json::Value>) -> Self {
        Self {
            code,
            message: message.to_string(),
            data,
        }
    }

    /// A parse error
    pub fn parse_error(details: &str) -> Self {
        Self {
            code: -32700,
            message: format!("Error while parsing request: {}", details),
            data: Some(serde_json::json!({ "details": details })),
        }
    }

    /// An error when a client sends an invalid request
    pub fn invalid_request_error(message: &str) -> Self {
        Self {
            code: -32600,
            message: format!("Request is invalid: {}", message),
            data: None,
        }
    }

    /// An error when the requested method does not exist
    pub fn method_not_found_error(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method '{}' does not exist", method),
            data: None,
        }
    }

    /// An error when one of more parameters are invalid
    pub fn invalid_param_error(message: &str) -> Self {
        Self {
            code: -32602,
            message: message.to_string(),
            data: None,
        }
    }

    /// A generic internal server error
    pub fn server_error(message: &str) -> Self {
        Self {
            code: -32000,
            message: message.to_string(),
            data: None,
        }
    }

    /// An error to indicate the server lacks the requested capability
    pub fn capability_error(capability: &str, method: &str, params: &serde_json::Value) -> Self {
        Self {
            code: -32001,
            message: format!("Incapable of {}", capability),
            data: Some(serde_json::json!({
                "method": method,
                "params": params
            })),
        }
    }
}

// The following are dispatching functions that check the supplied JSON arguments
// and send them on to the relevant core functions, raising errors is arguments are
// missing or of the wrong type, and converting returned values to JSON.

async fn documents_create(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let path = optional_string(params, "path")?;
    let content = optional_string(params, "content")?;
    let format = optional_string(params, "format")?;

    let id = DOCUMENTS.create(path, content, format).await?;
    Ok((json!({ "id": id }), Subscription::None))
}

async fn documents_open(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let path = required_string(params, "path")?;

    let id = DOCUMENTS.open(&path, None).await?;
    Ok((json!({ "id": id }), Subscription::None))
}

async fn documents_close(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;

    let id = DOCUMENTS.close(&id).await?;
    Ok((json!({ "id": id }), Subscription::None))
}

async fn documents_write(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let document_id = required_string(params, "documentId")?;

    // TODO: make immutable
    //DOCUMENTS.get(&document_id).await?.write(None, None).await?;
    Ok((json!(true), Subscription::None))
}

async fn documents_load(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let document_id = required_string(params, "documentId")?;
    let content = required_string(params, "content")?;
    let format = optional_string(params, "format")?;

    // TODO: make immutable
    //DOCUMENTS
    //    .get(&document_id)
    //    .await?
    //    .load(content, format)
    //    .await?;
    Ok((json!(true), Subscription::None))
}

async fn documents_dump(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let document_id = required_string(params, "documentId")?;
    let format = optional_string(params, "format")?;
    let node_id = optional_string(params, "nodeId")?;

    let content = DOCUMENTS
        .get(&document_id)
        .await?
        .dump(
            format,
            node_id,
            Some(EncodeOptions {
                compact: false,
                ..Default::default()
            }),
        )
        .await?;
    Ok((json!(content), Subscription::None))
}

async fn documents_subscribe(
    params: &Params,
    client: &str,
) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;
    let topic = required_string(params, "topic")?;

    let topic = ["documents:", &id, ":", &topic].concat();
    Ok((json!({ "id": id }), Subscription::Subscribe(topic)))
}

async fn documents_unsubscribe(
    params: &Params,
    client: &str,
) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;
    let topic = required_string(params, "topic")?;

    let topic = ["documents:", &id, ":", &topic].concat();
    Ok((json!({ "id": id }), Subscription::Unsubscribe(topic)))
}

async fn documents_patch(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;
    let patch = required_value(params, "patch")?;
    let patch: Patch = serde_json::from_value(patch)?;
    let compile = optional_string(params, "compile")?
        .and_then(|value| When::from_str(&value).ok())
        .unwrap_or(When::Soon);
    let execute = optional_string(params, "execute")?
        .and_then(|value| When::from_str(&value).ok())
        .unwrap_or(When::Never);
    let write = optional_string(params, "write")?
        .and_then(|value| When::from_str(&value).ok())
        .unwrap_or(When::Soon);

    DOCUMENTS
        .get(&id)
        .await?
        .patch_request(patch, compile, execute, write)
        .await?;
    Ok((json!(true), Subscription::None))
}

async fn documents_compile(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;
    let execute = optional_string(params, "execute")?
        .and_then(|value| When::from_str(&value).ok())
        .unwrap_or(When::Never);
    let write = optional_string(params, "write")?
        .and_then(|value| When::from_str(&value).ok())
        .unwrap_or(When::Soon);
    let node_id = optional_string(params, "nodeId")?;

    DOCUMENTS
        .get(&id)
        .await?
        .compile_request(execute, write, node_id)
        .await?;
    Ok((json!(true), Subscription::None))
}

async fn documents_execute(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;
    let write = optional_string(params, "write")?
        .and_then(|value| When::from_str(&value).ok())
        .unwrap_or(When::Soon);
    let node_id = optional_string(params, "nodeId")?;
    let ordering = match optional_string(params, "ordering")? {
        Some(ordering) => Some(PlanOrdering::from_str(&ordering)?),
        None => None,
    };

    DOCUMENTS
        .get(&id)
        .await?
        .execute_request(write, node_id, ordering, None)
        .await?;
    Ok((json!(true), Subscription::None))
}

async fn documents_cancel(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;
    let node_id = optional_string(params, "nodeId")?;
    let scope = match optional_string(params, "scope")? {
        Some(scope) => Some(PlanScope::from_str(&scope)?),
        None => None,
    };

    DOCUMENTS.get(&id).await?.cancel(node_id, scope).await?;
    Ok((json!(true), Subscription::None))
}

async fn documents_restart(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;
    let kernel_id = optional_string(params, "kernelId")?;

    DOCUMENTS.get(&id).await?.restart(kernel_id).await?;
    Ok((json!(true), Subscription::None))
}

async fn documents_kernels(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;

    let kernels = DOCUMENTS.get(&id).await?.kernels().await;
    Ok((json!(kernels), Subscription::None))
}

async fn documents_symbols(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let id = required_string(params, "documentId")?;

    let symbols = DOCUMENTS.get(&id).await?.symbols().await;
    Ok((json!(symbols), Subscription::None))
}

// Helper functions for getting JSON-RPC parameters and raising appropriate errors
// if they are not present or of wrong type

fn required_value(params: &Params, name: &str) -> Result<serde_json::Value> {
    if let Some(param) = params.get(name) {
        Ok(param.clone())
    } else {
        bail!(Error::invalid_param_error(&format!(
            "Parameter `{}` is required",
            name
        )))
    }
}

#[allow(dead_code)]
fn optional_value(params: &Params, name: &str) -> Option<serde_json::Value> {
    params.get(name).cloned()
}

fn required_string(params: &Params, name: &str) -> Result<String> {
    if let Some(param) = required_value(params, name)?.as_str() {
        Ok(param.to_string())
    } else {
        bail!(Error::invalid_param_error(&format!(
            "Parameter `{}` is expected to be a string",
            name
        )))
    }
}

fn optional_string(params: &Params, name: &str) -> Result<Option<String>> {
    let param = if let Some(param) = params.get(name) {
        param
    } else {
        return Ok(None);
    };
    if param.is_null() {
        Ok(None)
    } else if let Some(param) = param.as_str() {
        Ok(Some(param.to_string()))
    } else {
        bail!(Error::invalid_param_error(&format!(
            "Parameter `{}` is expected to be a string",
            name
        )))
    }
}
