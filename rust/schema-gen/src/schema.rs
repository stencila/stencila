//! The meta-schema for schemas in the Stencila Schema

use std::path::PathBuf;

use schemars::JsonSchema;

use common::{
    eyre::{bail, eyre, Context, Result},
    indexmap::IndexMap,
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    serde_with::skip_serializing_none,
    serde_yaml,
    smart_default::SmartDefault,
    strum::{AsRefStr, Display},
    tokio::fs::read_to_string,
};

/// A schema in the Stencila Schema
///
/// This meta-schema is based on JSON Schema with custom extensions
/// to meet the needs of Stencila Schema.
///
/// Only the JSON Schema properties actually used by Stencila Schema,
/// are included in the meta-schema. An error will be thrown if a schema
/// as an unknown property.
///
/// Much of the documentation provided here for JSON Schema properties is
/// drawn directly from
/// https://json-schema.org/draft/2020-12/json-schema-core.html and
/// https://json-schema.org/draft/2020-12/json-schema-validation.html.
///
/// The current version of this meta-schema is published a https://stencila.dev/meta.schema.json.
/// Previous versions are available via https://raw.github.com/stencila/stencila/<version>/json-schema/meta.schema.json
/// (replace `<version>` with the version tag name e.g. `v2.0.0-alpha.6`).
///
/// Stencila Schema authors should start the schema with the `$schema` keyword pointing
/// to this meta-schema. Amongst other things, this provides useful tool tips and input validation
/// in several commonly used code editors.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(
    default,
    rename_all = "camelCase",
    deny_unknown_fields,
    crate = "common::serde"
)]
pub struct Schema {
    /// The meta-schema of the schema
    ///
    /// The value of this keyword MUST be "https://stencila.dev/meta.schema.json".
    #[serde(rename = "$schema")]
    pub schema: Option<String>,

    /// The JSON Schema id for the schema
    ///
    /// The value of this keyword MUST be a URI. It is automatically
    /// generated for each schema.Stencila Schema authors should use
    /// the `@id` property instead.
    #[serde(rename = "$id")]
    pub id: Option<String>,

    /// The JSON-LD id for the schema
    ///
    /// The value of this keyword MUST be a string.
    /// If the schema belongs to another vocabulary such as schema.org, prefix the
    /// id which that. e.g. `schema:Person`, otherwise, prefix it with `stencila`.
    #[serde(rename = "@id")]
    pub jid: Option<String>,

    /// A description of the schema
    ///
    /// The value of this keyword MUST be a string.
    pub title: Option<String>,

