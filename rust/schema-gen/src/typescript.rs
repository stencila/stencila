//! Generation of Typescript types from Stencila Schema

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
    tokio::fs::{self, create_dir_all, remove_file, write},
};

use crate::{
    schema::{Items, Schema, Type, Value},
    schemas::Schemas,
};

/// Comment to place at top of a files to indicate it is generated
const GENERATED_COMMENT: &str = "// Generated file; do not edit. See `../rust/schema-gen` crate.";

/// Modules that should not be generated
///
/// These modules are manually written, usually because they are
/// an alias for a native JavasScript type.
const NO_GENERATE_MODULE: &[&str] = &[
    "Array",
    "Boolean",
    "Cord",
    "Integer",
    "Null",
    "Number",
    "Object",
    "Primitive",
    "String",
    "UnsignedInteger",
];

const PRIMITIVES: &[&str] = &[
    "null",
    "boolean",
    "integer",
    "unsignedinteger",
    "number",
    "string",
    "cord",
    "array",
    "object",
];

/// Types for which native to TypesScript types are used directly
/// Note that this excludes `Integer`, `UnsignedInteger` and `Object`
/// which although they are implemented as native types have different semantics.
const NATIVE_TYPES: &[&str] = &["null", "boolean", "number", "string"];

impl Schemas {
    /// Generate a TypeScript module for each schema
    pub async fn typescript(&self) -> Result<()> {
        eprintln!("Generating TypeScript types");

        // The top level destination
        let dest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ts/src");
        let dest = dest
            .canonicalize()
            .context(format!("can not find directory `{}`", dest.display()))?;

        // The types directory that modules get generated into
        let types = dest.join("types");
        if types.exists() {
            // Already exists, so clean up existing files, except for those that are not generated
            for file in read_dir(&types)?.flatten() {
                let path = file.path();

                if NO_GENERATE_MODULE.contains(
                    &path
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .strip_suffix(".ts")
                        .unwrap(),
                ) {
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
            .map(|schema| self.typescript_module(&types, schema));
        try_join_all(futures).await?;

        // Collect all the types
        let types_list = read_dir(&types)
            .wrap_err(format!("unable to read directory `{}`", types.display()))?
            .flatten()
            .map(|entry| {
                entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .strip_suffix(".ts")
                    .unwrap()
                    .to_string()
            })
            .sorted()
            .collect_vec();

        // Create a types/index.ts which export types from all modules (including those
        // that are not generated)
        write(
            types.join("index.ts"),
            format!(
                r"{GENERATED_COMMENT}

{exports}
",
                exports = types_list
                    .iter()
                    .map(|module| format!("export * from \"./{module}.js\";"))
                    .join("\n")
            ),
        )
        .await?;

        // Populate the import and cases of the `hydrate` function
        let hydrate = dest.join("hydrate.ts");
        let mut content = fs::read_to_string(&hydrate).await?;

        const CASES_START: &str = "    // TYPE-CASES:START\n";
        const CASES_STOP: &str = "    // TYPE-CASES:STOP\n";
        let start = content.find(CASES_START).expect("should exist");
        let stop = content.rfind(CASES_STOP).expect("should exist");
        let cases = self.schemas.iter()
            .filter_map(|(name, schema)| (schema.any_of.is_none() && schema.r#type.is_none()).then_some(name))
            .sorted()
            .map(|name| {
                format!(
                    "    case \"{name}\":\n      return Object.setPrototypeOf(value, types.{name}.prototype);\n"
                )
            })
            .join("");
        content.replace_range(start.saturating_add(CASES_START.len())..stop, &cases);

        fs::write(hydrate, content).await?;

        Ok(())
    }

    /// Generate a TypeScript module for a schema
    async fn typescript_module(&self, dest: &Path, schema: &Schema) -> Result<()> {
        let Some(title) = &schema.title else {
            bail!("Schema has no title");
        };

        if NO_GENERATE_MODULE.contains(&title.as_str()) {
            return Ok(());
        }

        if schema.any_of.is_some() {
            Self::typescript_any_of(dest, schema).await?;
        } else if schema.r#type.is_none() {
            self.typescript_object(dest, title, schema).await?;
        }

        Ok(())
    }

    /// Generate a TypeScript type for a schema
    ///
    /// Returns the name of the type and whether:
    ///  - it is an array
    ///  - it is a type (rather than an enum variant)
    #[async_recursion]
    async fn typescript_type(dest: &Path, schema: &Schema) -> Result<(String, bool, bool)> {
        use Type::*;

        // If the Stencila Schema type name corresponds to a TypeScript
        // native type then return the name of the native type, otherwise
        // return the pascal cased name (e.g. `integer` -> `Integer`)
        let maybe_native_type = |type_name: &str| {
            let lower = type_name.to_lowercase();
            if NATIVE_TYPES.contains(&lower.as_str()) {
                lower
            } else {
                type_name.to_pascal_case()
            }
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
                            Self::typescript_type(dest, &schema).await?.0
                        }
                        Some(Items::List(inner)) => {
                            let schema = Schema {
                                any_of: Some(inner.clone()),
                                ..Default::default()
                            };
                            Self::typescript_type(dest, &schema).await?.0
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
            (Self::typescript_any_of(dest, schema).await?, false, true)
        } else if let Some(title) = &schema.title {
            (title.to_string(), false, true)
        } else if let Some(r#const) = &schema.r#const {
            (Self::typescript_value(r#const), false, false)
        } else {
            ("Unhandled".to_string(), false, true)
        };

        Ok(result)
    }

    /// Generate a TypeScript `class` for an object schema with `properties`
    ///
    /// Returns the name of the generated `class`.
    async fn typescript_object(
        &self,
        dest: &Path,
        title: &String,
        schema: &Schema,
    ) -> Result<String> {
        let path = dest.join(format!("{}.ts", title));
        if path.exists() {
            return Ok(title.to_string());
        }

        let mut used_types = HashSet::new();

        // Get the base class
        let base = schema.extends.first().map(|base| {
            used_types.insert(base.clone());
            base.clone()
        });

        let mut props = Vec::new();
        let mut required_props = Vec::new();
        let mut super_args = Vec::new();
        for (name, property) in schema.properties.iter() {
            let name = name.to_camel_case();

            if name == "type" {
                props.push(format!("  // @ts-expect-error 'not assignable to the same property in base type'\n  type: '{title}';"));
                continue;
            }

            let mut prop = name.clone();

            // Determine Typescript type of the property
            let (mut prop_type, is_array, ..) = Self::typescript_type(dest, property).await?;

            if !property.is_inherited || property.is_required {
                used_types.insert(prop_type.clone());
            }

            // Is the property optional?
            if !property.is_required {
                prop.push('?');
            }

            prop.push_str(": ");

            // Is the property an array?
            if is_array {
                prop_type.push_str("[]");
            };

            prop.push_str(&prop_type);

            // If the property is required, add it to the constructor args.
            if property.is_required {
                // An argument can not be named `arguments` so deal with that
                // special case here.
                required_props.push(if name == "arguments" {
                    (
                        name.clone(),
                        format!("this.{name} = args;"),
                        format!("args: {prop_type}, "),
                    )
                } else {
                    (
                        name.clone(),
                        format!("this.{name} = {name};"),
                        format!("{name}: {prop_type}, "),
                    )
                });

                if let Some(base) = &base {
                    if self
                        .schemas
                        .get(base)
                        .unwrap()
                        .properties
                        .get(&name)
                        .map(|prop| prop.is_required)
                        .unwrap_or(false)
                    {
                        super_args.push(name.clone())
                    }
                }
            }

            // Skip following for inherited props unless they are required on this
            // type but optional in the parent type
            if property.is_inherited
                && !(property.is_required
                    && self
                        .schemas
                        .get(&base.clone().expect("inherited so always base"))
                        .and_then(|parent| parent.properties.get(&name))
                        .map(|property| !property.is_required)
                        .unwrap_or(false))
            {
                continue;
            }

            // Does the property have a default?
            if let Some(default) = property.default.as_ref() {
                let default = Self::typescript_value(default);
                prop.push_str(&format!(" = {default}"));
            };

            let description = property
                .description
                .as_ref()
                .unwrap_or(&name)
                .trim_end_matches('\n')
                .replace('\n', "\n   * ");

            props.push(format!("  /**\n   * {description}\n   */\n  {prop};"));
        }
        let props = props.join("\n\n");
        let required_args = required_props.iter().map(|(.., arg)| arg).join("");
        let required_assignments = required_props
            .iter()
            .map(|(_, assignment, ..)| assignment)
            .join("\n    ");
        let super_args = super_args.join(", ");

        let class = if let Some(base) = base {
            format!(
                r#"export class {title} extends {base} {{
{props}

  constructor({required_args}options?: Partial<{title}>) {{
    super({super_args});
    this.type = "{title}";
    if (options) Object.assign(this, options);
    {required_assignments}
  }}
}}"#
            )
        } else {
            format!(
                r#"export class {title} {{
{props}

  constructor({required_args}options?: Partial<{title}>) {{
    if (options) Object.assign(this, options);
    {required_assignments}
  }}
}}"#
            )
        };

        let factory = format!(
            r#"/**
* Create a new `{title}`
*/
export function {name}({required_args}options?: Partial<{title}>): {title} {{
  return new {title}({args}options);
}}"#,
            name = match title.as_str() {
                "For" | "Function" | "Delete" | "If" => [&title.to_camel_case(), "_"].concat(),
                _ => title.to_camel_case(),
            },
            args = required_props
                .iter()
                .map(|(name, ..)| format!("{}, ", if name == "arguments" { "args" } else { name }))
                .join("")
        );

        let mut imports = used_types
            .into_iter()
            .filter(|used_type| {
                used_type != title && !NATIVE_TYPES.contains(&used_type.to_lowercase().as_str())
            })
            .sorted()
            .map(|used_type| format!("import {{ {used_type} }} from \"./{used_type}.js\";"))
            .join("\n");
        if !imports.is_empty() {
            imports.push_str("\n\n");
        }

        let description = schema
            .description
            .as_ref()
            .unwrap_or(title)
            .trim_end_matches('\n')
            .replace('\n', "\n * ");

        write(
            path,
            &format!(
                r#"{GENERATED_COMMENT}

{imports}/**
 * {description}
 */
{class}

{factory}
"#
            ),
        )
        .await?;

        Ok(title.to_string())
    }

    /// Generate a TypeScript discriminated union `type` for an `anyOf` root schema or property schema
    ///
    /// Returns the name of the generated enum.
    async fn typescript_any_of(dest: &Path, schema: &Schema) -> Result<String> {
        let Some(any_of) = &schema.any_of else {
            bail!("Schema has no anyOf");
        };

        let (alternatives, are_types): (Vec<_>, Vec<_>) =
            try_join_all(any_of.iter().map(|schema| async {
                let (typ, is_array, is_type) = Self::typescript_type(dest, schema).await?;
                let typ = if is_array {
                    Self::typescript_array_of(dest, &typ).await?
                } else {
                    typ
                };
                Ok::<_, Report>((typ, is_type))
            }))
            .await?
            .into_iter()
            .unzip();
        let all_are_types = are_types.iter().all(|item| *item);

        let name = schema.title.clone().unwrap_or_else(|| {
            alternatives
                .iter()
                .map(|name| name.to_pascal_case())
                .join("Or")
        });

        let path = dest.join(format!("{}.ts", name));
        if path.exists() {
            return Ok(name);
        }

        let description = if let Some(title) = &schema.title {
            schema
                .description
                .clone()
                .unwrap_or(title.clone())
                .trim_end_matches('\n')
                .replace('\n', "\n * ")
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
            .sorted()
            .filter_map(|(name, is_type)| {
                (*is_type && !NATIVE_TYPES.contains(&name.to_lowercase().as_str()))
                    .then_some(format!("import {{ type {name} }} from \"./{name}.js\";",))
            })
            .join("\n");
        if !imports.is_empty() {
            imports.push_str("\n\n");
        }

        let variants = alternatives
            .iter()
            .map(|(variant, is_type)| {
                if *is_type {
                    variant.clone()
                } else {
                    format!("'{variant}'")
                }
            })
            .join(" |\n  ");

        let is_union_type = all_are_types
        // A hack to avoid issues for the generated functions for these
        // Not necessary to have functions for these anyway.
        && ![
            "CreativeWorkTypeOrText",
            "IntegerOrString",
            "StringOrNumber",
            "ThingType",
        ]
        .contains(&name.as_str());

        let hydrate = if is_union_type {
            "
import { hydrate } from \"../hydrate.js\";

"
        } else {
            ""
        };

        let from = if is_union_type {
            format!(
                r#"/**
 * Create a `{name}` from an object
 */
export function {func_name}(other: {name}): {name} {{
  {primitives}switch(other.type) {{
    {cases}
      return hydrate(other) as {name}
    default:
      // @ts-ignore-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for {name}: ${{other.type}}`);
  }}
}}"#,
                func_name = name.to_camel_case(),
                primitives = if alternatives.iter().any(|(variant, ..)| {
                    variant == "Primitive" || PRIMITIVES.contains(&variant.to_lowercase().as_str())
                }) {
                    format!(
                        r#"if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {{
    return other as {name};
  }}
  "#
                    )
                } else {
                    String::new()
                },
                cases = alternatives
                    .iter()
                    .filter(|(variant, is_type)| {
                        *is_type && !PRIMITIVES.contains(&variant.to_lowercase().as_str())
                    })
                    .map(|(variant, ..)| format!("case \"{variant}\":"))
                    .join("\n    ")
            )
        } else {
            String::new()
        };

        write(
            path,
            format!(
                r#"{GENERATED_COMMENT}
{hydrate}{imports}/**
 * {description}
 */
export type {name} =
  {variants};

{from}
"#
            ),
        )
        .await?;

        Ok(name)
    }

    /// Generate a TypeScript `type` for an "array of" type
    ///
    /// Returns the name of the generated type which will be the plural
    /// of the type of the array items.
    async fn typescript_array_of(dest: &Path, item_type: &str) -> Result<String> {
        let name = item_type.to_plural();

        let path = dest.join(format!("{}.ts", name));
        if path.exists() {
            return Ok(name);
        }

        write(
            path,
            format!(
                r#"{GENERATED_COMMENT}
            
import {{ {item_type} }} from "./{item_type}.js";

export type {name} = {item_type}[];
"#
            ),
        )
        .await?;

        Ok(name)
    }

    /// Generate a TypeScript representation of a JSON schema value
    ///
    /// Returns a literal to the type of value.
    fn typescript_value(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Boolean(inner) => inner.to_string(),
            Value::Integer(inner) => inner.to_string(),
            Value::Number(inner) => inner.to_string(),
            Value::String(inner) => inner.to_string(),
            _ => "Unhandled value type".to_string(),
        }
    }
}
