use common::{
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    serde_json,
    strum::AsRefStr,
};
use stencila_schema::{BlockContent, InlineContent};

/// Type for the `value` property of `Add` and `Replace` operations
///
/// Variants are required for node types that will be values in
/// add and replace patch operations AND which are represented by HTML element
/// (rather than HTML attributes of elements). This includes things like content
/// types as well as their child node types not included in `InlineContent`
/// or `BlockContent` (e.g. `IfClause`, `CallArgument`, `CodeError`).
///
/// The `Json` variant acts as a catch all for node types that do not have a
/// corresponding variant.
///
/// For all the variants (apart from `Json`), the corresponding `Patchable` implementations
/// should implement `to_value` and `from_value` and `from_value` should handle `Json` values.
#[derive(Debug, Clone, AsRefStr)]
pub enum Value {
    Null,
    String(String),
    Inline(InlineContent),
    Block(BlockContent),
    Json(serde_json::Value),
}

impl Default for Value {
    fn default() -> Self {
        Self::Null
    }
}

impl Value {
    /// Generate HTML for a [`Value`]
    ///
    /// HTML is only required for node types that are represented by a HTML element
    /// e.g. `InlineContent`, `BlockContent`, `Person`, `Organization`.
    /// If necessary everything else can be represented in HTML as an element
    /// attribute (even arrays and object can be JSON serialized in browser client to JSON
    /// attributes).
    pub fn to_html(&self, root: &stencila_schema::Node) -> Option<String> {
        use codec_html::{EncodeContext, ToHtml};

        let mut context = EncodeContext {
            root,
            ..Default::default()
        };

        match self {
            Value::Inline(value) => Some(value.to_html(&mut context)),
            Value::Block(value) => Some(value.to_html(&mut context)),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    /// Implement `Deserialize` for [`Value`]
    ///
    /// Necessary for patches sent by clients. Note that there is no
    /// benefit (and actually there seems to be a cost) to attempting
    /// to narrow the type for complex node types e.g. `InlineContent`
    /// because that requires `value.clone()` for each attempted narrowing.
    /// Instead `from_value` implementations for types should handle the
    /// `Json` variant (i.e. do the narrowing at the point is is needed,
    /// not before).
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

        let value = if value.is_null() {
            Value::Null
        } else if let Some(string) = value.as_str() {
            Value::String(string.to_owned())
        } else {
            Value::Json(value)
        };

        Ok(value)
    }
}

impl Serialize for Value {
    /// Implement `Deserialize` for [`Value`]
    ///
    /// Necessary for sending patches to clients. All variants should
    /// be handled.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null => None::<bool>.serialize(serializer),
            Value::String(value) => value.serialize(serializer),
            Value::Inline(value) => value.serialize(serializer),
            Value::Block(value) => value.serialize(serializer),
            Value::Json(value) => value.serialize(serializer),
        }
    }
}
