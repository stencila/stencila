use eyre::Result;
use stencila_schema::{Node, SoftwareSourceCode};

/// Decode input to a type of `SoftwareSourceCode` with `text`
/// and `programming_language` properties.
pub fn decode(text: &str, programming_language: &str) -> Result<Node> {
    let programming_language = match programming_language.is_empty() {
        true => None,
        false => Some(Box::new(programming_language.to_string())),
    };
    Ok(Node::SoftwareSourceCode(SoftwareSourceCode {
        text: Some(Box::new(text.to_string())),
        programming_language,
        ..Default::default()
    }))
}
