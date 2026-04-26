use std::fmt;

use eyre::{Result, bail};
use reqwest::Client;
use serde::Deserialize;

use crate::{ErrorResponse, base_url, client};

#[derive(Deserialize)]
pub(crate) struct TokenResponse {
    pub(crate) access_token: String,
}

#[derive(Debug)]
pub(crate) enum TokenError {
    NotLinked { connect_url: Option<String> },
    RefreshFailed { connect_url: Option<String> },
    JsonParsing(String),
    Other(String),
}

impl fmt::Display for TokenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenError::NotLinked { connect_url } => {
                write!(
                    formatter,
                    "Account not connected. Connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            TokenError::RefreshFailed { connect_url } => {
                write!(
                    formatter,
                    "Failed to refresh token. Re-connect at: {}",
                    connect_url.as_deref().unwrap_or("https://stencila.cloud")
                )
            }
            TokenError::JsonParsing(message) | TokenError::Other(message) => {
                write!(formatter, "{message}")
            }
        }
    }
}

impl std::error::Error for TokenError {}

macro_rules! connection_token_error {
    (
        $(#[$meta:meta])*
        $visibility:vis enum $name:ident, $service:literal
    ) => {
        $(#[$meta])*
        #[derive(Debug)]
        $visibility enum $name {
            NotLinked { connect_url: Option<String> },
            RefreshFailed { connect_url: Option<String> },
            JsonParsing(String),
            Other(String),
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::NotLinked { connect_url } => {
                        write!(
                            formatter,
                            "{} account not connected. Connect at: {}",
                            $service,
                            connect_url.as_deref().unwrap_or("https://stencila.cloud")
                        )
                    }
                    Self::RefreshFailed { connect_url } => {
                        write!(
                            formatter,
                            "Failed to refresh {} token. Re-connect at: {}",
                            $service,
                            connect_url.as_deref().unwrap_or("https://stencila.cloud")
                        )
                    }
                    Self::JsonParsing(message) => write!(formatter, "Failed to parse response: {message}"),
                    Self::Other(message) => write!(formatter, "{message}"),
                }
            }
        }

        impl std::error::Error for $name {}

        impl From<$crate::tokens::TokenError> for $name {
            fn from(error: $crate::tokens::TokenError) -> Self {
                match error {
                    $crate::tokens::TokenError::NotLinked { connect_url } => {
                        Self::NotLinked { connect_url }
                    }
                    $crate::tokens::TokenError::RefreshFailed { connect_url } => {
                        Self::RefreshFailed { connect_url }
                    }
                    $crate::tokens::TokenError::JsonParsing(message) => Self::JsonParsing(message),
                    $crate::tokens::TokenError::Other(message) => Self::Other(message),
                }
            }
        }
    };
}

pub(crate) use connection_token_error;

pub(crate) async fn get_once_default(service_slug: &str) -> Result<String, TokenError> {
    let client = default_client().await?;
    get_once_with_client(&client, service_slug).await
}

pub(crate) async fn get_once_with_client(
    client: &Client,
    service_slug: &str,
) -> Result<String, TokenError> {
    let url = format!("{}/connections/{service_slug}/token", base_url());
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(error) => return Err(TokenError::Other(format!("Network error: {error}"))),
    };

    let status = response.status();

    match status.as_u16() {
        200 => {
            let token_response = response
                .json::<TokenResponse>()
                .await
                .map_err(|error| TokenError::JsonParsing(error.to_string()))?;
            Ok(token_response.access_token)
        }
        422 => {
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|error| TokenError::JsonParsing(error.to_string()))?;
            Err(TokenError::NotLinked {
                connect_url: error_response.url,
            })
        }
        500 => {
            let error_response = response
                .json::<ErrorResponse>()
                .await
                .map_err(|error| TokenError::JsonParsing(error.to_string()))?;
            Err(TokenError::RefreshFailed {
                connect_url: error_response.url,
            })
        }
        _ => {
            let error_msg = response
                .text()
                .await
                .unwrap_or_else(|_| format!("HTTP error: {status}"));
            Err(TokenError::Other(error_msg))
        }
    }
}

#[allow(clippy::print_stderr)]
pub(crate) async fn get_with_retry(service_name: &str, service_slug: &str) -> Result<String> {
    loop {
        let client = default_client().await?;

        match get_once_with_client(&client, service_slug).await {
            Ok(token) => return Ok(token),
            Err(TokenError::NotLinked { connect_url }) => {
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n🔗 {service_name} account not yet connected to your Stencila account.\n   Opening browser to connect your {service_name} account...\n"
                );

                if let Err(error) = webbrowser::open(url) {
                    eprintln!(
                        "⚠️  Failed to open browser: {}.\n   Please visit manually: {}\n",
                        error, url
                    );
                }

                stencila_ask::wait_for_enter("Press Enter after you've connected your account")
                    .await?;

                eprintln!("🔄 Trying again...\n");
            }
            Err(TokenError::RefreshFailed { connect_url }) => {
                let url = connect_url
                    .as_deref()
                    .unwrap_or("https://stencila.cloud/settings/connections");

                eprintln!(
                    "\n❌ Failed to refresh your {service_name} access token.\n\n   To fix:\n   1. Visit {}\n   2. Re-connect your {service_name} account\n   3. Try again\n",
                    url
                );
                bail!("{service_name} token refresh failed. Please re-connect your account.");
            }
            Err(TokenError::JsonParsing(message)) => {
                bail!("Failed to parse {service_name} token response: {message}");
            }
            Err(TokenError::Other(message)) => {
                bail!("Failed to get {service_name} access token: {message}");
            }
        }
    }
}

async fn default_client() -> Result<Client, TokenError> {
    client()
        .await
        .map_err(|error| TokenError::Other(error.to_string()))
}