    /// The title of the schema that this schema extends
    #[serde(
        deserialize_with = "deserialize_string_or_array",
        skip_serializing_if = "Vec::is_empty"
    )]
    #[schemars(schema_with = "schema_string_or_array")]
    pub extends: Vec<String>,

    /// Whether the schema is only an abstract base for other schemas
    ///
    /// Types are usually not generated for abstract schemas.
    #[serde(skip_serializing_if = "is_false")]
    pub r#abstract: bool,

    /// A description of the schema
    ///
    /// The value of this keyword MUST be a string.
    /// The description SHOULD be short, use `$comment` for more extensive
    /// descriptive content.
    pub description: Option<String>,

    /// Comments for the schema
    ///
    /// The value of this keyword MUST be a string.
    /// Use this for more extensive descriptive content such as the
    /// decisions made in the design of the schema.
    #[serde(rename = "$comment")]
    pub comment: Option<String>,

    /// The status of the schema
    pub status: Option<String>,

    /// Aliases which may be used for a property name
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// The stripping scopes that the property should be stripped for
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strip: Vec<StripScopes>,

    /// Options for converting the type or property to/from HTML
    pub html: Option<HtmlOptions>,

    /// A reference to another schema in Stencila Schema
    ///
    /// The value of this keyword MUST be a string of the
    /// title of the schema being referenced.
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>,

    #[rustfmt::skip]
    // Validation keywords for any instance type

    /// The value of this keyword MUST be either a string or an array.  If it
    /// is an array, elements of the array MUST be strings and MUST be
    /// unique.
    ///
    /// String values MUST be one of the six primitive types ("null",
    /// "boolean", "object", "array", "number", or "string"), or "integer"
    /// which matches any number with a zero fractional part.
    ///
    /// An instance validates if and only if the instance is in any of the
    /// sets listed for this keyword.
    pub r#type: Option<Type>,

    /// The value of this keyword MUST be an array.  This array SHOULD have
    /// at least one element.  Elements in the array SHOULD be unique.
    ///
    /// An instance validates successfully against this keyword if its value
    /// is equal to one of the elements in this keyword's array value.
    ///
    /// Elements in the array might be of any type, including null.
    pub r#enum: Option<Vec<Value>>,

    /// The value of this keyword MAY be of any type, including null.
    ///
    /// Use of this keyword is functionally equivalent to an "enum"
    /// with a single value.
    ///
    /// An instance validates successfully against this keyword if its value
    /// is equal to the value of the keyword.
    pub r#const: Option<Value>,

    #[rustfmt::skip]
    // Validation keywords for numeric instances (number and integer)

    /// The exclusive minimum valid value
    ///
    /// The value of "exclusiveMinimum" MUST be a number, representing an exclusive lower limit for a numeric instance.
    /// If the instance is a number, then the instance is valid only if it has a value strictly greater than
    /// (not equal to) "exclusiveMinimum".
    pub exclusive_minimum: Option<f64>,

    /// The minimum valid value
    ///
    /// The value of "minimum" MUST be a number, representing an inclusive lower limit for a numeric instance.
    /// If the instance is a number, then this keyword validates only if the instance is greater than or exactly
    /// equal to "minimum".
    pub minimum: Option<f64>,

    /// The exclusive maximum valid value
    ///
    /// The value of "exclusiveMaximum" MUST be a number, representing an exclusive upper limit for a numeric instance.
    /// If the instance is a number, then the instance is valid only if it has a value strictly less than
    /// (not equal to) "exclusiveMaximum".
    pub exclusive_maximum: Option<f64>,

    /// The maximum valid value
    ///
    /// The value of "maximum" MUST be a number, representing an inclusive upper limit for a numeric instance.
    /// If the instance is a number, then this keyword validates only if the instance is less than or exactly
    /// equal to "maximum".
    pub maximum: Option<f64>,

    #[rustfmt::skip]
    // Validation keywords for strings

    /// The expected format of the value
    ///
    /// The value of this keyword MUST be a string. This string SHOULD be a valid regular expression,
    /// according to the ECMA-262 regular expression dialect. A string instance is considered valid
    /// if the regular expression matches the instance successfully. Recall: regular expressions
    /// are not implicitly anchored.
    pub pattern: Option<String>,

    /// The expected format of the value
    pub format: Option<String>,

    #[rustfmt::skip]
    // Validation keywords for arrays

    /// Subschema for valid items in the array
    /// 
    /// The value of "items" MUST be a valid JSON Schema. This keyword applies its
    /// subschema to all instance array elements. 
    pub items: Option<Items>,

    /// The minimum number of items in the array
    ///
    /// The value of this keyword MUST be a non-negative integer. An array instance
    /// is valid against "minItems" if its size is greater than, or equal to, the
    /// value of this keyword. Omitting this keyword has the same behavior as a
    /// value of 0.
    pub min_items: Option<usize>,

    /// The maximum number of items in the array
    ///
    /// The value of this keyword MUST be a non-negative integer. An array instance
    /// is valid against "maxItems" if its size is less than, or equal to, the value
    /// of this keyword.
    pub max_items: Option<usize>,

    #[rustfmt::skip]
    // Validation keywords for objects

    /// The names of required properties of an object schema
    /// 
    /// The value of this keyword MUST be an array. Elements of this array, if any, MUST be strings,
    /// and MUST be unique. An object instance is valid against this keyword if every item in the array
    /// is the name of a property in the instance. Omitting this keyword has the same behavior
    /// as an empty array.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,

    /// Core properties, which although optional, should not be placed in
    /// the `options` field of generated Rust types
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub core: Vec<String>,

    /// The properties of an object schema
    ///
    /// The value of "properties" MUST be an object. Each value of this object MUST be a valid JSON Schema.
    /// Validation succeeds if, for each name that appears in both the instance and as a name within this
    /// keyword's value, the child instance for that name successfully validates against the corresponding
    /// schema. The annotation result of this keyword is the set of instance property names matched by this keyword.
    ///
    /// Omitting this keyword has the same assertion behavior as an empty object.
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub properties: IndexMap<String, Schema>,

    /// The subschema for additional properties
    ///
    /// The value of "additionalProperties" MUST be a valid JSON Schema. The behavior of this keyword
    /// depends on the presence and annotation results of "properties" and "patternProperties" within
    /// the same schema object. Validation with "additionalProperties" applies only to the child
    /// values of instance names that do not appear in the annotation results of either "properties"
    /// or "patternProperties". For all such properties, validation succeeds if the child instance
    /// validates against the "additionalProperties" schema.
    pub additional_properties: Option<Box<Schema>>,

    #[rustfmt::skip]
    // Validation keywords for unions

    /// Subschema of a union type
    /// 
    /// This keyword's value MUST be a non-empty array. Each item of the array MUST be a valid JSON Schema.
    /// An instance validates successfully against this keyword if it validates successfully against at least
    /// one schema defined by this keyword's value. Note that when annotations are being collected, all
    /// subschemas MUST be examined so that annotations are collected from each subschema that validates
    /// successfully.
    pub any_of: Option<Vec<Schema>>,

    /// A default value for the schema
    ///
    /// There are no restrictions placed on the value of this keyword. When multiple occurrences
    /// of this keyword are applicable to a single sub-instance, implementations SHOULD remove
    /// duplicates. This keyword can be used to supply a default JSON value associated with a
    /// particular schema. It is RECOMMENDED that a default value be valid against the associated schema.
    pub default: Option<Value>,

    #[rustfmt::skip]
    // Derived properties, not intended to be specified in schema, but
    // used internally when generating code etc.

    /// The schema that the property is defined on
    #[serde(skip)]
    pub defined_on: String,

    /// Whether this is a property schema and is inherited from another
    /// schema that the _parent_ schema extends.
    #[serde(skip)]
    pub is_inherited: bool,

    /// Whether this is a property schema and is required (is in the `required` keyword
    /// of the _parent_ schema).
    #[serde(skip)]
    pub is_required: bool,

    /// Whether this is a property schema and is core (is in the `core` keyword
    /// of the _parent_ schema).
    #[serde(skip)]
    pub is_core: bool,

    /// Whether the `extend()` method has been run on this schema yet
    #[serde(skip)]
    pub is_extended: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, AsRefStr, JsonSchema)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum Type {
    String,
    Number,
    Integer,
    Boolean,
    Object,
    Array,
    Null,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, crate = "common::serde")]
