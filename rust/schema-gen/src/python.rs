//! Generation of Python types from Stencila Schema

use std::path::PathBuf;

use topological_sort::TopologicalSort;

use common::{
    async_recursion::async_recursion,
    eyre::{bail, Context, Report, Result},
    futures::future::try_join_all,
    inflector::Inflector,
    itertools::Itertools,
    tokio::fs::write,
};

use crate::{
    schema::{Items, Schema, Type, Value},
    schemas::Schemas,
};

/// Header for types.py
const HEADER: &str = r#"# Generated file; do not edit. See the Rust `schema-gen` crate.
# We override the Literal `type` in each class so...
# pyright: reportIncompatibleVariableOverride=false
from __future__ import annotations

import sys
from dataclasses import dataclass, fields, is_dataclass
from typing import Literal, Union

if sys.version_info >= (3, 11):
    from enum import StrEnum
else:
    from strenum import StrEnum

# Primitive types
UnsignedInteger = int
Cord = str
Array = list
Primitive = Union[
    None,
    bool,
    int,
    UnsignedInteger,
    float,
    str,
    Array,
    "Object",
]

Object = dict[str, Primitive]


class _Base:
    """Provide a base class with a simplified repr that ignores None values."""

    def __repr__(self):
        if not is_dataclass(self):
            raise TypeError("_Base should only be used with dataclasses")

        field_names = [f.name for f in fields(self)]
        valid_fields = {
            name: getattr(self, name)
            for name in field_names
            if getattr(self, name) is not None
        }
        repr_str = (
            f"{self.__class__.__name__}("  # type: ignore
            + ", ".join([f"{key}={value!r}" for key, value in valid_fields.items()])
            + ")"
        )
        return repr_str
"#;

// This is for error checking. These are the primitives we currently expect in the schema and deal with manually above.
const EXPECTED_PRIMITIVES: [&str; 9] = [
    "Boolean",
    "UnsignedInteger",
    "Number",
    "Array",
    "Null",
    "Cord",
    "Object",
    "String",
    "Integer",
];

impl Schemas {
    /// Generate a Python module for each schema
    pub async fn python(&self) -> Result<()> {
        eprintln!("Generating Python types");

        // The top level destination
        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../python/stencila_types/src/stencila_types");
        let dest = dest
            .canonicalize()
            .context(format!("can not find directory `{}`", dest.display()))?;

        // There are four types of schema to deal with.
        // 1. Enumerate schemas that get turned into Python Enum types.
        // 2. Object schemas that get turned into Python classes.
        // 3. AnyOf schemas that get turned into Python Union types.
        // 4. Raw types or primitives.
        let (mut enums, mut classes, mut unions) = (Vec::new(), Vec::new(), Vec::new());
        for (name, schema) in self.schemas.iter() {
            if schema.extends.contains(&"Enumeration".to_string()) {
                enums.push(name.clone());
            } else if schema.any_of.is_some() {
                unions.push(name.clone());
            } else if schema.r#type.is_none() {
                classes.push(name.clone());
            } else if !EXPECTED_PRIMITIVES.contains(&&**name) {
                bail!("Unexpected primitive: {}", name);
            }
        }

        let mut sections: Vec<String> = vec![HEADER.to_string()];

        for name in enums.iter() {
            let schema = self.schemas.get(name).expect("Schema not found");
            sections.push(Self::python_enum(name, schema)?);
        }

        // The order of class definitions matters. Base classes must come first.
        let mut topo_sort = TopologicalSort::new();
        for name in classes.iter() {
            let schema = self.schemas.get(name).expect("Schema not found");
            for base in &schema.extends {
                topo_sort.add_dependency(name, base);
            }
        }

        let mut dep_order = Vec::new();
        loop {
            // This gives us the dependencies in reverse order.
            // So we will reverse them again below.
            let mut deps = topo_sort.pop_all();
            if deps.is_empty() {
                break;
            }
            deps.sort(); // Sort to maintain ordering between generations
            deps.reverse(); // Reverse to maintain alpha sort order in file.
            dep_order.append(&mut deps);
        }
        // Note: Reverse the order of the dependencies so that base classes come first.
        dep_order.reverse();

