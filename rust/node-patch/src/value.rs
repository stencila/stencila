use common::{
    itertools::Itertools,
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    serde_json,
    strum::AsRefStr,
};
use stencila_schema::{BlockContent, InlineContent, Primitive};
use unicode_segmentation::UnicodeSegmentation;

/// Type for the `value` property of `Add` and `Replace` operations
///
/// Variants are required for node types that will be values in
/// add and replace patch operations AND which are represented by HTML element
/// (rather than HTML attributes of elements). This includes things like content
/// types as well as their child node types not included in `InlineContent`
/// or `BlockContent` (e.g. `IfClause`, `CallArgument`, `CodeError`).
/// 
/// In addition, adding variants for other types can be beneficial for
/// performance of in-memory diffing and patching because instead of them being serialized
/// to/from JSON values they are held in memory. When patches are coming in from a
/// client this will not help because the deserialization needs to be done anyway.
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
    Primitive(Primitive),
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
            Self::Inline(value) => Some(value.to_html(&mut context)),
            Self::Block(value) => Some(value.to_html(&mut context)),
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
            Self::Null
        } else if let Some(string) = value.as_str() {
            Self::String(string.to_owned())
        } else {
            Self::Json(value)
        };

        Ok(value)
    }
}

impl Serialize for Value {
    /// Implement `Serialize` for [`Value`]
    ///
    /// Necessary for sending patches to clients. All variants should
    /// be handled.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => None::<bool>.serialize(serializer),
            Self::String(value) => value.serialize(serializer),
            Self::Primitive(value) => value.serialize(serializer),
            Self::Inline(value) => value.serialize(serializer),
            Self::Block(value) => value.serialize(serializer),
            Self::Json(value) => value.serialize(serializer),
        }
    }
}

/// A sequence of values
#[derive(Clone, Debug)]
pub struct Values(pub Vec<Value>);

impl Values {
    pub fn from_single(value: Value) -> Self {
        Self(vec![value])
    }

    pub fn from_pair(first: Value, second: Value) -> Self {
        Self(vec![first, second])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, value: Value) {
        self.0.push(value)
    }
}

impl<'de> Deserialize<'de> for Values {
    /// Implement `Deserialize` for [`Values`]
    ///
    /// Necessary for receiving operations with `Many` operations from clients.
    /// If all the value is a string then it is split into a vector of
    /// graphemes.
    fn deserialize<D>(deserializer: D) -> Result<Values, D::Error>
    where
        D: Deserializer<'de>,
    {
        use common::serde::de::Error;
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

        let vec = if let Some(string) = value.as_str() {
            string
                .graphemes(true)
                .map(|grapheme| Value::String(grapheme.to_string()))
                .collect_vec()
        } else {
            serde_json::from_value::<Vec<Value>>(value)
                .map_err(|_| D::Error::custom("Expected a string or an array of values"))?
        };

        Ok(Self(vec))
    }
}

impl Serialize for Values {
    /// Implement `Serialize` for [`Values`]
    ///
    /// Necessary for sending operations with `Many` operations to clients.
    /// If all the values are strings then serialized as a joined string,
    /// otherwise serialized as an array of values.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut all_strings = true;
        let mut joined_strings = String::new();
        for value in &self.0 {
            if let Value::String(grapheme) = value {
                joined_strings.push_str(grapheme);
            } else {
                all_strings = false;
                break;
            }
        }

        if all_strings {
            joined_strings.serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}
