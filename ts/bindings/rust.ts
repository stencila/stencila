/** `
 * Module for generating Rust language bindings.
 */

/* eslint-disable @typescript-eslint/restrict-template-expressions */

import { pascalCase, snakeCase } from 'change-case'
import fs from 'fs-extra'
import path from 'path'
import { JsonSchema } from '../JsonSchema'
import {
  filterEnumSchemas,
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
  propertyName?: string
  typeName?: string
  propertyTypeName?: string
  anonEnums: Record<string, string>
}

// Custom attributes to add to particular properties
const propertyAttributes: Record<string, string[]> = {
  'Date.value': ['#[def = "chrono::Utc::now().to_rfc3339()"]'],
  'PropertyValue.value': [
    '#[def = "PropertyValueValue::String(String::new())"]',
  ],
}

// Custom types for particular properties
//
// These custom property types are most often used to ensure only
// as much memory is allocated as necessary, instead of using the
// defaults e.g. `i32` for integers
const propertyTypes: Record<string, string> = {
  // Avoid the multiple strings definition that gets automatically generated
  'Date.value': 'String',
  // Expect depths to be 1 to 6, this allows for 0 to 255
  'Heading.depth': 'u8',
  // Expect column and row spans to be non-zero
  'TableCell.colspan': 'u32',
  'TableCell.rowspan': 'u32',
  // Expect list items positions to be non-zero
  'ListItem.position': 'u32',
  // These validation related properties have a minimum of zero
  'ArrayValidator.min_items': 'u32',
  'ArrayValidator.max_items': 'u32',
  'StringValidator.min_length': 'u32',
  'StringValidator.max_length': 'u32',
}

// Types that should not get automatically boxed if the property is
// optional.
//
// In addition to these Vec types are not boxed.
// Reasons for not boxing:
//  - no space saving advantage because less than or same size as Box (8 bytes)
//  - ergonomics: properties that are optional but usually present
const noBoxTypes = [
  // Enums with no data
  'CiteCitationMode',
  'ClaimClaimType',
  'ListOrder',
  'NoteNoteType',
  'SoftwareSessionStatus',
  'TableCellCellType',
  'TableRowRowType',
  // Small primitives (8 bytes or less)
  'Boolean',
  'Integer',
  'Number',
  // Enums that have two alternative Vec variants
  // and so are no larger than a Vec anyway.
  'ListItemContent',
  'TableCellContent',
]

// Properties that need to use a `Box` pointer to prevent circular references
// (the "recursive type has infinite size" error) or because it is
// memory efficient (especially for optional properties on deeply nested structs)
const pointerProperties = [
  '*.isPartOf',
  'Organization.parentOrganization',
  'ImageObject.publisher', // recursive because publisher has `logo`
  'ImageObject.thumbnail',
  'ListItem.item',
  'Comment.parentItem',
  'ArrayValidator.contains',
  'ArrayValidator.itemsValidator',
  'ConstantValidator.value',
  'CodeExpression.output',
  'Parameter.default',
  'Parameter.value',
  'Variable.value',
]

// For types that extend `CreativeWork`, _and_ which are part of `InlineContent`
// or `BlockContent`, we generate a separate `XxxxSimple` struct that
// excludes all of the properties inherited from `CreativeWork` other than `content`

let creativeWorkTypes: string[]
let inlineContent: string[]
let blockContent: string[]

function initializeGlobals(schemas: JsonSchema[]): void {
  const entries = schemas.map((schema) => [schema.title, schema])
  const lookup = Object.fromEntries(entries) as Record<string, JsonSchema>
  const extract = (title: string): string[] => {
    return (lookup[title].anyOf ?? []).map((obj: JsonSchema) =>
      (obj.$ref ?? '').replace('.schema.json', '')
    )
  }
  creativeWorkTypes = extract('CreativeWorkTypes')
  inlineContent = extract('InlineContent')
  blockContent = extract('BlockContent')
}

function isCreativeWorkContent(title: string): boolean {
  return (
    creativeWorkTypes.includes(title) &&
    (inlineContent.includes(title) || blockContent.includes(title))
  )
}

/**
 * Generate `../../rust/types.rs` from schemas.
 */
