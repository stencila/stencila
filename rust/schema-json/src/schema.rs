use eyre::Result;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

pub type Map<K, V> = IndexMap<K, V>;

/// A JSON Schema representation that can be serialized to valid JSON Schema
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JsonSchema {
    /// The JSON Schema version identifier
    #[serde(rename = "$schema")]
    schema: Option<String>,

    /// The schema identifier URI
    #[serde(rename = "$id")]
    id: Option<String>,

    /// The schema title
    title: Option<String>,

    /// The schema description
    description: Option<String>,

    /// The schema type(s)
    #[serde(rename = "type")]
    r#type: Option<SchemaType>,

    /// Object properties
    properties: Option<Map<String, JsonSchema>>,

    /// Required property names
    required: Option<Vec<String>>,

    /// Array items schema
    items: Option<Box<JsonSchema>>,

    /// Additional array items schema
    additional_items: Option<Box<JsonSchema>>,

    /// Additional properties schema
    additional_properties: Option<AdditionalProperties>,

    /// Schema must match any of these schemas
    any_of: Option<Vec<JsonSchema>>,

    /// Schema must match exactly one of these schemas
    one_of: Option<Vec<JsonSchema>>,

    /// Schema must match all of these schemas
    all_of: Option<Vec<JsonSchema>>,

    /// Schema must not match this schema
    not: Option<Box<JsonSchema>>,

    /// Reference to another schema
    #[serde(rename = "$ref")]
    reference: Option<String>,

    /// Schema definitions
    definitions: Option<Map<String, JsonSchema>>,

    /// Enum values
    #[serde(rename = "enum")]
    enum_values: Option<Vec<Value>>,

    /// Constant value
    #[serde(rename = "const")]
    const_value: Option<Value>,

    /// Default value
    default: Option<Value>,

    /// Examples
    examples: Option<Vec<Value>>,

    // String validation
    min_length: Option<u32>,

    max_length: Option<u32>,

    pattern: Option<String>,

    format: Option<String>,

    // Numeric validation
    minimum: Option<f32>,

    maximum: Option<f32>,

    exclusive_minimum: Option<f32>,

    exclusive_maximum: Option<f32>,

    multiple_of: Option<f32>,

    // Array validation
    min_items: Option<u32>,

    max_items: Option<u32>,

    unique_items: Option<bool>,

    // Object validation
    min_properties: Option<u32>,

    max_properties: Option<u32>,

    /// Internal field for deferred references (not serialized)
    #[serde(skip)]
    deferred_ref: Option<Box<JsonSchema>>,
}

/// JSON Schema type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null,
}

/// Additional properties handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Bool(bool),
    Schema(Box<JsonSchema>),
}

/// Create a deferred reference to a schema that will be resolved during
/// `standalone()` processing
pub fn refer(schema: JsonSchema) -> JsonSchema {
    // Verify the schema has a title for reference generation
    schema
        .title
        .as_ref()
        .expect("Schema must have a title to be used as a reference");

    JsonSchema {
        deferred_ref: Some(Box::new(schema)),
        ..JsonSchema::new()
    }
}

impl JsonSchema {
    /// Create a new empty JSON Schema
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a reference to another schema
    pub fn reference<S: Into<String>>(reference: S) -> Self {
        Self {
            reference: Some(reference.into()),
            ..Self::new()
        }
    }

    /// Create a null schema
    pub fn null() -> Self {
        Self::new().r#type(SchemaType::Null)
    }

    /// Create a boolean schema
    pub fn boolean() -> Self {
        Self::new().r#type(SchemaType::Boolean)
    }

    /// Create an integer schema
    pub fn integer() -> Self {
        Self::new().r#type(SchemaType::Integer)
    }

    /// Create a number schema
    pub fn number() -> Self {
        Self::new().r#type(SchemaType::Number)
    }

    /// Create a string schema
    pub fn string() -> Self {
        Self::new().r#type(SchemaType::String)
    }

    /// Create an array schema
    pub fn array() -> Self {
        Self::new().r#type(SchemaType::Array)
    }

