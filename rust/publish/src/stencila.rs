use cloud::ErrorResponse;
use common::{
    eyre::{bail, eyre, Result},
    reqwest::{
        multipart::{Form, Part},
        Client,
    },
    serde::Serialize,
    serde_json,
    tempfile::TempDir,
    tokio,
};
use schema::Node;

#[derive(Serialize)]
#[serde(crate = "common::serde")]
struct Manifest {
    key: String,
}

/// Publish a single node to Stencila Cloud
pub(super) async fn publish_node(node: &Node, key: &Option<String>) -> Result<()> {
    let token = cloud::api_key().as_ref().ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or key chain entry found. Get one at https://stencila.cloud/."))?;

    let manifest = Manifest {
        key: key.as_deref().unwrap_or_default().to_string(),
    };
    let manifest = serde_json::to_string(&manifest)?;
    let manifest = Part::text(manifest);

    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path().join("publish.swb");
    codecs::to_path(&node, &temp_path, None).await?;

    let bundle: Vec<u8> = tokio::fs::read(temp_path).await?;
    let bundle = Part::bytes(bundle).file_name("publish.swb");

    let form = Form::new()
        .part("manifest", manifest)
        .part("bundle", bundle);

    let response = Client::new()
        .put(format!("{}/sites", cloud::base_url()))
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