pub enum Value {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Object(IndexMap<String, Value>),
    Array(Vec<Value>),
    #[serde(rename = "null")]
    Null,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, crate = "common::serde")]
pub enum Items {
    // This should be `Option<Box<Schema>>` but serde have difficulty resolving
    // the non-list variants given that the properties are optional
    Ref(ItemsRef),
    Type(ItemsType),
    AnyOf(ItemsAnyOf),
    List(Vec<Schema>),
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "common::serde")]
pub struct ItemsRef {
    #[serde(rename = "$ref")]
    pub r#ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "common::serde")]
pub struct ItemsType {
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "common::serde")]
pub struct ItemsAnyOf {
    #[serde(rename = "anyOf", skip_serializing_if = "Vec::is_empty")]
    pub r#any_of: Vec<Schema>,
}

/// Targets for stripping properties
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Display)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
#[strum(serialize_all = "lowercase")]
pub enum StripScopes {
    /// Strip the `id` property only
    ///
    /// This MUST only ever be used on the `Entity.id` property.
    Id,

    /// Strip all code related properties
    ///
    /// Includes any properties that define the execution of a node e.g.
    ///
    /// - `code` and `programmingLanguage` of any `CodeExecutable` node
    /// - `source` of a `Include` or `Call` node
    Code,

    /// Strip all execution related properties
    ///
    /// Includes any properties that record the execution state of a node e.g.
    ///
    /// - `executionCount` of any `Executable` node
    Execution,

    /// Strip all output related properties
    ///
    /// Includes any properties that are the result of executing a node e.g.
    ///
    /// - `outputs` of a `CodeChunk` node
    /// - `output` of a `CodeExpression` node
    /// - `content` of a `Include` or `Call` node
    Output,
}

/// Options for conversion to/from HTML
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", crate = "common::serde")]
pub struct HtmlOptions {
    /// The name of the HTML element to use for a type or property
    pub elem: Option<String>,

    /// Whether a custom element is defined for the node type
    ///
    /// If this is `true`, then, if `elem` is not `None`, the HTML `is` attribute
    /// will be set to the kebab-cased type name.
    #[serde(skip_serializing_if = "is_false")]
    pub custom: bool,

    /// The HTML attribute name for a property
    ///
    /// Should only be used when `elem` is not `None`. When `elem` is `None`,
    /// the name of the attribute will be the name of the property.
    pub attr: Option<String>,

