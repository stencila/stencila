use codec::{
    common::{eyre::Result, once_cell::sync::Lazy, regex::Regex, serde_json, serde_yaml, tracing},
    schema::Node,
};

use crate::decode::{decode_blocks, decode_inlines};

/// Decode any YAML front matter in a Markdown document into a `Node`
///
/// Any front matter is deserialized to a [`Node`], defaulting to the
/// [`Node::Article`] variant if `type` is not defined. If there is no front matter detected,
/// will return `None`.
///
/// Also returns the position of the ending `---` delimiter so that content
/// before that can be ignore by the calling function.
pub(super) fn decode_frontmatter(md: &str) -> Result<(usize, Option<Node>)> {
    static REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new("^-{3,}((.|\\n)*)?\\n-{3,}").expect("Unable to create regex"));

    if let Some(captures) = REGEX.captures(md) {
        let end = captures[0].len();

        let yaml = captures[1].trim().to_string();

        // Empty YAML so return None
        if yaml.is_empty() {
            return Ok((end, None));
        }

        // Deserialize YAML to a value, and add `type: Article` if necessary
        let mut value = match serde_yaml::from_str(&yaml) {
            Ok(serde_json::Value::Object(mut value)) => {
                if value.get("type").is_none() {
                    value.insert(
                        "type".to_string(),
                        serde_json::Value::String("Article".to_string()),
                    );
                }
                serde_json::Value::Object(value)
            }
            Ok(_) => {
                tracing::warn!("YAML frontmatter is not an object, will be ignored");
                return Ok((end, None));
            }
            Err(error) => {
                tracing::warn!(
                    "Error while parsing YAML frontmatter, will be ignored: {}",
                    error
                );
                return Ok((end, None));
            }
        };

        // Parse title and abstract as Markdown (need to do here before deserializing to node)
        let title = value
            .get_mut("title")
            .and_then(|value| value.as_str())
            .map(|title| decode_inlines(title).0);
        let abs = value
            .get_mut("abstract")
            .and_then(|value| value.as_str())
            .map(|abs| decode_blocks(abs).0);

        // Deserialize to a `Node` not that `type` is ensured to be present
        let mut node = serde_json::from_value(value)?;

        // Set title and abstract if node is Article
        if let Node::Article(article) = &mut node {
            article.title = title;
            article.r#abstract = abs;
        }

        Ok((end, Some(node)))
    } else {
        Ok((0, None))
    }
}