async function build(): Promise<void> {
  const schemas = await readSchemas()
  initializeGlobals(schemas)

  const context = {
    anonEnums: {},
  }

  const structs = filterInterfaceSchemas(schemas)
    .map((schema) => {
      let structs = interfaceSchemaToStruct(schema, context)
      if (isCreativeWorkContent(schema.title ?? '')) {
        structs += interfaceSchemaToSimpleStruct(schema, context)
      }
      return structs
    })
    .join('\n')

  const enumEnums = filterEnumSchemas(schemas)
    .map((schema) => enumSchemaToEnum(schema, context))
    .join('\n')

  const unionEnums = filterUnionSchemas(schemas)
    .map((schema) => unionSchemaToEnum(schema, context))
    .join('\n')

  const code = `// Generated by rust.ts; do not edit

#![allow(clippy::large_enum_variant)]

use crate::{impl_enum, impl_struct};
use crate::prelude::*;

/*********************************************************************
 * Structs for "interface" schemas
 ********************************************************************/

${structs}

/*********************************************************************
 * Enums for struct properties which use JSON Schema 'enum' or 'anyOf'
 ********************************************************************/

${Object.values(context.anonEnums).join('\n')}

/*********************************************************************
 * Enums for "enum" schemas
 ********************************************************************/

${enumEnums}

/*********************************************************************
 * Enums for "union" schemas
 ********************************************************************/
  
${unionEnums}`

  await fs.writeFile(
    path.join(__dirname, '..', '..', 'rust', 'src', 'types.rs'),
    code
  )
}

/**
 * Generate a doc comments
 */
function docComment(description: string): string {
  return '/// ' + description.trim().replace(/[\n\r]+/g, ' ')
}

/**
 * Generate a Rust `struct` for an "interface" schema.
 *
 * Adds a `type_` property that is required for in de-serialization
 * to disambiguate among alternative types in an enum. This is
 * necessary because we can not use `#[serde(tag = "type")]` for enums
 * that involve primitive types. Although we could add that option
 * to each struct it does not help with disambiguation when it comes to
 * deserialization. See https://github.com/serde-rs/serde/issues/760.
 * This current solution is preferable to adding a `String` field
 * to each struct because it takes up less memory (single variant enum
 * takes up no space)
 */
export function interfaceSchemaToStruct(
  schema: JsonSchema,
  context: Context,
  typeName?: string
): string {
  const { title = 'Untitled', description = title } = schema
  const { all } = getSchemaProperties(schema)

  const fields = all
    .filter(({ name }) => name !== 'meta')
    .map(({ name, schema, optional, inherited, override }) => {
      const { description = name, from } = schema

      // Generate a type name for this property (to avoid duplication
      // use the name of the type that this property was defined on)
      context.propertyName = name
      context.typeName = inherited && !override ? from : title
      context.propertyTypeName = pascalCase(
        `${context.typeName} ${context.propertyName}`
      )

      const propertyPath = `${title}.${name}`

      const isPointer =
        pointerProperties.includes(propertyPath) ||
        pointerProperties.includes(`*.${name}`)

      let attrs = propertyAttributes[propertyPath] ?? []
      if (isPointer) attrs = [...attrs, '#[serde(skip)]']

      let type = propertyTypes[propertyPath]
      if (type === undefined) {
        type = schemaToType(schema, context)
        type =
          isPointer ||
          // Optional properties are boxed to reduce the size allocated to the stack
          // but not if they are the same size (or smaller than) a Box (8 bytes) or,
          // for ergonomic reasons, a Vec (24 bytes).
          (optional && !(noBoxTypes.includes(type) || type.startsWith('Vec')))
            ? `Box<${type}>`
            : type
      }
      type = optional ? `Option<${type}>` : type

      return `    ${docComment(description)}
${attrs.map((attr) => `    ${attr}\n`).join('')}    pub ${snakeCase(
        name
      )}: ${type},`
    })
    .join('\n\n')

  const derives = [
    'Clone',
    'Debug',
    'Defaults',
    'Serialize',
    'Deserialize',
  ].join(', ')

  return `
${docComment(description)}
#[skip_serializing_none]
#[derive(${derives})]
#[serde(default, rename_all = "camelCase")]
pub struct ${title} {
    /// The name of this type
    #[def = "${title}_::${typeName ?? title}"]
    pub type_: ${title}_,

${fields}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ${title}_ {
  ${typeName ?? title}
}

impl_struct!(${title});`
}

/**
 * Generate a Rust `struct` for a type derived from `CreativeWork`
 * for when it is part of content.
 */