    /// Whether a property should be encoded as content of the parent element
    #[serde(skip_serializing_if = "is_false")]
    pub content: bool,

    /// Whether a property should be encoded as a slot of the parent element
    /// and the HTML element (e.g. `div`) to use for that slot
    pub slot: Option<String>,
}

impl Schema {
    /// Read a `schema/*.yaml` file into a [`Schema`] object
    pub async fn read(file: PathBuf) -> Result<(String, Schema)> {
        let yaml = read_to_string(&file)
            .await
            .context(format!("unable to read file `{}`", file.display()))?;

        let mut schema: Self = serde_yaml::from_str(&yaml)
            .context(format!("unable to deserialize file `{}`", file.display()))?;

        let title = file
            .file_name()
            .and_then(|name| {
                name.to_string_lossy()
                    .strip_suffix(".yaml")
                    .map(String::from)
            })
            .expect("all files to have a prefix");

        schema.schema = Some("https://stencila.dev/meta.schema.json".to_string());
        schema.id = Some(format!("https://stencila.dev/{title}.schema.json"));

        Ok((title, schema))
    }

    /// Check and normalize the schema
    ///
    /// This performs normalization on fields to make subsequent steps, as well as
    /// code generation easier.
    pub fn normalize(&mut self, name: &str, is_prop: bool) -> Result<()> {
        if !is_prop {
            let Some(title) = &mut self.title else {
                bail!("schema does not have a title")
            };
            if title != name {
                bail!("title is not the same as the name of file")
            }
        }

        if self.description.is_none() {
            bail!("schema does not have a description")
        };

        for (name, property) in self.properties.iter_mut() {
            property.normalize(name, true)?;
        }

        Ok(())
    }

    /// Extend the schema by inheriting properties of it's parent
    ///
    /// Also inherits `required` and `core` from parent.
    pub fn extend(&self, name: &str, schemas: &mut IndexMap<String, Schema>) -> Result<Schema> {
        let mut parents: Vec<Schema> = self
            .extends
            .iter()
            .map(|extend| {
                let mut parent = schemas
                    .get(extend)
                    .ok_or_else(|| eyre!("no schema matching `extends` keyword: {}", extend))
                    .unwrap()
                    .clone();
                if !parent.is_extended {
                    parent = parent.extend(extend, schemas).unwrap();
                }
                parent
            })
            .collect();

        let mut extended = self.clone();

        let mut properties: IndexMap<String, Schema> = parents
            .iter_mut()
            .flat_map(|parent| std::mem::take(&mut parent.properties).into_iter())
            .chain(extended.properties.clone().into_iter())
            .collect();
        let cores: Vec<String> = parents
            .iter_mut()
            .flat_map(|parent| std::mem::take(&mut parent.core).into_iter())
            .chain(extended.core.into_iter())
            .collect();
        let requireds: Vec<String> = parents
            .iter_mut()
            .flat_map(|parent| std::mem::take(&mut parent.required).into_iter())
            .chain(extended.required.into_iter())
            .collect();

        for (property_name, property) in properties.iter_mut() {
            if extended.properties.contains_key(property_name) {
                property.defined_on = name.to_string()
            } else {
                property.is_inherited = true;
            }

            if requireds.contains(property_name) {
                property.is_required = true;
            }
            if cores.contains(property_name) {
                property.is_core = true;
            }
        }

        extended.properties = properties;
        extended.required = requireds;
        extended.core = cores;
        extended.is_extended = true;

        schemas.insert(name.to_string(), extended.clone());

        Ok(extended)
    }
}

/// Deserialize an optional string or array of strings field into an `Vec<String>`
fn deserialize_string_or_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: common::serde::Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(untagged, crate = "common::serde")]
    enum StringOrArray {
        String(String),
        Array(Vec<String>),
    }
    // Try to deserialize the value as a single string or an array of strings
    match StringOrArray::deserialize(deserializer)? {
        StringOrArray::String(s) => Ok(vec![s]),
        StringOrArray::Array(arr) => {
            let result: Vec<String> = arr.into_iter().collect();
            Ok(result)
        }
    }
}

/// Set the JSON Schema as allowing for an optional string or array of strings
fn schema_string_or_array(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    serde_json::from_value(json!({
        "anyOf": [
            {
                "type": "string"
            },
            {
                "type": "array",
                "items": {
                    "type": "string"
                }
            }
        ]
    }))
    .expect("invalid JSON Schema")
}

/// Is a boolean false?
fn is_false(bool: &bool) -> bool {
    !(*bool)
}
