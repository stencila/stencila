/** `
 * Module for generating R language bindings.
 */

import { pascalCase, snakeCase } from 'change-case'
import fs from 'fs-extra'
import path from 'path'
import { JsonSchema } from '../JsonSchema'
import {
  filterInterfaceSchemas,
  filterUnionSchemas,
  getSchemaProperties,
  readSchemas,
} from '../util/helpers'

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (require.main) build()

// Code generation context
interface Context {
  propertyTypeName?: string
  anonEnums: Record<string, string>
}

// Manually defined types for properties of some types
const propertyTypes = {
  DateValue: 'chrono::DateTime::<chrono::Utc>',
}

// Properties that need to use a pointer to prevent circular references
// (the "recursive type has infinite size" error)
const pointerProperties = [
  '*.isPartOf',
  'Organization.parentOrganization',
  'ImageObject.publisher', // recursive because publisher has `logo`
  'ImageObject.thumbnail',
  'Comment.parentItem',
  'ArrayValidator.contains',
  'ArrayValidator.itemsValidator',
]

// Types to NOT derive from Default
// This usually have a property that is required that
// doe snot have a default itself
const skipDefaultTypes = ['Date', 'PropertyValue']

/**
 * Generate `../../rust/types.rs` from schemas.
 */
async function build(): Promise<void> {
  const schemas = await readSchemas()

  const context = {
    anonEnums: {},
  }

  const structs = filterInterfaceSchemas(schemas)
    .map((schema) => structGenerator(schema, context))
    .join('\n')

  const unions = filterUnionSchemas(schemas)
    .map((schema) => enumGenerator(schema, context))
    .join('\n')

  const code = `
#![allow(clippy::large_enum_variant)]

use std::sync::Arc;

type Null = serde_json::Value;
type Bool = bool;
type Integer = i32;
type Number = f32;
type Array = Vec<serde_json::Value>;
type Object = std::collections::HashMap<String, serde_json::Value>;

// Structs for each type

${structs}

// Types for properties that are manually defined

${Object.entries(propertyTypes).map(
  ([key, value]) => `type ${key} = ${value};\n`
)}

// Enums for properties which use JSON Schema 'enum' or 'anyOf'

${Object.values(context.anonEnums).join('\n')}

// Enums for "union" types
  
${unions}`

  await fs.writeFile(
    path.join(__dirname, '..', '..', 'rust', 'src', 'lib.rs'),
    code
  )
}

/**
 * Generate a struct for a normal type.
 */
export function structGenerator(schema: JsonSchema, context: Context): string {
  const { title = 'Untitled', description = title } = schema
  const { all } = getSchemaProperties(schema)

  const fields = all
    .map(({ name, schema, optional }) => {
      const { description = name } = schema

      const propertyTypeName = pascalCase(`${title} ${name}`)
      context.propertyTypeName = propertyTypeName

      let type =
        propertyTypeName in propertyTypes
          ? propertyTypeName
          : schemaToType(schema, context)

      type =
        pointerProperties.includes(`${title}.${name}`) ||
        pointerProperties.includes(`*.${name}`)
          ? `Arc<${type}>`
          : type

      type = optional ? `Option<${type}>` : type

      return `    ${docComment(description)}
    pub ${snakeCase(name)}: ${type},`
    })
    .join('\n\n')

  const derives = [
    'Debug',
    ...(skipDefaultTypes.includes(title) ? [] : ['Default']),
  ].join(', ')

  const code = `
/// ${title}
///
${docComment(description)}
#[derive(${derives})]
pub struct ${title} {
${fields}
}`

  return code
}

/**
 * Generate a doc comments
 */
function docComment(description: string): string {
  return '/// ' + description.trim().replace(/[\n\r]+/g, ' ')
}

/**
 * Convert a schema definition to a Rust type
 */
function schemaToType(schema: JsonSchema, context: Context): string {
  const { type, anyOf, allOf, $ref } = schema

  if ($ref !== undefined) return `${$ref.replace('.schema.json', '')}`
  if (anyOf !== undefined) return anyOfToType(anyOf, context)
  if (allOf !== undefined) return allOfToType(allOf, context)
  if (schema.enum !== undefined) return enumToType(schema.enum, context)

  if (type === 'null') return 'Null'
  if (type === 'boolean') return 'Bool'
  if (type === 'number') return 'Number'
  if (type === 'integer') return 'Integer'
  if (type === 'string') return 'String'
  if (type === 'array') return arrayToType(schema, context)
  if (type === 'object') return 'Object'

  throw new Error(`Unhandled schema: ${JSON.stringify(schema)}`)
}

/**
 * Convert the `anyOf` property of a JSON schema to a Rust `enum`.
 */
function anyOfToType(anyOf: JsonSchema[], context: Context): string {
  const name = anyOf
    .map((schema) =>
      schemaToType(schema, context).replace('<', '').replace('>', '')
    )
    .join('')

  const variants = anyOf
    .map((schema) => {
      const type = schemaToType(schema, context)
      const name = type.replace('<', '').replace('>', '')
      return `    ${name}(${type}),\n`
    })
    .join('')

  const definition = `#[derive(Debug)]
pub enum ${name} {\n${variants}}\n`
  context.anonEnums[name] = definition

  return name
}

/**
 * Convert the values of an `enum` property to a Rust `enum`.
 */
export function enumToType(enu: (string | number)[], context: Context): string {
  const lines = enu
    .map((variant) => {
      variant = typeof variant === 'string' ? variant : `V${variant}`
      return `    ${pascalCase(variant)},\n`
    })
    .join('')

  const name = context.propertyTypeName ?? ''
  const definition = `#[derive(Debug)]
pub enum ${name} {\n${lines}}\n`
  context.anonEnums[name] = definition

  return name
}

/**
 * Convert a schema with the `allOf` property to a type.
 */
function allOfToType(allOf: JsonSchema[], context: Context): string {
  if (allOf.length === 1) return schemaToType(allOf[0], context)
  else return schemaToType(allOf[allOf.length - 1], context)
}

/**
 * Convert a schema with the `array` property to an `Array` type checker.
 */
function arrayToType(schema: JsonSchema, context: Context): string {
  const items = Array.isArray(schema.items)
    ? anyOfToType(schema.items, context)
    : schema.items !== undefined
    ? schemaToType(schema.items, context)
    : 'ANY'
  return items === 'ANY' ? 'Array' : `Vec<${items}>`
}

/**
 * Generate an enum from a "union" type.
 */
export function enumGenerator(schema: JsonSchema, context: Context): string {
  const { title = '', description = title, anyOf } = schema

  const variants = anyOf
    ?.map((schema) => {
      const name = schemaToType(schema, context)
      return `    ${name}(${name}),\n`
    })
    .join('')

  return `#[derive(Debug)]
${docComment(description)}\n
pub enum ${title} {\n${variants}}\n`
}
