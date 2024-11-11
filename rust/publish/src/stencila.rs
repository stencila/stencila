use cloud::ErrorResponse;
use codec::{Codec, EncodeOptions};
use codec_swb::SwbCodec;
use common::{
    eyre::{bail, eyre, Result},
    reqwest::{
        multipart::{Form, Part},
        Client,
    },
    serde::Serialize,
    serde_json,
    tempfile::TempDir,
    tokio, tracing,
};
use schema::Node;

#[derive(Serialize)]
#[serde(crate = "common::serde")]
struct Manifest {}

/// Publish a single node to Stencila Cloud
pub(super) async fn publish_node(
    node: &Node,
    options: EncodeOptions,
    key: &Option<String>,
    dry_run: bool,
    swb: &SwbCodec,
) -> Result<()> {
    let token = cloud::api_key().ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or key chain entry found. Get one at https://stencila.cloud/."))?;

    let key = key.as_deref().unwrap_or_default().to_string();
    let base_url = format!("https://{key}.stencila.site");

    let manifest = Manifest {};
    let manifest = serde_json::to_string(&manifest)?;
    let manifest = Part::text(manifest);

    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().join("publish.swb");
    swb.to_path(
        node,
        &temp_path,
        Some(EncodeOptions {
            base_url: Some(base_url),
            ..options
        }),
    )
    .await?;

    let bundle: Vec<u8> = tokio::fs::read(temp_path).await?;
    let bundle = Part::bytes(bundle).file_name("publish.swb");

    let form = Form::new()
        .part("manifest", manifest)
        .part("bundle", bundle);

    if dry_run {
        tracing::info!("Dry run completed");
        return Ok(());
    }

    let response = Client::new()
        .put(format!("{}/sites/{}", cloud::base_url(), key))
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        let ErrorResponse { error, .. } = response.json().await?;
        bail!("{error}")
    }
}
