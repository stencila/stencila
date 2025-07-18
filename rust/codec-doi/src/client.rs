use codec::common::{
    eyre::Result,
    reqwest::{self, Client, header},
    serde_json, tracing,
};

use version::STENCILA_VERSION;

use crate::csl::CslItem;

pub struct DoiClient {
    client: Client,
}

impl DoiClient {
    pub fn new() -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.citationstyles.csl+json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .user_agent(format!(
                "stencila/{STENCILA_VERSION} (mailto:hello@stencila.io)"
            ))
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { client })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get(&self, doi: &str) -> Result<CslItem, DoiClientError> {
        tracing::debug!("Fetching metadata for DOI: {}", doi);

        let response = self
            .client
            .get(format!("https://doi.org/{doi}"))
            .send()
            .await?;

        match response.status().as_u16() {
            404 => return Err(DoiClientError::DoiNotFound(doi.to_string())),
            status if !response.status().is_success() => {
                return Err(DoiClientError::Http {
                    status,
                    doi: doi.to_string(),
                });
            }
            _ => {}
        }

        let csl_item = response.json::<CslItem>().await?;
        Ok(csl_item)
    }
}

#[derive(Debug)]
pub enum DoiClientError {
    DoiNotFound(String),
    Network(reqwest::Error),
    Http { status: u16, doi: String },
    Json(serde_json::Error),
}

impl std::fmt::Display for DoiClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DoiClientError::DoiNotFound(doi) => write!(f, "DOI not found: {}", doi),
            DoiClientError::Network(err) => write!(f, "Network error: {}", err),
            DoiClientError::Http { status, doi } => {
                write!(f, "HTTP error: {} for DOI {}", status, doi)
            }
            DoiClientError::Json(err) => write!(f, "JSON parsing error: {}", err),
        }
    }
}

impl std::error::Error for DoiClientError {}

impl From<reqwest::Error> for DoiClientError {
    fn from(err: reqwest::Error) -> Self {
        DoiClientError::Network(err)
    }
}

impl From<serde_json::Error> for DoiClientError {
    fn from(err: serde_json::Error) -> Self {
        DoiClientError::Json(err)
    }
}