        for name in dep_order {
            let schema = self.schemas.get(&name).expect("Schema not found");
            sections.push(self.python_class(&name, schema).await?);
        }

        // Finally, do the unions. These reference all the types already generated.
        for name in unions.iter() {
            // We did this already.
            if name == "Primitive" {
                continue;
            }
            let schema = self.schemas.get(name).expect("Schema not found");
            sections.push(self.python_union(schema).await?);
        }

        // Create a module for each schema
        // let futures = schema_order.iter().map(|s| self.python_module(&types, s));
        // let v: Vec<_> = try_join_all(futures).await?.into_iter().collect();
        write(dest.join("types.py"), sections.join("\n\n")).await?;

        Ok(())
    }

    /// Generate a Python `enum`
    ///
    /// Generates a `EnumStr`.
    ///
    /// Returns a string of the generated class
    fn python_enum(name: &String, schema: &Schema) -> Result<String> {
        let Some(any_of) = &schema.any_of else {
            bail!("Enum Schema has no anyOf");
        };

        let description = if let Some(title) = &schema.title {
            schema
                .description
                .clone()
                .unwrap_or(title.clone())
                .trim_end_matches('\n')
                .replace('\n', "\n    ")
        } else {
            "".to_string()
        };

        let mut lines = Vec::new();
        for variant in any_of {
            let python_value = if let Some(v) = variant.r#const.as_ref() {
                Self::python_value(v)
            } else {
                bail!("Enum variant has no const value")
            };
            lines.push(format!("    {python_value} = \"{python_value}\""));
        }
        let variants = lines.join("\n");
        let class_def = format!(
            r#"class {name}(StrEnum):
    """
    {description}
    """

{variants}
"#
        );
        Ok(class_def)
    }

    /// Generate a Python `class` for an object schema with `properties`
    ///
    /// Generates a `dataclass`. This needs to have `kw_only` for init function
    /// due to the fact that some inherited fields are required.
    ///
    /// Attempts to make this work with Pydantic `dataclass` and `BaseModel`
    /// failed seemingly due to cyclic dependencies in types.
    ///
    /// Returns the generated `class` text.
    async fn python_class(&self, name: &String, schema: &Schema) -> Result<String> {
        // Add our custom base class to the extends list.
        let base = if name == "Entity" {
            "_Base".to_string()
        } else {
            schema.extends.clone().join(", ")
        };
        let mut fields = Vec::new();

        // Always add the `type` field as a literal
        fields.push(format!(r#"    type: Literal["{name}"] = "{name}""#));

        for (name, property) in schema.properties.iter() {
            let name = name.to_snake_case();

            // Skip the `type` field and anything inherited
            if name == "type" || property.is_inherited {
                continue;
            }

            // Determine Python type of the property
            let (mut field_type, is_array, ..) = Self::python_type(property).await?;

            // Is the property an array?
            if is_array {
                field_type = format!("list[{field_type}]");
            };

            // Is the property optional?
            if !property.is_required {
                field_type = format!("{field_type} | None");
            };

            let mut field = format!("{name}: {field_type}");

            // Does the property have a default or is optional?
            if let Some(default) = property.default.as_ref() {
                let default = Self::python_value(default);
                field.push_str(&format!(" = {default}"));
            } else if !property.is_required {
                field.push_str(" = None");
            };

            let description = property
                .description
                .clone()
                .unwrap_or(name)
                .trim_end_matches('\n')
                .replace('\n', " ");
            fields.push(format!(
                r#"    {field}
    """{description}""""#
            ));
        }
        let fields = fields.join("\n\n");

        let description = schema
            .description
            .as_ref()
            .unwrap_or(name)
            .trim_end_matches('\n')
            .replace('\n', "    ");

        let cls_def = format!(
            r#"
@dataclass(kw_only=True, repr=False)
class {name}({base}):
    """
    {description}
    """

{fields}
"#
        );

        Ok(cls_def)
    }

    /// Generate a Python type for a schema
    ///
    /// Returns the name of the type and whether:
    ///  - it is an array
    ///  - it is a type (rather than an enum variant)
    #[async_recursion]
    async fn python_type(schema: &Schema) -> Result<(String, bool, bool)> {
        use Type::*;

        // If the Stencila Schema type name corresponds to a Python
        // native type then return the name of the native type, otherwise
        // return the pascal cased name
        let maybe_native_type = |type_name: &str| match type_name.to_lowercase().as_str() {
            "null" => "None".to_string(),
            "boolean" => "bool".to_string(),
            "integer" => "int".to_string(),
            "number" => "float".to_string(),
            "string" => "str".to_string(),
            _ => type_name.to_pascal_case(),
        };

        let result = if let Some(r#type) = &schema.r#type {
            match r#type {
                Array => {
                    let items = match &schema.items {
                        Some(Items::Ref(inner)) => maybe_native_type(&inner.r#ref),
                        Some(Items::Type(inner)) => maybe_native_type(&inner.r#type.to_string()),
                        Some(Items::AnyOf(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.any_of.clone()),
                                ..Default::default()
                            };
                            Self::python_type(&schema).await?.0
                        }
                        Some(Items::List(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.clone()),
                                ..Default::default()
                            };
                            Self::python_type(&schema).await?.0
                        }
                        None => "Unhandled".to_string(),
                    };
                    (items, true, true)
                }
                _ => (maybe_native_type(&r#type.to_string()), false, true),
            }
        } else if let Some(r#ref) = &schema.r#ref {
            (maybe_native_type(r#ref), false, true)
        } else if schema.any_of.is_some() {
            let name = if let Some(name) = schema.title.clone() {
                name
            } else {
                let mut sub_names = Vec::new();
                for subs in schema.any_of.clone().unwrap().iter() {
                    let name = Self::python_type(subs).await?.0;
                    sub_names.push(name);
                }
                sub_names.join(" | ")
            };
            (name, false, true)
        } else if let Some(title) = &schema.title {
            (maybe_native_type(title), false, true)
        } else if let Some(r#const) = &schema.r#const {
            (Self::python_value(r#const), false, false)
        } else {
            ("Unhandled".to_string(), false, true)
        };

        Ok(result)
    }

    /// Generate a Python discriminated union `type` for an `anyOf` root schema or property schema
    ///
    /// Returns the Union section
    async fn python_union(&self, schema: &Schema) -> Result<String> {
        let Some(any_of) = &schema.any_of else {
            bail!("Schema has no anyOf");
        };

        let (alternatives, are_types): (Vec<_>, Vec<_>) =
            try_join_all(any_of.iter().map(|schema| async {
                let (typ, is_array, is_type) = Self::python_type(schema).await?;
                let typ = if is_array {
                    Self::python_array_of(&typ).await?
                } else {
                    typ
                };
                Ok::<_, Report>((typ, is_type))
            }))
            .await?
            .into_iter()
            .unzip();

        let name = schema.title.clone().unwrap_or_else(|| {
            alternatives
                .iter()
                .map(|name| name.to_pascal_case())
                .join("Or")
        });

        let description = if let Some(title) = &schema.title {
            schema
                .description
                .clone()
                .unwrap_or(title.clone())
                .trim_end_matches('\n')
                .replace('\n', "\n    ")
        } else {
            alternatives
                .iter()
                .map(|variant| format!("`{variant}`"))
                .join(" or ")
        };

        let alternatives = alternatives
            .into_iter()
            .zip(are_types.into_iter())
            .collect_vec();

        let code = if alternatives.iter().all(|(.., is_type)| *is_type) {
            let types = alternatives
                .iter()
                .map(|(variant, ..)| format!("    {variant},"))
                .join("\n");

            format!(
                r#"{name} = Union[
{types}
]
"""
{description}
"""
"#
            )
        } else {
            bail!("Union contains non-type alternatives")
        };
        Ok(code)
    }

    /// Generate a Python `type` for an "array of" type
    ///
    /// Returns the name of the generated type which will be the plural
    /// of the type of the array items.
    async fn python_array_of(item_type: &str) -> Result<String> {
        let name = item_type.to_plural();
        Ok(name)
    }

    /// Generate a Python representation of a JSON schema value
    ///
    /// Returns a literal to the type of value.
    fn python_value(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Boolean(value) => value.to_string(),
            Value::Integer(value) => value.to_string(),
            Value::Number(value) => value.to_string(),
            Value::String(value) => value.clone(),
            _ => "Unhandled value type".to_string(),
        }
    }
}
