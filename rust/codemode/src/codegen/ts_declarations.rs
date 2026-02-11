use std::collections::HashSet;

use serde_json::Value;

/// Context for tracking state during schema-to-TypeScript conversion.
pub(crate) struct TypeContext<'a> {
    /// The root schema document (for resolving `$ref`).
    root_schema: &'a Value,
    /// Set of `$ref` paths currently being resolved (cycle detection).
    visiting: HashSet<String>,
    /// Name to use when a recursive self-reference is detected.
    self_ref_name: Option<String>,
    /// Set to `true` if recursion was detected during conversion.
    pub has_recursion: bool,
}

impl<'a> TypeContext<'a> {
    pub fn new(root_schema: &'a Value) -> Self {
        Self {
            root_schema,
            visiting: HashSet::new(),
            self_ref_name: None,
            has_recursion: false,
        }
    }

    pub fn with_self_ref_name(mut self, name: &str) -> Self {
        self.self_ref_name = Some(name.to_string());
        self
    }
}

/// Convert a JSON Schema to a TypeScript type string.
///
/// Handles: basic types, enum, const, oneOf/anyOf, nullable, `$ref`,
/// recursive schemas, `additionalProperties`, `patternProperties`,
/// and tuple schemas per spec §6.2.
///
/// Falls back to `unknown` with an inline comment for unsupported features.
/// (Deviation: §6.2 says "doc comment warning" but inline comments are placed
/// directly at the affected type for better locality.)
pub(crate) fn schema_to_ts(schema: &Value, ctx: &mut TypeContext<'_>) -> String {
    // Boolean schemas
    match schema {
        Value::Bool(true) => return "unknown".to_string(),
        Value::Bool(false) => return "never".to_string(),
        _ => {}
    }

    let Some(obj) = schema.as_object() else {
        return "unknown".to_string();
    };

    // $ref — resolve within the same schema document
    if let Some(ref_val) = obj.get("$ref").and_then(|v| v.as_str()) {
        return resolve_ref(ref_val, ctx);
    }

    // const — literal type
    if let Some(const_val) = obj.get("const") {
        return value_to_ts_literal(const_val);
    }

    // enum — union of literals
    if let Some(enum_vals) = obj.get("enum").and_then(|v| v.as_array()) {
        let variants: Vec<String> = enum_vals.iter().map(value_to_ts_literal).collect();
        return variants.join(" | ");
    }

    // oneOf — union
    if let Some(one_of) = obj.get("oneOf").and_then(|v| v.as_array()) {
        let variants: Vec<String> = one_of.iter().map(|s| schema_to_ts(s, ctx)).collect();
        return variants.join(" | ");
    }

    // anyOf — union
    if let Some(any_of) = obj.get("anyOf").and_then(|v| v.as_array()) {
        let variants: Vec<String> = any_of.iter().map(|s| schema_to_ts(s, ctx)).collect();
        return variants.join(" | ");
    }

    // allOf — intersection
    if let Some(all_of) = obj.get("allOf").and_then(|v| v.as_array()) {
        let variants: Vec<String> = all_of.iter().map(|s| schema_to_ts(s, ctx)).collect();
        return variants.join(" & ");
    }

    // nullable flag (JSON Schema draft-07)
    let nullable = obj
        .get("nullable")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let base_type = if let Some(type_val) = obj.get("type") {
        match type_val {
            Value::String(s) => type_string_to_ts(s.as_str(), obj, ctx),
            Value::Array(types) => {
                // type: ["string", "null"] — handle as union
                let non_null: Vec<&str> = types
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter(|s| *s != "null")
                    .collect();
                let has_null = types.iter().any(|v| v.as_str() == Some("null"));

                let ts_types: Vec<String> = non_null
                    .iter()
                    .map(|s| type_string_to_ts(s, obj, ctx))
                    .collect();

                let base = if ts_types.is_empty() {
                    "unknown".to_string()
                } else {
                    ts_types.join(" | ")
                };

                if has_null || nullable {
                    format!("{base} | null")
                } else {
                    base
                }
            }
            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    };

    if nullable && !base_type.contains("| null") {
        format!("{base_type} | null")
    } else {
        base_type
    }
}

/// Convert a single JSON Schema type string to its TypeScript equivalent.
fn type_string_to_ts(
    type_name: &str,
    schema_obj: &serde_json::Map<String, Value>,
    ctx: &mut TypeContext<'_>,
) -> String {
    match type_name {
        "string" => "string".to_string(),
        "number" | "integer" => "number".to_string(),
        "boolean" => "boolean".to_string(),
        "null" => "null".to_string(),
        "array" => array_to_ts(schema_obj, ctx),
        "object" => object_to_ts(schema_obj, ctx),
        _ => format!("unknown /* unsupported type: {type_name} */"),
    }
}

/// Convert an array schema to TypeScript.
fn array_to_ts(schema_obj: &serde_json::Map<String, Value>, ctx: &mut TypeContext<'_>) -> String {
    match schema_obj.get("items") {
        // Tuple: items is an array of schemas
        Some(Value::Array(items)) => {
            let elements: Vec<String> = items.iter().map(|item| schema_to_ts(item, ctx)).collect();
            format!("[{}]", elements.join(", "))
        }
        // Regular array: items is a single schema
        Some(items) => {
            let item_type = schema_to_ts(items, ctx);
            if item_type.contains(" | ") || item_type.contains(" & ") {
                format!("({item_type})[]")
            } else {
                format!("{item_type}[]")
            }
        }
        // No items specified
        None => "unknown[]".to_string(),
    }
}

/// Convert an object schema to TypeScript.
///
/// When both named properties and `additionalProperties`/`patternProperties` are
/// present, an intersection type is emitted (e.g. `{ name: string } & Record<string, number>`)
/// instead of an inline index signature, which would be invalid TS when the index
/// value type is narrower than the named property types.
fn object_to_ts(schema_obj: &serde_json::Map<String, Value>, ctx: &mut TypeContext<'_>) -> String {
    let properties = schema_obj.get("properties").and_then(|v| v.as_object());
    let additional = schema_obj.get("additionalProperties");
    let pattern = schema_obj
        .get("patternProperties")
        .and_then(|v| v.as_object());
    let required: HashSet<&str> = schema_obj
        .get("required")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let has_properties = properties.is_some_and(|p| !p.is_empty());
    let has_additional = matches!(additional, Some(v) if v != &Value::Bool(false));
    let has_pattern = pattern.is_some_and(|p| !p.is_empty());

    // Pure additionalProperties (no named properties)
    if !has_properties && has_additional && !has_pattern {
        return format!("Record<string, {}>", additional_value_type(additional, ctx));
    }

    // Pure patternProperties (no named properties)
    if !has_properties
        && !has_additional
        && has_pattern
        && let Some(patterns) = pattern
    {
        let value_type = pattern_values_type(patterns, ctx);
        return format!("Record<string, {value_type}>");
    }

    // No properties at all
    if !has_properties && !has_additional && !has_pattern {
        return "Record<string, unknown>".to_string();
    }

    // Build named properties object type
    let props_type = if has_properties {
        let mut parts = Vec::new();
        if let Some(props) = properties {
            let mut prop_names: Vec<&String> = props.keys().collect();
            prop_names.sort();

            for name in prop_names {
                if let Some(prop_schema) = props.get(name) {
                    let optional = if required.contains(name.as_str()) {
                        ""
                    } else {
                        "?"
                    };
                    let prop_type = schema_to_ts(prop_schema, ctx);
                    parts.push(format!("{name}{optional}: {prop_type}"));
                }
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(format!("{{ {} }}", parts.join("; ")))
        }
    } else {
        None
    };

    // Build index signature type for additional/pattern properties
    let index_type = if has_additional {
        Some(format!(
            "Record<string, {}>",
            additional_value_type(additional, ctx)
        ))
    } else if has_pattern {
        pattern.map(|patterns| {
            let value_type = pattern_values_type(patterns, ctx);
            format!("Record<string, {value_type}>")
        })
    } else {
        None
    };

    // Combine: intersection when both present, otherwise just the one we have
    match (props_type, index_type) {
        (Some(p), Some(i)) => format!("{p} & {i}"),
        (Some(p), None) => p,
        (None, Some(i)) => i,
        (None, None) => "Record<string, unknown>".to_string(),
    }
}

/// Derive the TypeScript value type for an `additionalProperties` schema.
fn additional_value_type(additional: Option<&Value>, ctx: &mut TypeContext<'_>) -> String {
    match additional {
        Some(Value::Bool(true)) | None => "unknown".to_string(),
        Some(schema) if schema.is_object() => schema_to_ts(schema, ctx),
        _ => "unknown".to_string(),
    }
}

/// Compute the union type of all `patternProperties` value schemas.
fn pattern_values_type(
    patterns: &serde_json::Map<String, Value>,
    ctx: &mut TypeContext<'_>,
) -> String {
    let value_types: Vec<String> = patterns.values().map(|s| schema_to_ts(s, ctx)).collect();
    if value_types.len() == 1 {
        value_types
            .into_iter()
            .next()
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        value_types.join(" | ")
    }
}

/// Resolve a `$ref` path within the root schema.
fn resolve_ref(ref_path: &str, ctx: &mut TypeContext<'_>) -> String {
    // Detect recursion
    if ctx.visiting.contains(ref_path) {
        ctx.has_recursion = true;
        return ctx
            .self_ref_name
            .clone()
            .unwrap_or_else(|| ref_to_type_name(ref_path));
    }

    // Self-reference to root schema
    if ref_path == "#" {
        ctx.visiting.insert(ref_path.to_string());
        let root = ctx.root_schema.clone();
        let result = schema_to_ts(&root, ctx);
        ctx.visiting.remove(ref_path);
        return result;
    }

    // JSON Pointer: #/path/to/schema
    if let Some(pointer) = ref_path.strip_prefix('#') {
        let resolved = resolve_json_pointer(ctx.root_schema, pointer).cloned();
        match resolved {
            Some(schema) => {
                ctx.visiting.insert(ref_path.to_string());
                let result = schema_to_ts(&schema, ctx);
                ctx.visiting.remove(ref_path);
                result
            }
            None => format!("unknown /* unresolved $ref: {ref_path} */"),
        }
    } else {
        format!("unknown /* external $ref not supported: {ref_path} */")
    }
}

/// Resolve a JSON Pointer path within a JSON value.
fn resolve_json_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        return Some(root);
    }

    let path = pointer.strip_prefix('/').unwrap_or(pointer);
    let mut current = root;

    for segment in path.split('/') {
        // cspell:ignore rsplit
        let unescaped = segment.replace("~1", "/").replace("~0", "~");
        match current {
            Value::Object(obj) => {
                current = obj.get(&unescaped)?;
            }
            Value::Array(arr) => {
                let idx: usize = unescaped.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// Extract a type name from a `$ref` path for use in recursive references.
// cspell:ignore rsplit
fn ref_to_type_name(ref_path: &str) -> String {
    if ref_path == "#" {
        return "Self".to_string();
    }
    ref_path.rsplit('/').next().unwrap_or("Unknown").to_string()
}

/// Convert a JSON value to a TypeScript literal type string.
fn value_to_ts_literal(val: &Value) -> String {
    match val {
        Value::String(s) => format!("\"{}\"", escape_ts_string(s)),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => "unknown".to_string(),
    }
}

/// Escape a string for embedding inside a TypeScript string literal (double-quoted).
fn escape_ts_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

/// Generate a `JSDoc` comment block from a description and optional annotations (§6.3).
pub(crate) fn generate_doc_comment(
    description: Option<&str>,
    annotations: Option<&Value>,
    indent: &str,
) -> String {
    let mut lines = Vec::new();

    if let Some(desc) = description {
        for line in desc.lines() {
            lines.push(format!("{indent} * {line}"));
        }
    }

    if let Some(ann) = annotations.and_then(|a| a.as_object()) {
        if !lines.is_empty() {
            lines.push(format!("{indent} *"));
        }

        let mut keys: Vec<&String> = ann.keys().collect();
        keys.sort();

        for key in keys {
            if let Some(val) = ann.get(key) {
                let val_str = match val {
                    Value::Bool(b) => b.to_string(),
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                lines.push(format!("{indent} * @{key} {val_str}"));
            }
        }
    }

    if lines.is_empty() {
        return String::new();
    }

    format!("{indent}/**\n{}\n{indent} */\n", lines.join("\n"))
}

/// Convert a tool export name to `PascalCase` for use as a named interface.
///
/// If the result starts with a digit, it is prefixed with `_` to produce
/// a valid TypeScript identifier.
pub(crate) fn to_pascal_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    // Prefix with `_` if result starts with a digit (invalid TS identifier)
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert(0, '_');
    }

    result
}

/// Convert a schema to TypeScript, returning the type string and whether recursion was detected.
///
/// Shared helper to avoid duplicating the conversion + recursion-check logic.
pub(crate) fn convert_schema(schema: &Value, self_ref_name: &str) -> (String, bool) {
    let root = schema.clone();
    let mut ctx = TypeContext::new(&root).with_self_ref_name(self_ref_name);
    let ts = schema_to_ts(schema, &mut ctx);
    (ts, ctx.has_recursion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn convert(schema: &Value) -> String {
        let root = schema.clone();
        let mut ctx = TypeContext::new(&root);
        schema_to_ts(schema, &mut ctx)
    }

    // Basic types
    #[test]
    fn string_type() {
        assert_eq!(convert(&json!({"type": "string"})), "string");
    }

    #[test]
    fn number_type() {
        assert_eq!(convert(&json!({"type": "number"})), "number");
    }

    #[test]
    fn integer_type() {
        assert_eq!(convert(&json!({"type": "integer"})), "number");
    }

    #[test]
    fn boolean_type() {
        assert_eq!(convert(&json!({"type": "boolean"})), "boolean");
    }

    #[test]
    fn null_type() {
        assert_eq!(convert(&json!({"type": "null"})), "null");
    }

    #[test]
    fn boolean_schema_true() {
        assert_eq!(convert(&json!(true)), "unknown");
    }

    #[test]
    fn boolean_schema_false() {
        assert_eq!(convert(&json!(false)), "never");
    }

    // Enum and const
    #[test]
    fn string_enum() {
        assert_eq!(
            convert(&json!({"enum": ["a", "b", "c"]})),
            r#""a" | "b" | "c""#
        );
    }

    #[test]
    fn mixed_enum() {
        assert_eq!(
            convert(&json!({"enum": ["on", 1, true, null]})),
            r#""on" | 1 | true | null"#
        );
    }

    #[test]
    fn const_string() {
        assert_eq!(convert(&json!({"const": "fixed"})), r#""fixed""#);
    }

    #[test]
    fn const_number() {
        assert_eq!(convert(&json!({"const": 42})), "42");
    }

    // Nullable
    #[test]
    fn nullable_string() {
        assert_eq!(
            convert(&json!({"type": "string", "nullable": true})),
            "string | null"
        );
    }

    #[test]
    fn type_array_with_null() {
        assert_eq!(
            convert(&json!({"type": ["string", "null"]})),
            "string | null"
        );
    }

    #[test]
    fn type_array_multi() {
        assert_eq!(
            convert(&json!({"type": ["string", "number"]})),
            "string | number"
        );
    }

    // oneOf / anyOf / allOf
    #[test]
    fn one_of() {
        assert_eq!(
            convert(&json!({"oneOf": [{"type": "string"}, {"type": "number"}]})),
            "string | number"
        );
    }

    #[test]
    fn any_of() {
        assert_eq!(
            convert(&json!({"anyOf": [{"type": "string"}, {"type": "boolean"}]})),
            "string | boolean"
        );
    }

    #[test]
    fn all_of() {
        assert_eq!(
            convert(&json!({"allOf": [
                {"type": "object", "properties": {"a": {"type": "string"}}},
                {"type": "object", "properties": {"b": {"type": "number"}}}
            ]})),
            "{ a?: string } & { b?: number }"
        );
    }

    // Arrays
    #[test]
    fn array_of_strings() {
        assert_eq!(
            convert(&json!({"type": "array", "items": {"type": "string"}})),
            "string[]"
        );
    }

    #[test]
    fn array_no_items() {
        assert_eq!(convert(&json!({"type": "array"})), "unknown[]");
    }

    #[test]
    fn array_union_items_wrapped() {
        assert_eq!(
            convert(
                &json!({"type": "array", "items": {"oneOf": [{"type": "string"}, {"type": "number"}]}})
            ),
            "(string | number)[]"
        );
    }

    #[test]
    fn tuple() {
        assert_eq!(
            convert(
                &json!({"type": "array", "items": [{"type": "string"}, {"type": "number"}, {"type": "boolean"}]})
            ),
            "[string, number, boolean]"
        );
    }

    // Objects
    #[test]
    fn object_with_properties() {
        assert_eq!(
            convert(&json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "number"}
                },
                "required": ["name"]
            })),
            "{ age?: number; name: string }"
        );
    }

    #[test]
    fn object_empty() {
        assert_eq!(
            convert(&json!({"type": "object"})),
            "Record<string, unknown>"
        );
    }

    #[test]
    fn additional_properties_typed() {
        assert_eq!(
            convert(&json!({"type": "object", "additionalProperties": {"type": "string"}})),
            "Record<string, string>"
        );
    }

    #[test]
    fn additional_properties_true() {
        assert_eq!(
            convert(&json!({"type": "object", "additionalProperties": true})),
            "Record<string, unknown>"
        );
    }

    #[test]
    fn pattern_properties() {
        assert_eq!(
            convert(&json!({"type": "object", "patternProperties": {"^x-": {"type": "string"}}})),
            "Record<string, string>"
        );
    }

    #[test]
    fn properties_with_additional_uses_intersection() {
        let result = convert(&json!({
            "type": "object",
            "properties": {"name": {"type": "string"}},
            "additionalProperties": {"type": "number"},
            "required": ["name"]
        }));
        // Must use intersection, not inline index signature
        assert_eq!(result, "{ name: string } & Record<string, number>");
    }

    #[test]
    fn properties_with_pattern_uses_intersection() {
        let result = convert(&json!({
            "type": "object",
            "properties": {"id": {"type": "number"}},
            "patternProperties": {"^x-": {"type": "string"}},
            "required": ["id"]
        }));
        assert_eq!(result, "{ id: number } & Record<string, string>");
    }

    // $ref
    #[test]
    fn ref_to_defs() {
        let schema = json!({
            "type": "object",
            "properties": {
                "addr": {"$ref": "#/$defs/Address"}
            },
            "$defs": {
                "Address": {
                    "type": "object",
                    "properties": {
                        "street": {"type": "string"}
                    },
                    "required": ["street"]
                }
            }
        });
        let root = schema.clone();
        let mut ctx = TypeContext::new(&root);
        let result = schema_to_ts(&schema, &mut ctx);
        assert!(result.contains("addr?: { street: string }"));
    }

    #[test]
    fn ref_to_definitions() {
        let schema = json!({
            "type": "object",
            "properties": {
                "item": {"$ref": "#/definitions/Item"}
            },
            "definitions": {
                "Item": {"type": "string"}
            }
        });
        let root = schema.clone();
        let mut ctx = TypeContext::new(&root);
        let result = schema_to_ts(&schema, &mut ctx);
        assert_eq!(result, "{ item?: string }");
    }

    #[test]
    fn ref_unresolved() {
        let result = convert(&json!({"$ref": "#/missing/path"}));
        assert!(result.contains("unknown"));
        assert!(result.contains("unresolved"));
    }

    #[test]
    fn ref_external_unsupported() {
        let result = convert(&json!({"$ref": "https://example.com/schema.json"}));
        assert!(result.contains("unknown"));
        assert!(result.contains("external"));
    }

    // Recursive schemas
    #[test]
    fn recursive_self_ref() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "children": {
                    "type": "array",
                    "items": {"$ref": "#"}
                }
            }
        });
        let root = schema.clone();
        let mut ctx = TypeContext::new(&root).with_self_ref_name("TreeNode");
        let result = schema_to_ts(&schema, &mut ctx);
        assert!(ctx.has_recursion);
        assert!(result.contains("TreeNode[]"));
        assert!(result.contains("name?: string"));
    }

    #[test]
    fn recursive_defs_ref() {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {"type": "string"},
                "next": {"$ref": "#/$defs/Node"}
            },
            "$defs": {
                "Node": {
                    "type": "object",
                    "properties": {
                        "value": {"type": "string"},
                        "next": {"$ref": "#/$defs/Node"}
                    }
                }
            }
        });
        let root = schema.clone();
        let mut ctx = TypeContext::new(&root);
        let result = schema_to_ts(&schema, &mut ctx);
        assert!(ctx.has_recursion);
        assert!(result.contains("Node"));
    }

    // Doc comments
    #[test]
    fn doc_comment_description_only() {
        let result = generate_doc_comment(Some("Read a file"), None, "  ");
        assert!(result.contains("/**"));
        assert!(result.contains(" * Read a file"));
        assert!(result.contains(" */"));
    }

    #[test]
    fn doc_comment_with_annotations() {
        let annotations = json!({"readOnlyHint": true, "destructiveHint": false});
        let result = generate_doc_comment(Some("Delete item"), Some(&annotations), "  ");
        assert!(result.contains(" * Delete item"));
        assert!(result.contains(" * @destructiveHint false"));
        assert!(result.contains(" * @readOnlyHint true"));
    }

    #[test]
    fn doc_comment_empty() {
        let result = generate_doc_comment(None, None, "  ");
        assert!(result.is_empty());
    }

    // PascalCase
    #[test]
    fn pascal_case_simple() {
        assert_eq!(to_pascal_case("readFile"), "ReadFile");
    }

    #[test]
    fn pascal_case_with_underscores() {
        assert_eq!(to_pascal_case("read_file"), "ReadFile");
    }

    #[test]
    fn pascal_case_with_hyphens() {
        assert_eq!(to_pascal_case("read-file"), "ReadFile");
    }

    #[test]
    fn pascal_case_leading_digit_gets_prefix() {
        assert_eq!(to_pascal_case("123tool"), "_123tool");
        assert_eq!(to_pascal_case("0"), "_0");
    }

    // String escaping
    #[test]
    fn string_literal_with_quotes() {
        assert_eq!(
            value_to_ts_literal(&json!("he said \"hello\"")),
            r#""he said \"hello\"""#
        );
    }

    #[test]
    fn string_literal_with_backslash() {
        assert_eq!(
            value_to_ts_literal(&json!("path\\to\\file")),
            r#""path\\to\\file""#
        );
    }

    #[test]
    fn string_literal_with_newline() {
        assert_eq!(
            value_to_ts_literal(&json!("line1\nline2")),
            r#""line1\nline2""#
        );
    }

    // Unknown fallback
    #[test]
    fn unsupported_type_falls_back() {
        let result = convert(&json!({"type": "custom"}));
        assert!(result.contains("unknown"));
    }

    #[test]
    fn empty_schema_is_unknown() {
        assert_eq!(convert(&json!({})), "unknown");
    }

    // convert_schema helper
    #[test]
    fn convert_schema_detects_recursion() {
        let schema = json!({
            "type": "object",
            "properties": {
                "child": {"$ref": "#"}
            }
        });
        let (ts, has_recursion) = convert_schema(&schema, "Node");
        assert!(has_recursion);
        assert!(ts.contains("Node"));
    }

    #[test]
    fn convert_schema_no_recursion() {
        let schema = json!({"type": "string"});
        let (ts, has_recursion) = convert_schema(&schema, "Unused");
        assert!(!has_recursion);
        assert_eq!(ts, "string");
    }
}
