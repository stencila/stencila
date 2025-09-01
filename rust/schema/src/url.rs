use serde::Serialize;
use eyre::{Result, bail};

use node_path::NodePath;
use node_strip::{StripNode, StripScope, StripTargets};
use node_type::NodeType;

pub use node_url::{NodePosition, NodeUrl};

/// Create a [`NodeUrl`] with the path to the node (in a cache) to allow reconstitution
pub fn node_url_path(
    node_type: NodeType,
    path: NodePath,
    position: Option<NodePosition>,
) -> NodeUrl {
    NodeUrl {
        r#type: Some(node_type),
        path: Some(path),
        position,
        ..Default::default()
    }
}

/// Create a [`NodeUrl`] with the `jzb64` field to allow reconstitution
///
/// If the URL is more than 16k in length will successively strip
/// properties (starting with output, which tend to be large)
/// until the URL is below that.
pub fn node_url_jzb64<T>(
    node_type: NodeType,
    node: &T,
    position: Option<NodePosition>,
) -> Result<NodeUrl>
where
    T: Serialize + Clone + StripNode,
{
    // There is no official limit to the length of URLs.
    // Using the maximum length that Cloudflare Workers will accept (16k)
    const MAX_LENGTH: usize = 16_384;

    let mut url = NodeUrl {
        r#type: Some(node_type),
        jzb64: Some(NodeUrl::to_jzb64(node)?),
        position,
        ..Default::default()
    };
    if url.to_string().len() < MAX_LENGTH {
        return Ok(url);
    }

    let mut node = node.clone();
    for scope in [
        StripScope::Output,
        StripScope::Metadata,
        StripScope::Provenance,
        StripScope::Authors,
        StripScope::Compilation,
        StripScope::Execution,
        StripScope::Timestamps,
        StripScope::Content,
        StripScope::Code,
    ] {
        node.strip(&StripTargets {
            scopes: vec![scope],
            ..Default::default()
        });

        url.jzb64 = Some(NodeUrl::to_jzb64(&node)?);
        if url.to_string().len() < MAX_LENGTH {
            return Ok(url);
        }
    }

    bail!("Unable to generate node url with `jzb64` less than `{MAX_LENGTH}` chars long")
}
