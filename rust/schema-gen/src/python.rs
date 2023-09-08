//! Generation of Python types from Stencila Schema

use std::{
    collections::HashSet,
    fs::read_dir,
    path::{Path, PathBuf},
};

use common::{
    async_recursion::async_recursion,
    eyre::{bail, Context, Report, Result},
    futures::future::try_join_all,
    inflector::Inflector,
    itertools::Itertools,
    tokio::fs::{create_dir_all, remove_file, write},
};

use crate::{
    schema::{Items, Schema, Type, Value},
    schemas::Schemas,
};

/// Comment to place at top of a files to indicate it is generated
const GENERATED_COMMENT: &str = "# Generated file; do not edit. See the Rust `schema-gen` crate.";

/// Modules that should not be generated
///
/// These modules are manually written, usually because they are
/// an alias for a native JavasScript type.
const HANDWRITTEN_MODULES: &[&str] = &[
    "array.py",
    "cord.py",
    "object.py",
    "prelude.py",
    "unsigned_integer.py",
];

/// Native Python or Pydantic types which do not need to be imported
const NATIVE_TYPES: &[&str] = &["None", "bool", "int", "float", "str"];

/// Types which need to be declared as forward refs to circular imports
const FORWARD_REFS: &[&str] = &[
    "Comment",
    "Organization",
    "ImageObject",
    "SoftwareApplication",
    "Validator",
];

// Generate a valid module name
fn module_name(name: &str) -> String {
    let name = name.to_snake_case();
    match name.as_str() {
        "for" => "for_".to_string(),
        "if" => "if_".to_string(),
        _ => name,
    }
}

impl Schemas {
    /// Generate a Python module for each schema
    pub async fn python(&self) -> Result<()> {
        eprintln!("Generating Python types");

        // The top level destination
        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../python/stencila");
        let dest = dest
            .canonicalize()
            .context(format!("can not find directory `{}`", dest.display()))?;

        // The types directory that modules get generated into
        let types = dest.join("types");
        if types.exists() {
            // Already exists, so clean up existing files, except for those that are not generated
            for file in read_dir(&types)?
                .flatten()
                .filter(|entry| entry.path().is_file())
            {
                let path = file.path();

                if HANDWRITTEN_MODULES
                    .contains(&path.file_name().unwrap().to_string_lossy().as_ref())
                {
                    continue;
                }

                remove_file(&path).await?
            }
        } else {
            // Doesn't exist, so create it
            create_dir_all(&types).await?;
        }

        // Create a module for each schema
        let futures = self
            .schemas
            .values()
            .map(|schema| Self::python_module(&types, schema));
        try_join_all(futures).await?;

        // Create an __init__.py which export types from all modules (including those
        // that are not generated)
        let exports = read_dir(&types)
            .wrap_err(format!("unable to read directory `{}`", types.display()))?
            .flatten()
            .filter(|entry| entry.path().is_file())
            .map(|entry| {
                entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .strip_suffix(".py")
                    .unwrap()
                    .to_string()
            })
            .filter(|module| module != "prelude")
            .sorted()
            .map(|module| {
                format!(
                    "from .{module} import {name}",
                    name = module.to_pascal_case()
                )
            })
            .join("\n");
        write(
            types.join("__init__.py"),
            format!(
                r"{GENERATED_COMMENT}

{exports}
"
            ),
        )
        .await?;

        Ok(())
    }

    /// Generate a Python module for a schema
    async fn python_module(dest: &Path, schema: &Schema) -> Result<()> {
        let Some(title) = &schema.title else {
            bail!("Schema has no title");
        };

        if HANDWRITTEN_MODULES.contains(&title.as_str()) {
            return Ok(());
        }

        if schema.any_of.is_some() {
            Self::python_any_of(dest, schema).await?;
        } else if schema.r#type.is_none() {
            Self::python_object(dest, title, schema).await?;
        }

