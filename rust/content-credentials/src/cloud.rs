//! Client for Stencila Cloud C2PA signing.
//!
//! The `/v1/sign` endpoint is a Stencila extension hosted alongside the C2PA
//! Soft Binding Resolution API. It is not part of the official C2PA API.

use std::{fs, path::Path};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use stencila_schema::Graph;

use stencila_version::STENCILA_USER_AGENT;

use crate::{
    error::{Error, Result},
    graph::graph_assertion_payload,
    producer::{SoftBindingAssertion, SoftBindingRegistration},
    signer::CredentialCloudSigningConfig,
    snapshot::IngredientSnapshot,
};

/// Request for Cloud signing an asset.
pub(crate) struct CloudSignRequest<'a> {
    pub input_path: &'a Path,
    pub media_type: &'a str,
    pub title: &'a str,
    pub assertion: &'a Graph,
    pub ingredients: &'a [IngredientSnapshot],
    pub soft_bindings: &'a [SoftBindingAssertion],
    pub embed: bool,
}

/// Signed bytes returned by the Cloud signing service.
pub(crate) struct CloudSignedAsset {
    pub asset: Vec<u8>,
    pub sidecar: Option<Vec<u8>>,
    pub soft_binding_registrations: Vec<SoftBindingRegistration>,
    pub warnings: Vec<String>,
}

/// Client for the Stencila C2PA Cloud service.
pub(crate) struct CloudSigningClient {
    config: CredentialCloudSigningConfig,
    client: Client,
}

impl CloudSigningClient {
    /// Create a Cloud signing client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn new(config: CredentialCloudSigningConfig) -> Result<Self> {
        let client = Client::builder()
            .user_agent(STENCILA_USER_AGENT)
            .build()
            .map_err(Error::Http)?;

        Ok(Self { config, client })
    }

    /// Sign an asset using `POST /v1/sign`.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset cannot be read, the request cannot be
    /// encoded, the service rejects the request, or the response cannot be
    /// decoded.
    pub async fn sign(&self, request: CloudSignRequest<'_>) -> Result<CloudSignedAsset> {
        self.config.require_authenticated()?;

        let asset_bytes = fs::read(request.input_path)?;
        let file_name = request
            .input_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("asset")
            .to_string();

        let payload = CloudSignPayload {
            media_type: request.media_type,
            title: request.title,
            assertion: graph_assertion_payload(request.assertion)?,
            ingredients: request.ingredients,
            soft_bindings: request.soft_bindings,
            manifest: if request.embed {
                CloudManifestMode::Embedded
            } else {
                CloudManifestMode::Sidecar
            },
            register_soft_binding: self.config.register_soft_binding,
        };
        let payload_json = serde_json::to_string(&payload)?;
        let payload_part = multipart::Part::text(payload_json).mime_str("application/json")?;
        let asset_part = multipart::Part::bytes(asset_bytes)
            .file_name(file_name)
            .mime_str(request.media_type)?;
        let form = multipart::Form::new()
            .part("request", payload_part)
            .part("asset", asset_part);

        let mut http_request = self.client.post(self.endpoint()).multipart(form);
        if let Some(token) = self
            .config
            .token
            .as_deref()
            .filter(|token| !token.is_empty())
        {
            http_request = http_request.bearer_auth(token);
        }

        let response = http_request.send().await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::CloudSigningFailed(format!(
                "Stencila Cloud signing failed with status {status}: {body}"
            )));
        }

        let response: CloudSignResponse = response.json().await?;
        Ok(CloudSignedAsset {
            asset: STANDARD.decode(response.asset)?,
            sidecar: response
                .sidecar
                .map(|sidecar| STANDARD.decode(sidecar))
                .transpose()?,
            soft_binding_registrations: response.bindings,
            warnings: response.warnings,
        })
    }

    fn endpoint(&self) -> String {
        format!("{}/sign", self.config.base_url.trim_end_matches('/'))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudSignPayload<'a> {
    media_type: &'a str,
    title: &'a str,
    assertion: serde_json::Value,
    ingredients: &'a [IngredientSnapshot],
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    soft_bindings: &'a [SoftBindingAssertion],
    manifest: CloudManifestMode,
    register_soft_binding: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum CloudManifestMode {
    Embedded,
    Sidecar,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloudSignResponse {
    asset: String,
    sidecar: Option<String>,
    #[serde(default)]
    bindings: Vec<SoftBindingRegistration>,
    #[serde(default)]
    warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_appends_sign_to_base_url() -> Result<()> {
        let client = CloudSigningClient::new(
            CredentialCloudSigningConfig::resolve().with_base_url("https://c2pa.example.test/v1/"),
        )?;

        assert_eq!(client.endpoint(), "https://c2pa.example.test/v1/sign");
        Ok(())
    }

    #[test]
    fn sign_response_preserves_service_metadata() -> Result<()> {
        let response: CloudSignResponse = serde_json::from_value(serde_json::json!({
            "asset": "",
            "sidecar": null,
            "bindings": [{
                "alg": "io.iscc.v0",
                "bindingValue": "ISCC:test",
                "similarityScore": 100
            }],
            "warnings": ["soft bindings are not supported for application/pdf"]
        }))?;

        assert_eq!(response.bindings.len(), 1);
        assert_eq!(response.bindings[0].binding_value, "ISCC:test");
        assert_eq!(
            response.warnings,
            vec!["soft bindings are not supported for application/pdf".to_string()]
        );
        Ok(())
    }
}
