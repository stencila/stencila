//! Handling of errors for Stencila Cloud API

use common::{
    eyre::{bail, Result},
    serde::{Deserialize, Serialize},
};
use http_utils::reqwest::Response;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ErrorPayload {
    pub error: Error,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Error {
    message: String,
    hint: Option<String>,
}

impl Error {
    pub async fn response_to_string(response: Response) -> String {
        let status = response.status().as_u16();
        if let Ok(payload) = response.json::<ErrorPayload>().await {
            let error = payload.error;
            let hint = error
                .hint
                .map(|hint| format!(". {}", hint))
                .unwrap_or_else(String::new);
            format!("{}{} [{}]", error.message, hint, status)
        } else {
            format!("Unknown error [{}]", status)
        }
    }

    pub async fn from_response<T>(response: Response) -> Result<T> {
        bail!("{}", Error::response_to_string(response).await)
    }
}