        Ok(())
    }

    /// Generate a Python type for a schema
    ///
    /// Returns the name of the type and whether:
    ///  - it is an array
    ///  - it is a type (rather than an enum variant)
    #[async_recursion]
    async fn python_type(dest: &Path, schema: &Schema) -> Result<(String, bool, bool)> {
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
                        Some(Items::Type(inner)) => maybe_native_type(inner.r#type.as_ref()),
                        Some(Items::AnyOf(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.any_of.clone()),
                                ..Default::default()
                            };
                            Self::python_type(dest, &schema).await?.0
                        }
                        Some(Items::List(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.clone()),
                                ..Default::default()
                            };
                            Self::python_type(dest, &schema).await?.0
                        }
                        None => "Unhandled".to_string(),
                    };
                    (items, true, true)
                }
                _ => (maybe_native_type(r#type.as_ref()), false, true),
            }
        } else if let Some(r#ref) = &schema.r#ref {
            (maybe_native_type(r#ref), false, true)
        } else if schema.any_of.is_some() {
            (Self::python_any_of(dest, schema).await?, false, true)
        } else if let Some(title) = &schema.title {
            (maybe_native_type(title), false, true)
        } else if let Some(r#const) = &schema.r#const {
            (Self::python_value(r#const), false, false)
        } else {
            ("Unhandled".to_string(), false, true)
        };

        Ok(result)
    }

    /// Generate a Python `class` for an object schema with `properties`
    ///
    /// Generates a `dataclass`. This needs to have `kw_only` for init function
    /// due to the fact that some inherited fields are required.
    ///
    /// Attempts to make this work with Pydantic `dataclass` and `BaseModel`
    /// failed seemingly due to cyclic dependencies in types.
    ///
    /// Returns the name of the generated `class`.
    async fn python_object(dest: &Path, title: &String, schema: &Schema) -> Result<String> {
        let path = dest.join(format!("{}.py", module_name(title)));
        if path.exists() {
            return Ok(title.to_string());
        }

        let mut used_types = HashSet::new();

        // Get the base class
        let base = match schema.extends.first().cloned() {
            Some(base) => {
                used_types.insert(base.clone());
                base
            }
            None => String::new(),
        };

        let mut fields = Vec::new();

        // Always add the `type` field as a literal
        fields.push(format!(
            r#"    type: Literal["{title}"] = field(default="{title}", init=False)"#
        ));

        for (name, property) in schema
            .properties
            .iter()
            .filter(|(.., property)| !property.is_inherited)
        {
            let name = name.to_snake_case();

            // Skip the `type` field which we force above
            if name == "type" {
                continue;
            }

            // Determine Python type of the property
            let (mut field_type, is_array, ..) = Self::python_type(dest, property).await?;
            used_types.insert(field_type.clone());

            // Is the property an array?
            if is_array {
                field_type = format!("List[{field_type}]");
            };

            // Is the property optional?
            if !property.is_required {
                field_type = format!("Optional[{field_type}]");
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

        let mut imports = used_types
            .into_iter()
            .filter(|used_type| !NATIVE_TYPES.contains(&used_type.as_str()))
            .sorted()
            .map(|used_type| {
                if FORWARD_REFS.contains(&used_type.as_str()) {
                    format!(r#"{used_type} = ForwardRef("{used_type}")"#,)
                } else {
                    format!(
                        "from .{module} import {used_type}",
                        module = used_type.to_snake_case()
                    )
                }
            })
            .join("\n");
        if !imports.is_empty() {
            imports.push_str("\n\n");
        }

        let description = schema
            .description
            .as_ref()
            .unwrap_or(title)
            .trim_end_matches('\n')
            .replace('\n', "    ");

        write(
            path,
            &format!(
                r#"{GENERATED_COMMENT}

from .prelude import *

{imports}
@dataclass(kw_only=True, frozen=True)
class {title}({base}):
    """
    {description}
    """

{fields}
"#
            ),
        )
        .await?;

        Ok(title.to_string())
    }

    /// Generate a Python discriminated union `type` for an `anyOf` root schema or property schema
    ///
    /// Returns the name of the generated enum.
    async fn python_any_of(dest: &Path, schema: &Schema) -> Result<String> {
        let Some(any_of) = &schema.any_of else {
            bail!("Schema has no anyOf");
        };

        let (alternatives, are_types): (Vec<_>, Vec<_>) =
            try_join_all(any_of.iter().map(|schema| async {
                let (typ, is_array, is_type) = Self::python_type(dest, schema).await?;
                let typ = if is_array {
                    Self::python_array_of(dest, &typ).await?
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

        let path = dest.join(format!("{}.py", name.to_snake_case()));
        if path.exists() {
            return Ok(name);
        }

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

        let mut imports = alternatives
            .iter()
            .filter(|(used_type, is_type)| *is_type && !NATIVE_TYPES.contains(&used_type.as_str()))
            .sorted()
            .map(|(used_type, ..)| format!(r#"{used_type} = ForwardRef("{used_type}")"#,))
            .join("\n");
        if !imports.is_empty() {
            imports.push_str("\n\n");
        }

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
            let variants = alternatives
                .iter()
                .map(|(variant, ..)| format!("    {variant} = \"{variant}\""))
                .join("\n");

            format!(
                r#"class {name}(StrEnum):
    """
    {description}
    """

{variants}
"#
            )
        };

        write(
            path,
            format!(
                r#"{GENERATED_COMMENT}

from .prelude import *

{imports}
{code}"#
            ),
        )
        .await?;

        Ok(name)
    }

    /// Generate a Python `type` for an "array of" type
    ///
    /// Returns the name of the generated type which will be the plural
    /// of the type of the array items.
    async fn python_array_of(dest: &Path, item_type: &str) -> Result<String> {
        let name = item_type.to_plural();

        let path = dest.join(format!("{}.py", name.to_snake_case()));
        if path.exists() {
            return Ok(name);
        }

        let python = format!(
            r#"{GENERATED_COMMENT}

from .prelude import *

from .{module} import {item_type}

{name} = List[{item_type}]
"#,
            module = module_name(item_type)
        );
        write(path, python).await?;

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
