use common::{
    eyre::{bail, Result},
    serde_json,
};
use stencila_schema::Node;

/// Supported query languages
pub const LANGS: [&str; 2] = ["jmespath", "jsonptr"];

/// Query a node
///
/// Returns a JSON value. Returns `null` if the query does not select anything.
///
/// # Arguments
///
pub fn query(node: &Node, query: &str, lang: Option<&str>) -> Result<serde_json::Value> {
    let lang = lang.unwrap_or(LANGS[0]);
    Ok(match lang {
        #[cfg(feature = "jmespath")]
        "jmespath" => {
            let expr = jmespatch::compile(query)?;
            let result = expr.search(node)?;
            serde_json::to_value(result)?
        }
        #[cfg(feature = "jsonptr")]
        "jsonptr" => {
            let data = serde_json::to_value(node)?;
            let result = data.pointer(query);
            match result {
                Some(value) => value.clone(),
                None => serde_json::Value::Null,
            }
        }
        _ => bail!("Unknown query language '{}'", lang),
    })
}