    /// Create an object schema
    pub fn object() -> Self {
        Self::new().r#type(SchemaType::Object)
    }

    /// Create an enum schema with string values
    pub fn string_enum<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let enum_values = values
            .into_iter()
            .map(|s| Value::String(s.into()))
            .collect();
        Self::string().enum_values(enum_values)
    }

    /// Create a constant string schema
    pub fn string_const<S: Into<String>>(value: S) -> Self {
        Self::string().const_value(Value::String(value.into()))
    }

    // Builder methods

    /// Set the schema ID
    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the schema title
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the schema description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the schema type
    pub fn r#type(mut self, schema_type: SchemaType) -> Self {
        self.r#type = Some(schema_type);
        self
    }

    /// Add a property to the schema
    pub fn property<S: Into<String>>(mut self, name: S, schema: JsonSchema) -> Self {
        self.properties
            .get_or_insert_with(Map::new)
            .insert(name.into(), schema);
        self
    }

    /// Set all properties at once
    pub fn properties(mut self, properties: Map<String, JsonSchema>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set required fields
    pub fn required<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required = Some(fields.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Set array items schema
    pub fn items(mut self, items: JsonSchema) -> Self {
        self.items = Some(Box::new(items));
        self
    }

    /// Set additional properties
    pub fn additional_properties(mut self, additional: AdditionalProperties) -> Self {
        self.additional_properties = Some(additional);
        self
    }

    /// Allow additional properties
    pub fn allow_additional_properties(mut self) -> Self {
        self.additional_properties = Some(AdditionalProperties::Bool(true));
        self
    }

    /// Disallow additional properties
    pub fn disallow_additional_properties(mut self) -> Self {
        self.additional_properties = Some(AdditionalProperties::Bool(false));
        self
    }

    /// Add an anyOf schema
    pub fn any_of_schema(mut self, schema: JsonSchema) -> Self {
        self.any_of.get_or_insert_with(Vec::new).push(schema);
        self
    }

    /// Set anyOf schemas
    pub fn any_of(mut self, schemas: Vec<JsonSchema>) -> Self {
        self.any_of = Some(schemas);
        self
    }

    /// Add a oneOf schema
    pub fn one_of_schema(mut self, schema: JsonSchema) -> Self {
        self.one_of.get_or_insert_with(Vec::new).push(schema);
        self
    }

    /// Set oneOf schemas
    pub fn one_of(mut self, schemas: Vec<JsonSchema>) -> Self {
        self.one_of = Some(schemas);
        self
    }

    /// Add an allOf schema
    pub fn all_of_schema(mut self, schema: JsonSchema) -> Self {
        self.all_of.get_or_insert_with(Vec::new).push(schema);
        self
    }

    /// Set allOf schemas
    pub fn all_of(mut self, schemas: Vec<JsonSchema>) -> Self {
        self.all_of = Some(schemas);
        self
    }

    /// Set not schema
    pub fn not(mut self, schema: JsonSchema) -> Self {
        self.not = Some(Box::new(schema));
        self
    }

    /// Add a definition
    pub fn definition<S: Into<String>>(mut self, name: S, schema: JsonSchema) -> Self {
        self.definitions
            .get_or_insert_with(Map::new)
            .insert(name.into(), schema);
        self
    }

    /// Set enum values
    pub fn enum_values(mut self, values: Vec<Value>) -> Self {
        self.enum_values = Some(values);
        self
    }

    /// Set const value
    pub fn const_value(mut self, value: Value) -> Self {
        self.const_value = Some(value);
        self
    }

    /// Set default value
    pub fn default_value(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Add an example
    pub fn example(mut self, example: Value) -> Self {
        self.examples.get_or_insert_with(Vec::new).push(example);
        self
    }

    /// Set examples
    pub fn examples(mut self, examples: Vec<Value>) -> Self {
        self.examples = Some(examples);
        self
    }

    // String validation builders

    /// Set minimum string length
    pub fn min_length(mut self, min_length: u32) -> Self {
        self.min_length = Some(min_length);
        self
    }

    /// Set maximum string length
    pub fn max_length(mut self, max_length: u32) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set string pattern
    pub fn pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set string format
    pub fn format<S: Into<String>>(mut self, format: S) -> Self {
        self.format = Some(format.into());
        self
    }

    // Numeric validation builders

    /// Set minimum value
    pub fn minimum(mut self, minimum: f32) -> Self {
        self.minimum = Some(minimum);
        self
    }

    /// Set maximum value
    pub fn maximum(mut self, maximum: f32) -> Self {
        self.maximum = Some(maximum);
        self
    }

    /// Set exclusive minimum value
    pub fn exclusive_minimum(mut self, exclusive_minimum: f32) -> Self {
        self.exclusive_minimum = Some(exclusive_minimum);
        self
    }

    /// Set exclusive maximum value
    pub fn exclusive_maximum(mut self, exclusive_maximum: f32) -> Self {
        self.exclusive_maximum = Some(exclusive_maximum);
        self
    }

    /// Set multiple of constraint
    pub fn multiple_of(mut self, multiple_of: f32) -> Self {
        self.multiple_of = Some(multiple_of);
        self
    }

    // Array validation builders

    /// Set minimum array items
    pub fn min_items(mut self, min_items: u32) -> Self {
        self.min_items = Some(min_items);
        self
    }

    /// Set maximum array items
    pub fn max_items(mut self, max_items: u32) -> Self {
        self.max_items = Some(max_items);
        self
    }

    /// Set unique items constraint
    pub fn unique_items(mut self, unique_items: bool) -> Self {
        self.unique_items = Some(unique_items);
        self
    }

    // Object validation builders

    /// Set minimum object properties
    pub fn min_properties(mut self, min_properties: u32) -> Self {
        self.min_properties = Some(min_properties);
        self
    }

    /// Set maximum object properties
    pub fn max_properties(mut self, max_properties: u32) -> Self {
        self.max_properties = Some(max_properties);
        self
    }

    // Utility methods

    /// Serialize this schema to a JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse a JSON Schema from a JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Check if this schema is a reference
    pub fn is_reference(&self) -> bool {
        self.reference.is_some()
    }

    /// Get the reference target if this is a reference schema
    pub fn get_reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }

    /// Check if this schema has definitions
    pub fn has_definitions(&self) -> bool {
        self.definitions.as_ref().is_some_and(|d| !d.is_empty())
    }

    /// Get a definition by name
    pub fn get_definition(&self, name: &str) -> Option<&JsonSchema> {
        self.definitions.as_ref().and_then(|d| d.get(name))
    }

    /// Merge definitions from another schema
    pub fn merge_definitions_from(mut self, other: &JsonSchema) -> Self {
        if let Some(other_defs) = &other.definitions {
            for (key, value) in other_defs {
                self = self.definition(key.clone(), value.clone());
            }
        }
        self
    }

    /// Create a complete JSON Schema wrapper around a definition schema
    /// This converts a "definition-only" schema into a full JSON Schema document
    /// and processes all deferred references
    pub fn standalone(definition: JsonSchema) -> JsonSchema {
        let mut definitions_map = Map::new();
        let processed_schema = Self::process_deferred_refs(definition, &mut definitions_map);

        let mut result = JsonSchema::new();

        // Copy the main schema properties from the processed schema
        if let Some(title) = &processed_schema.title {
            result = result.title(title.clone());
        }
        if let Some(description) = &processed_schema.description {
            result = result.description(description.clone());
        }
        if let Some(id) = &processed_schema.id {
            result = result.id(id.clone());
        }
        if let Some(schema_type) = &processed_schema.r#type {
            result = result.r#type(schema_type.clone());
        }
        if let Some(properties) = &processed_schema.properties {
            result = result.properties(properties.clone());
        }
        if let Some(required) = &processed_schema.required {
            result = result.required(required.clone());
        }
        if let Some(items) = &processed_schema.items {
            result = result.items((**items).clone());
        }
        if let Some(any_of) = &processed_schema.any_of {
            result = result.any_of(any_of.clone());
        }
        if let Some(one_of) = &processed_schema.one_of {
            result = result.one_of(one_of.clone());
        }
        if let Some(all_of) = &processed_schema.all_of {
            result = result.all_of(all_of.clone());
        }
        if let Some(examples) = &processed_schema.examples {
            result = result.examples(examples.clone());
        }

        // Ensure top level schema has additionalProperties:false
        result = result.disallow_additional_properties();

        // Add all collected definitions
        if !definitions_map.is_empty() {
            result.definitions = Some(definitions_map);
        }

        // Merge any existing definitions from the original schema
        result.merge_definitions_from(&processed_schema)
    }

    /// Process deferred references recursively, replacing them with $ref and collecting definitions
    fn process_deferred_refs(
        schema: JsonSchema,
        definitions: &mut Map<String, JsonSchema>,
    ) -> JsonSchema {
        // If this schema is a deferred reference, resolve it
        if let Some(ref_schema) = &schema.deferred_ref {
            let def_name = ref_schema
                .title
                .as_ref()
                .expect("Referenced schema must have a title")
                .clone();

            // Process the referenced schema recursively first
            let processed_ref = Self::process_deferred_refs((**ref_schema).clone(), definitions);

            // Add the processed schema to definitions if not already present
            if !definitions.contains_key(&def_name) {
                definitions.insert(def_name.clone(), processed_ref);
            }

            // Return a reference to the definition
            return JsonSchema::reference(format!("#/definitions/{def_name}"));
        }

        // Process nested schemas
        let mut result = schema.clone();

        // Clear the deferred_ref field in the result
        result.deferred_ref = None;

        // Process properties
        if let Some(properties) = &schema.properties {
            let mut processed_props = Map::new();
            for (key, prop_schema) in properties {
                processed_props.insert(
                    key.clone(),
                    Self::process_deferred_refs(prop_schema.clone(), definitions),
                );
            }
            result.properties = Some(processed_props);
        }

        // Process array items
        if let Some(items) = &schema.items {
            result.items = Some(Box::new(Self::process_deferred_refs(
                (**items).clone(),
                definitions,
            )));
        }

        // Process anyOf
        if let Some(any_of) = &schema.any_of {
            let processed_any_of = any_of
                .iter()
                .map(|s| Self::process_deferred_refs(s.clone(), definitions))
                .collect();
            result.any_of = Some(processed_any_of);
        }

        // Process oneOf
        if let Some(one_of) = &schema.one_of {
            let processed_one_of = one_of
                .iter()
                .map(|s| Self::process_deferred_refs(s.clone(), definitions))
                .collect();
            result.one_of = Some(processed_one_of);
        }

        // Process allOf
        if let Some(all_of) = &schema.all_of {
            let processed_all_of = all_of
                .iter()
                .map(|s| Self::process_deferred_refs(s.clone(), definitions))
                .collect();
            result.all_of = Some(processed_all_of);
        }

        // Process not
        if let Some(not_schema) = &schema.not {
            result.not = Some(Box::new(Self::process_deferred_refs(
                (**not_schema).clone(),
                definitions,
            )));
        }

        // Process additional properties if it's a schema
        if let Some(AdditionalProperties::Schema(additional)) = &schema.additional_properties {
            result.additional_properties = Some(AdditionalProperties::Schema(Box::new(
                Self::process_deferred_refs((**additional).clone(), definitions),
            )));
        }

        // Process existing definitions
        if let Some(existing_defs) = &schema.definitions {
            let mut processed_defs = Map::new();
            for (key, def_schema) in existing_defs {
                processed_defs.insert(
                    key.clone(),
                    Self::process_deferred_refs(def_schema.clone(), definitions),
                );
            }
            result.definitions = Some(processed_defs);
        }

        result
    }

    /// Transform this schema recursively using the provided function
    fn transform_recursively<F>(mut self, transform_fn: F) -> Self
    where
        F: Fn(JsonSchema) -> JsonSchema + Clone,
    {
        // Apply transformation to this schema
        self = transform_fn(self);

        // Recursively transform properties
        if let Some(properties) = &self.properties {
            let mut transformed_props = Map::new();
            for (key, prop_schema) in properties {
                transformed_props.insert(
                    key.clone(),
                    prop_schema
                        .clone()
                        .transform_recursively(transform_fn.clone()),
                );
            }
            self.properties = Some(transformed_props);
        }

        // Recursively transform array items
        if let Some(items) = &self.items {
            self.items = Some(Box::new(
                (**items)
                    .clone()
                    .transform_recursively(transform_fn.clone()),
            ));
        }

        // Recursively transform anyOf
        if let Some(any_of) = &self.any_of {
            let transformed_any_of = any_of
                .iter()
                .map(|s| s.clone().transform_recursively(transform_fn.clone()))
                .collect();
            self.any_of = Some(transformed_any_of);
        }

        // Recursively transform oneOf
        if let Some(one_of) = &self.one_of {
            let transformed_one_of = one_of
                .iter()
                .map(|s| s.clone().transform_recursively(transform_fn.clone()))
                .collect();
            self.one_of = Some(transformed_one_of);
        }

        // Recursively transform allOf
        if let Some(all_of) = &self.all_of {
            let transformed_all_of = all_of
                .iter()
                .map(|s| s.clone().transform_recursively(transform_fn.clone()))
                .collect();
            self.all_of = Some(transformed_all_of);
        }

        // Recursively transform not
        if let Some(not_schema) = &self.not {
            self.not = Some(Box::new(
                (**not_schema)
                    .clone()
                    .transform_recursively(transform_fn.clone()),
            ));
        }

        // Recursively transform additional properties if it's a schema
        if let Some(AdditionalProperties::Schema(additional)) = &self.additional_properties {
            self.additional_properties = Some(AdditionalProperties::Schema(Box::new(
                (**additional)
                    .clone()
                    .transform_recursively(transform_fn.clone()),
            )));
        }

        // Recursively transform definitions
        if let Some(definitions) = &self.definitions {
            let mut transformed_defs = Map::new();
            for (key, def_schema) in definitions {
                transformed_defs.insert(
                    key.clone(),
                    def_schema
                        .clone()
                        .transform_recursively(transform_fn.clone()),
                );
            }
            self.definitions = Some(transformed_defs);
        }

        self
    }

    /// Transform schema for Mistral API compatibility
    ///
    /// See: https://docs.mistral.ai/capabilities/function_calling/
    pub fn for_mistral(self) -> Self {
        self.transform_recursively(|mut schema| {
            // Remove unsupported string validation
            schema.format = None;
            schema.pattern = None;

            // Ensure objects disallow additional properties (required by Mistral)
            if schema.r#type == Some(SchemaType::Object) {
                schema = schema.disallow_additional_properties();
            }

            schema
        })
    }

    /// Transform schema for Google Gemini API compatibility
    ///
    /// See: https://ai.google.dev/api/generate-content#FIELDS.response_json_schema
    pub fn for_google(self) -> Self {
        self.transform_recursively(|mut schema| {
            // Convert const to single-value enum
            if let Some(const_val) = schema.const_value.take() {
                schema.enum_values = Some(vec![const_val]);
            }

            // Do not remove other properties since Google says they are ignored.

            schema
        })
    }

    /// Transform schema for OpenAI API compatibility
    ///
    /// See: https://platform.openai.com/docs/guides/structured-outputs#supported-schemas
    pub fn for_openai(self) -> Self {
        self.transform_recursively(|mut schema| {
            // Remove unsupported schema composition
            schema.all_of = None;
            schema.not = None;

            // Remove default values (not supported)
            schema.default = None;

            // Ensure objects disallow additional properties (required by OpenAI)
            if schema.r#type == Some(SchemaType::Object) {
                schema = schema.disallow_additional_properties();
            }

            schema
        })
    }
}
