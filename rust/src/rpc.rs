use crate::nodes::Node;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericRequest<P> {
    /// A string specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Option<String>,

    /// An identifier for the request established by the client
    /// The standard allows this to be a number or a string but here we use
    /// `u64` because it proved to require less code and is probably more efficient.
    /// May be `None` for notifications from the server
    pub id: Option<u64>,

    /// Parameters of the method call
    pub params: P,
}

/// A JSON-RPC 2.0 request
///
/// @see {@link https://www.jsonrpc.org/specification#request_object}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "lowercase")]
pub enum Request {
    Decode(GenericRequest<crate::decode::rpc::Params>),
    Execute(GenericRequest<crate::execute::rpc::Params>),
}

impl Request {
    pub fn id(&self) -> Option<u64> {
        match self {
            Request::Decode(request) => request.id,
            Request::Execute(request) => request.id,
        }
    }

    pub fn dispatch(self) -> Result<Node> {
        match self {
            Request::Decode(request) => crate::decode::rpc::decode(request.params),
            Request::Execute(request) => crate::execute::rpc::execute(request.params),
        }
    }
}

/// A JSON-RPC 2.0 response
///
/// @see {@link https://www.jsonrpc.org/specification#response_object}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Response {
    /// A string specifying the version of the JSON-RPC protocol.
    pub jsonrpc: Option<String>,

    /// The id of the request that this response is for
    /// May be `None` for notifications from the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    /// The result of the method call
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Node>,

    /// Any error that may have occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

impl Response {
    pub fn new(id: Option<u64>, result: Option<Node>, error: Option<anyhow::Error>) -> Self {
        Response {
            id,
            result,
            error: match error {
                Some(error) => Some(Error::server_error(&error.to_string())),
                None => None,
            },
            ..Default::default()
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self {
            jsonrpc: Some("2.0".to_string()),
            id: None,
            result: None,
            error: None,
        }
    }
}

/// A JSON-RPC 2.0 error
///
/// @see {@link https://www.jsonrpc.org/specification#error_object}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Error {
    /// A number that indicates the error type that ocurred
    pub code: i16,

    /// A string providing a short description of the error
    pub message: String,

    /// A value that contains additional information about the error
    pub data: Option<serde_json::Value>,
}

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
