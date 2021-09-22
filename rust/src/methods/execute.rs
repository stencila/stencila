use eyre::Result;
use maplit::hashmap;
use stencila_schema::Node;

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub async fn execute(node: &mut Node) -> Result<Node> {
    #[cfg(feature = "plugins")]
    return crate::plugins::delegate(
        super::Method::Execute,
        hashmap! { "node".to_string() => serde_json::to_value(node)? },
    )
    .await;

    #[cfg(not(feature = "plugins"))]
    eyre::bail!("Unable to execute node")
}
