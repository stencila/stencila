use crate::prelude::*;

/// A node, of any type, in the current document
///
/// Used in cases that the specific node type is not known
/// (i.e. in fields where the `Node` enum is valid such as `CodeChunk.outputs`).
///
/// For convenience, the type of the node is available as a property, and
/// the value of the node is both a JavaScript object and as Markdown.
#[derive(Clone, Trace)]
#[rquickjs::class]
pub struct Node {
    /// The type of the node
    #[qjs(get, enumerable)]
    r#type: String,

    /// The node as a JSON string
    #[qjs(get, enumerable)]
    json: String,

    /// The node as a Markdown string
    #[qjs(get, enumerable)]
    markdown: String,
}

impl Node {
    #[cfg(test)]
    pub fn new(node_type: &str, json: &str, md: &str) -> Self {
        Self {
            r#type: node_type.into(),
            json: json.into(),
            markdown: md.into(),
        }
    }
}

impl From<&schema::Node> for Node {
    fn from(node: &schema::Node) -> Self {
        Self {
            r#type: node.node_type().to_string(),
            json: serde_json::to_string_pretty(node).unwrap_or_default(),
            markdown: to_markdown(node),
        }
    }
}

#[rquickjs::methods]
impl Node {
    /// Get the JavaScript value of the node
    #[qjs(get, enumerable)]
    fn value<'js>(&self, ctx: Ctx<'js>) -> Result<Value<'js>, rquickjs::Error> {
        ctx.json_parse(self.json.clone())
    }

    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("type", self.r#type.clone())?;
        obj.set("json", self.json.clone())?;
        obj.set("markdown", self.markdown.clone())?;
        Ok(obj)
    }
}
