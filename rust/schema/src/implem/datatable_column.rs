use crate::{
    ArrayValidator, BooleanValidator, DatatableColumn, IntegerValidator, Null, NumberValidator,
    Primitive, StringValidator, Validator, prelude::*,
};

impl DatatableColumn {
    /// Create a datatable column from string-like values with type inference
    ///
    /// Takes an iterator of string-like values, performs type inference to determine
    /// the most appropriate type, and creates a column with proper validator.
    pub fn from_strings<S: AsRef<str>>(name: String, values: Vec<S>) -> Self {
        let values: Vec<String> = values.into_iter().map(|s| s.as_ref().to_string()).collect();

        let mut has_integers = false;
        let mut has_floats = false;
        let mut has_booleans = false;
        let mut non_null_count = 0;

        // First pass: analyze types
        for value in &values {
            if value.trim().is_empty() {
                continue;
            }

            non_null_count += 1;

            if value.parse::<bool>().is_ok() && (value == "true" || value == "false") {
                has_booleans = true;
            } else if value.parse::<i64>().is_ok() {
                has_integers = true;
            } else if value.parse::<f64>().is_ok() {
                has_floats = true;
            }
        }

        // Determine the most appropriate type
        let (parsed_values, validator) = if non_null_count == 0 {
            // All null/empty values - default to string
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| {
                    if v.trim().is_empty() {
                        Primitive::Null(Null)
                    } else {
                        Primitive::String(v)
                    }
                })
                .collect();
            let mut validator = ArrayValidator::new();
            validator.items_validator =
                Some(Box::new(Validator::StringValidator(StringValidator::new())));
            (vals, validator)
        } else if has_booleans
            && non_null_count == values.iter().filter(|v| v.parse::<bool>().is_ok()).count()
        {
            // All non-null values are booleans
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| {
                    if v.trim().is_empty() {
                        Primitive::Null(Null)
                    } else {
                        match v.parse::<bool>() {
                            Ok(b) => Primitive::Boolean(b),
                            Err(_) => Primitive::String(v),
                        }
                    }
                })
                .collect();
            let mut validator = ArrayValidator::new();
            validator.items_validator = Some(Box::new(Validator::BooleanValidator(
                BooleanValidator::new(),
            )));
            (vals, validator)
        } else if has_floats {
            // Has floating point numbers
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| {
                    if v.trim().is_empty() {
                        Primitive::Null(Null)
                    } else {
                        match v.parse::<f64>() {
                            Ok(n) => Primitive::Number(n),
                            Err(_) => Primitive::String(v),
                        }
                    }
                })
                .collect();
            let mut validator = ArrayValidator::new();
            validator.items_validator =
                Some(Box::new(Validator::NumberValidator(NumberValidator::new())));
            (vals, validator)
        } else if has_integers {
            // Only integers
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| {
                    if v.trim().is_empty() {
                        Primitive::Null(Null)
                    } else {
                        match v.parse::<i64>() {
                            Ok(i) => Primitive::Integer(i),
                            Err(_) => Primitive::String(v),
                        }
                    }
                })
                .collect();
            let mut validator = ArrayValidator::new();
            validator.items_validator = Some(Box::new(Validator::IntegerValidator(
                IntegerValidator::new(),
            )));
            (vals, validator)
        } else {
            // Default to string
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| {
                    if v.trim().is_empty() {
                        Primitive::Null(Null)
                    } else {
                        Primitive::String(v)
                    }
                })
                .collect();
            let mut validator = ArrayValidator::new();
            validator.items_validator =
                Some(Box::new(Validator::StringValidator(StringValidator::new())));
            (vals, validator)
        };

        let mut column = DatatableColumn::new(name, parsed_values);
        column.validator = Some(validator);
        column
    }

    /// Create a datatable column from JSON values with type inference
    ///
    /// Analyzes the JSON values to determine the most appropriate type and creates
    /// a column with proper validator.
    pub fn from_json_values(name: String, values: Vec<serde_json::Value>) -> Self {
        let mut has_integers = false;
        let mut has_floats = false;
        let mut has_booleans = false;
        let mut non_null_count = 0;

        // First pass: analyze types directly from JSON values
        for value in &values {
            match value {
                serde_json::Value::Null => continue,
                serde_json::Value::Bool(_) => {
                    has_booleans = true;
                    non_null_count += 1;
                }
                serde_json::Value::Number(n) => {
                    non_null_count += 1;
                    if n.is_i64() || n.is_u64() {
                        has_integers = true;
                    } else {
                        has_floats = true;
                    }
                }
                serde_json::Value::String(_) => {
                    non_null_count += 1;
                    // Strings could contain parseable numbers or booleans, but we'll treat as strings
                    // for JSON data to preserve the original intent
                }
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    non_null_count += 1;
                    // Complex types will be serialized to strings
                }
            }
        }

        // Determine the most appropriate type and create primitives
        let (primitives, validator) = if non_null_count == 0 {
            // All null values
            let vals: Vec<Primitive> = values.into_iter().map(|_| Primitive::Null(Null)).collect();

            let mut validator = ArrayValidator::new();
            validator.items_validator =
                Some(Box::new(Validator::StringValidator(StringValidator::new())));

            (vals, validator)
        } else if has_booleans
            && !has_integers
            && !has_floats
            && non_null_count
                == values
                    .iter()
                    .filter(|v| matches!(v, serde_json::Value::Bool(_) | serde_json::Value::Null))
                    .count()
        {
            // Pure boolean column
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| match v {
                    serde_json::Value::Bool(b) => Primitive::Boolean(b),
                    _ => Primitive::Null(Null),
                })
                .collect();

            let mut validator = ArrayValidator::new();
            validator.items_validator = Some(Box::new(Validator::BooleanValidator(
                BooleanValidator::new(),
            )));

            (vals, validator)
        } else if has_floats && !has_booleans {
            // Has floating point numbers (integers will be promoted to floats)
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| match v {
                    serde_json::Value::Number(n) => {
                        if let Some(f) = n.as_f64() {
                            Primitive::Number(f)
                        } else {
                            Primitive::Null(Null)
                        }
                    }
                    serde_json::Value::Null => Primitive::Null(Null),
                    other => Primitive::String(serde_json::to_string(&other).unwrap_or_default()),
                })
                .collect();

            let mut validator = ArrayValidator::new();
            validator.items_validator =
                Some(Box::new(Validator::NumberValidator(NumberValidator::new())));

            (vals, validator)
        } else if has_integers && !has_floats && !has_booleans {
            // Pure integer column
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| match v {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Primitive::Integer(i)
                        } else {
                            Primitive::Null(Null)
                        }
                    }
                    serde_json::Value::Null => Primitive::Null(Null),
                    other => Primitive::String(serde_json::to_string(&other).unwrap_or_default()),
                })
                .collect();

            let mut validator = ArrayValidator::new();
            validator.items_validator = Some(Box::new(Validator::IntegerValidator(
                IntegerValidator::new(),
            )));

            (vals, validator)
        } else {
            // Mixed types or strings - default to string
            let vals: Vec<Primitive> = values
                .into_iter()
                .map(|v| match v {
                    serde_json::Value::Null => Primitive::Null(Null),
                    serde_json::Value::String(s) => Primitive::String(s),
                    other => Primitive::String(serde_json::to_string(&other).unwrap_or_default()),
                })
                .collect();

            let mut validator = ArrayValidator::new();
            validator.items_validator =
                Some(Box::new(Validator::StringValidator(StringValidator::new())));

            (vals, validator)
        };

        let mut column = DatatableColumn::new(name, primitives);
        column.validator = Some(validator);
        column
    }
}

impl DomCodec for DatatableColumn {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_id(&self.id)
            .push_attr("name", &self.name);

        // This does not encode the `values`` of the column since that is done,
        // row-by-row in `impl DomCodec` for the parent `Datatable`.

        // The <stencila-datatable-column> web component expect this to be a JSON object
        if let Some(validator) = &self.validator {
            let validator = serde_json::to_string(validator).unwrap_or_default();
            context.push_attr("validator", &validator);
        }

        // Put name in a <span> as well so it is visible in static view.
        context.enter_elem("span").push_text(&self.name).exit_elem();

        context.exit_node();
    }
}
