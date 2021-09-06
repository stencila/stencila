use crate::sessions::SESSIONS;
use defaults::Defaults;
use eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use strum::{Display, EnumString, EnumVariantNames};

#[derive(Debug, Display, EnumString, EnumVariantNames, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Protocol {
    #[cfg(any(feature = "request-stdio", feature = "serve-stdio"))]
    Stdio,
    #[cfg(any(feature = "request-http", feature = "serve-http"))]
    Http,
    #[cfg(any(feature = "request-ws", feature = "serve-ws"))]
    Ws,
}

type Params = HashMap<String, serde_json::Value>;

/// A JSON-RPC 2.0 request
///
/// See <https://www.jsonrpc.org/specification#request_object>.
#[skip_serializing_none]
#[derive(Debug, Clone, Defaults, Serialize, Deserialize)]
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

    pub async fn dispatch(self, client: &str) -> (Response, Subscription) {
        let result: Result<(serde_json::Value, Subscription)> = match self.method.as_str() {
            "sessions.start" => sessions_start(&self.params).await,
            "sessions.stop" => sessions_stop(&self.params).await,
            "sessions.subscribe" => sessions_subscribe(&self.params, client).await,
            "sessions.unsubscribe" => sessions_unsubscribe(&self.params, client).await,
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
                // Otherwise, convert it into a generic `server_error`.
                let message = error.to_string();
                let error = match error.downcast::<Error>() {
                    Ok(error) => error,
                    Err(_) => Error::server_error(&message),
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
#[derive(Debug, Defaults, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

async fn sessions_start(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let project = required_string(params, "project")?;
    let snapshot = required_string(params, "snapshot")?;

    let session = SESSIONS.start(&project, &snapshot).await?;
    Ok((json!(session), Subscription::None))
}

async fn sessions_stop(params: &Params) -> Result<(serde_json::Value, Subscription)> {
    let session = required_string(params, "session")?;

    let session = SESSIONS.stop(&session).await?;
    Ok((json!(session), Subscription::None))
}

async fn sessions_subscribe(
    params: &Params,
    client: &str,
) -> Result<(serde_json::Value, Subscription)> {
    let session = required_string(params, "session")?;
    let topic = required_string(params, "topic")?;

    let (session, topic) = SESSIONS.subscribe(&session, &topic, client).await?;
    Ok((json!(session), Subscription::Subscribe(topic)))
}

async fn sessions_unsubscribe(
    params: &Params,
    client: &str,
) -> Result<(serde_json::Value, Subscription)> {
    let session = required_string(params, "session")?;
    let topic = required_string(params, "topic")?;

    let (session, topic) = SESSIONS.unsubscribe(&session, &topic, client).await?;
    Ok((json!(session), Subscription::Unsubscribe(topic)))
}

// Helper functions for getting JSON-RPC parameters and raising appropriate errors
// if they are not present or of wrong type

fn required_string(params: &Params, name: &str) -> Result<String> {
    if let Some(param) = params.get(name) {
        if let Some(param) = param.as_str() {
            Ok(param.to_string())
        } else {
            bail!(Error::invalid_param_error(&format!(
                "Parameter `{}` is expected to be a string",
                name
            )))
        }
    } else {
        bail!(Error::invalid_param_error(&format!(
            "Parameter `{}` is required",
            name
        )))
    }
}