export function interfaceSchemaToSimpleStruct(
  schema: JsonSchema,
  context: Context
): string {
  const { title, properties = {} } = schema
  const filteredProperties = Object.fromEntries(
    Object.entries(properties).reduce(
      (prev: [string, JsonSchema][], [name, property]) => {
        const keep =
          ['content', 'parts'].includes(name) ||
          !['Thing', 'CreativeWork'].includes(property.from ?? '')
        return keep ? [...prev, [name, property]] : prev
      },
      []
    )
  )
  const contentSchema: JsonSchema = {
    ...schema,
    title: `${title}Simple`,
    properties: filteredProperties,
  }
  return interfaceSchemaToStruct(contentSchema, context, title)
}

/**
 * Generate a Rust `enum` from a "enum" schema.
 */
export function enumSchemaToEnum(
  schema: JsonSchema,
  _context: Context
): string {
  const { title = '', description = title, anyOf } = schema

  const variants = anyOf
    ?.map((schema) => {
      const { description = '', const: const_ = '' } = schema
      return `    /// ${description}\n    ${const_ as string},\n`
    })
    .join('')

  return `${docComment(description)}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ${title} {\n${variants}}
impl_enum!(${title});`
}

/**
 * Generate a Rust `enum` from a "union" schema.
 *
 * Needs to use `serde(untagged)` because the union may include
 * primitive types such as `Number` and `String` which can not
 * be tagged. Tagging is done within structs.
 */
export function unionSchemaToEnum(
  schema: JsonSchema,
  context: Context
): string {
  const { title = '', description = title, anyOf } = schema

  const variants = anyOf
    ?.map((schema) => {
      const type = schemaToType(schema, context)
      if (type === 'Null') return `    Null,\n`
      if (
        (title === 'InlineContent' || title === 'BlockContent') &&
        isCreativeWorkContent(type)
      ) {
        return `    ${type}(${type}Simple),\n`
      }
      const name = type === 'Vec<Node>' ? 'Array' : type
      return `    ${name}(${type}),\n`
    })
    .join('')

  return `${docComment(description)}${
    // Can not use enum dispatch on enums that include `Null`
    !['Node', 'InlineContent'].includes(title)
      ? '\n#[enum_dispatch(NodeTrait)]'
      : ''
  }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ${title} {\n${variants}}\n`
}

/**
 * Convert a schema definition to a Rust type
 */
function schemaToType(schema: JsonSchema, context: Context): string {
  const { type, anyOf, allOf, $ref } = schema

  if ($ref !== undefined) return `${$ref.replace('.schema.json', '')}`
  if (anyOf !== undefined) return anyOfToEnum(anyOf, context)
  if (allOf !== undefined) return allOfToType(allOf, context)
  if (schema.enum !== undefined) return enumToEnum(schema.enum, context)

  if (type === 'null') return 'Null'
  if (type === 'boolean') return 'Boolean'
  if (type === 'integer') return 'Integer'
  if (type === 'number') return 'Number'
  if (type === 'string') return 'String'
  if (type === 'array') return arrayToType(schema, context)
  if (type === 'object') return 'Object'

  throw new Error(`Unhandled schema: ${JSON.stringify(schema)}`)
}

/**
 * Convert the `anyOf` property of a JSON schema to a Rust `enum`.
 *
 * Needs to use `serde(untagged)` because the property may allow for
 * primitive types such as `Number` and `String` which can not
 * be tagged. Tagging is done within structs.
 */
function anyOfToEnum(anyOf: JsonSchema[], context: Context): string {
  const variants = anyOf
    .map((schema) => {
      const type = schemaToType(schema, context)
      const name = type.replace('<', '').replace('>', '')
      return type === 'Null' ? name : `    ${name}(${type}),\n`
    })
    .join('')

  const name = context.propertyTypeName ?? ''
  const definition = `/// Types permitted for the \`${context.propertyName}\` property of a \`${context.typeName}\` node.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ${name} {\n${variants}}\n`
  context.anonEnums[name] = definition

  return name
}

/**
 * Convert the values of an `enum` property of a JSON schema to a Rust `enum`.
 */
export function enumToEnum(enu: (string | number)[], context: Context): string {
  const lines = enu
    .map((variant) => {
      variant = typeof variant === 'string' ? variant : `V${variant}`
      return `    ${variant},\n`
    })
    .join('')

  const name = context.propertyTypeName ?? ''
  const definition = `#[derive(Clone, Debug, Serialize, Deserialize)]
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
    ? anyOfToEnum(schema.items, context)
    : schema.items !== undefined
    ? schemaToType(schema.items, context)
    : '?'
  return `Vec<${items}>`
}
