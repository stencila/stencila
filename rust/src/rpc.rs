use crate::nodes::Node;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenericRequest<P> {
    pub jsonrpc: Option<String>,

    // Using `u64` proved to require less code (and is probably more efficient) than
    // using `String`.
    pub id: Option<u64>,

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

    pub fn method(&self) -> &str {
        match self {
            Request::Decode(_) => "decode",
            Request::Execute(_) => "execute",
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
    pub jsonrpc: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Node>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Error>,
}

impl Response {
    pub fn new(id: Option<u64>, result: Option<Node>, error: Option<anyhow::Error>) -> Self {
        Response {
            id,
            result,
            error: match error {
                Some(error) => Some(Error {
                    code: -32000,
                    message: error.to_string(),
                }),
                None => None,
            },
            ..Default::default()
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Response {
            jsonrpc: "2.0".to_string(),
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
    pub code: i16,
    pub message: String,
}
