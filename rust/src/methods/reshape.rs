use defaults::Defaults;
use eyre::Result;
use stencila_schema::Node;

/// Reshaping options
#[derive(Defaults)]
pub struct Options {}

/// Reshape
pub fn reshape(node: Node, _options: Options) -> Result<Node> {
    // TODO
    Ok(node)
}
