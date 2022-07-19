use common::{eyre::Result, serde_json::json};
use http_utils::CLIENT;
use stencila_schema::Node;

use crate::{api, errors::*, orgs::org_default, types::ProjectLocal, utils::token_read};

/// Get the URL of a node
pub fn node_url(key: &str) -> String {
    api!("nodes/{key}")
}

/// Create a new node
pub async fn node_create(
    key: &str,
    node: &Node,
    org_id: Option<u64>,
    project_id: Option<u64>,
) -> Result<()> {
    let org_id = org_id.or_else(|| org_default().ok());

    let project_id =
        project_id.or_else(|| ProjectLocal::current().ok().and_then(|project| project.id));

    let response = CLIENT
        .post(api!("nodes"))
        .bearer_auth(token_read()?)
        .json(&json!({
            "key": key,
            "json": node,
            "org_id": org_id,
            "project_id": project_id,
        }))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Error::from_response(response).await
    }
}

/// Retrieve a node
pub async fn node_retrieve(key: &str) -> Result<Node> {
    let response = CLIENT
        .get(node_url(key))
        .bearer_auth(token_read()?)
        .send()
        .await?;

    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Error::from_response(response).await
    }
}
