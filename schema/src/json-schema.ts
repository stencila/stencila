/**
 * Script to build JSON Schema files from YAML files
 */

import fs from 'fs'
import path from 'path'
import url from 'url'

import Ajv from 'ajv'
import clone from 'fast-clone'
import yaml from 'js-yaml'
import shell from 'shelljs'

import { JsonSchema } from './types'

const __dirname = path.dirname(url.fileURLToPath(import.meta.url))
const SOURCE_DIR = path.join(__dirname, '..', 'yaml')
const DEST_DIR = path.join(__dirname, '..', '..', 'json-schema')

const SOURCE_BASE_URL =
  'https://github.com/stencila/stencila/blob/main/schema/yaml'
const DEST_BASE_URL =
  'https://raw.githubusercontent.com/stencila/stencila/main/json-schema'

// Delete all files previously generated JSON schema files
shell.rm('-rf', path.join(DEST_DIR, '*.json'))

// Read all YAML source files as `JsonSchema` objects
const files = await Promise.all(
  shell.ls(path.join(SOURCE_DIR, '*.yaml')).map(async (file: string) => {
    const schema = yaml.load(
      await fs.promises.readFile(file, 'utf-8')
    ) as JsonSchema

    const title = schema.title

    if (title === undefined || typeof title !== 'string')
      throw new Error(`Schema title is required in source file: ${file}`)

    if (path.parse(file).name !== title)
      console.warn(`Schema title differs to file name: "${title}" in ${file}`)

    const id = `${DEST_BASE_URL}/${title}.schema.json`
    const source = `${SOURCE_BASE_URL}/${title}.yaml`

    return [
      title,
      {
        $schema: 'http://json-schema.org/draft-07/schema',
        $id: id,
        ...schema,
        source,
      },
    ]
  })
)

// Put schema objects into a look up object for cross referencing by `title`
const schemas: Record<string, JsonSchema> = Object.fromEntries(files)

// Check that schemas are valid
const ajv = new Ajv()
const allIds = {}
const allProperties = {}
for (const schema of Object.values(schemas)) {
  const { title, description, '@id': id, status, properties } = schema

  // Is valid JSON Schema v7
  if (ajv.validateSchema(schema) !== true) {
    throw Error(`${title} is not a valid JSON Schema:\n${ajv.errors}`)
  }

  // Has a valid description
  const maxDescriptionLength = 120
  if (description === undefined)
    throw Error(`${title} schema is missing description`)
  else if (description.length > maxDescriptionLength)
    throw Error(`${title}.description is too long`)

  // Has a valid status
  const validStatuses = ['stable', 'unstable', 'experimental', 'deprecated']
  if (status === undefined) throw Error(`${title} schema is missing status`)
  else if (!validStatuses.includes(status))
    throw Error(`${title}.status should be in ${validStatuses}`)

  // Struct schemas (those with `properties`) have...
  if (properties !== undefined) {
    // ...a valid `@id` associated with only one name
    if (id === undefined) throw Error(`${title} is missing @id`)
    else {
      if (!/^[a-z]+:/.test(id))
        throw Error(
          `@id "${id}" is not prefixed by a vocabulary e.g. "schema:"`
        )

      if (allIds[id] !== undefined && allIds[id] !== title)
        throw Error(
          `@id "${id}" is associated with more than one name "${allIds[id]}", "${title}"`
        )
      else allIds[id] = title
    }

    // ...have properties that are valid...
    for (const [name, property] of Object.entries(properties)) {
      // ... `@id` of the property is valid and associated with one name
      const id = property['@id']
      if (id === undefined) throw Error(`${title}.${name} is missing @id`)
      else {
        if (!/^[a-z]+:/.test(id))
          throw Error(
            `@id "${id}" is not prefixed by a vocabulary e.g. "schema:"`
          )
        if (allIds[id] !== undefined && allIds[id] !== name)
          throw Error(
            `@id "${id}" is associated with more than one name "${allIds[id]}", "${name}"`
          )
        else allIds[id] = name
      }

      // ... property name is associated with only one `@id`
      if (allProperties[name] !== undefined) {
        if (allProperties[name] !== id)
          throw Error(
            `Property "${name}" is associated with more than one @id "${id}", "${allProperties[name]}"`
          )
      } else if (id !== undefined) {
        allProperties[name] = id
      }

      // ... property has a description
      if (property.description === undefined)
        throw Error(`${title}.${name} is missing description`)
      else if (property.description.length > maxDescriptionLength)
        throw Error(`${title}.${name}.description is too long`)

      // ... property is not using ordered validation
      if (property.type === 'array' && Array.isArray(property.items)) {
        throw Error(
          `${title}.${name}.items is using ordered validation, use plain '$ref' or 'anyOf' instead?`
        )
      }
    }
  }

  // Check $refs are valid, noting that `*Types.schema.json` files
  // are generated after this check.
  const checkRefs = (node: JsonSchema): void => {
    if (typeof node !== 'object') return
    for (const [key, child] of Object.entries(node)) {
      if (
        key === '$ref' &&
        typeof child === 'string' &&
        !child.startsWith('#') &&
        !child.endsWith('Types') &&
        schemas[child] === undefined
      ) {
        throw Error(`${title} has a $ref to unknown type "${child}"`)
      }
      checkRefs(child as JsonSchema)
    }
  }
  checkRefs(schema)
}

// Get the parent schema, if any, of a schema
const parentSchema = (schema: JsonSchema): JsonSchema | null => {
  if (schema.extends === undefined) return null

  const parent = schemas[schema.extends]
  if (parent === undefined)
    throw new Error(`Unknown schema used in "extends": "${schema.extends}"`)

  return parent
}

// Process schemas to implement inheritance and add derived properties
function processSchema(schema: JsonSchema) {
  const { title, children, descendants } = schema

  // If it's already got a children and descendants, then it's been processed.
  if (children !== undefined && descendants !== undefined) return

  schema.children = []
  schema.descendants = []

  // Inherit from parent schema
  const parent = parentSchema(schema)
  let parentProperties: { [key: string]: JsonSchema } = {}
  let parentPropertyAliases: { [key: string]: string } = {}
  let parentRequired: string[] = []
  if (parent) {
    // Ensure that the parent schema has been processed itself (so it
    // has all its inherited properties)
    processSchema(parent)

    if (parent.properties !== undefined) parentProperties = parent.properties
    if (parent.propertyAliases !== undefined)
      parentPropertyAliases = parent.propertyAliases
    if (parent.required !== undefined) parentRequired = parent.required
  }

  // Process properties
  if (schema.properties !== undefined) {
    schema.type = 'object'

    const propertyAliases: { [key: string]: string } = {}
    for (const [name, property] of Object.entries(schema.properties)) {
      property.from = title
      // Mark if this is an array property
      const isArray =
        property.type === 'array' ||
        (property.allOf?.filter((item) => item.type === 'array').length ?? 0) >
          0
      if (isArray) property.isArray = true
      const isPlural = isArray && name.endsWith('s')
      if (isPlural) property.isPlural = true
      // Registered declared aliases
      if (property.aliases !== undefined) {
        for (const alias of property.aliases) propertyAliases[alias] = name
      }
      // Add aliases for array properties (if not yet registered)
      if (isPlural) {
        const alias = name.slice(0, -1)
        if (property.aliases === undefined) property.aliases = []
        if (!property.aliases.includes(alias)) property.aliases.push(alias)
        propertyAliases[alias] = name
      }
      // Is this an override of a property schema in parent?
      if (name in parentProperties) property.isOverride = true
    }

    if (Object.keys(propertyAliases).length > 0)
      schema.propertyAliases = propertyAliases

    if (schema.additionalProperties === undefined)
      schema.additionalProperties = false
  }

  // Apply `extends` keyword
  if (parent !== null && parent.title !== 'Enumeration') {
    // Extend `properties`
    schema.properties = {
      ...clone(parentProperties),
      ...(schema.properties ?? {}),
    }

    // Extend `definitions` (these may be required for inline $refs
    // in inherited properties to work)
    if (parent.definitions !== undefined) {
      schema.definitions = {
        ...clone(parent.definitions),
        ...(schema.definitions ?? {}),
      }
    }

    // Flag inherited, but newly required properties, as overrides
    for (const [name, property] of Object.entries(schema.properties)) {
      if (
        property.from !== title &&
        schema.required !== undefined &&
        schema.required.includes(name)
      )
        property.isOverride = true
    }

    // Having done that, now we can extend `required`
    schema.required = [
      ...parentRequired,
      ...(schema.required !== undefined ? schema.required : []),
    ]

    // Extend `propertyAliases`
    schema.propertyAliases = {
      ...parentPropertyAliases,
      ...(schema.propertyAliases !== undefined ? schema.propertyAliases : {}),
    }

    // Initialize the `type` property
    if (schema.properties.type !== undefined) {
      schema.properties.type = {
        ...schema.properties.type,
        enum: [title],
        default: title,
      }
    }

    // Add to parent's children
    parent.children =
      parent.children === undefined
        ? [title]
        : [...parent.children, title].sort()

    // Add to all ancestors' descendants and type enum
    let ancestor: JsonSchema | null = parent
    while (ancestor !== null) {
      ancestor.descendants =
        ancestor.descendants === undefined
          ? [title]
          : [...ancestor.descendants, title].sort()
      if (
        ancestor.title !== undefined &&
        ancestor.properties !== undefined &&
        ancestor.properties.type !== undefined &&
        ancestor.properties.type.enum !== undefined
      ) {
        ancestor.properties.type.enum = [
          ancestor.title,
          ...ancestor.descendants,
        ]
      }
      ancestor = parentSchema(ancestor)
    }
  }

  // Replace any `$ref`s that are not internal (i.e. starting with `#`)
  // with a ref to the JSON file generated by this function
  const walk = (node: JsonSchema): void => {
    if (typeof node !== 'object') return
    for (const [key, child] of Object.entries(node)) {
      if (
        key === '$ref' &&
        typeof child === 'string' &&
        !child.startsWith('#') &&
        !child.endsWith('.schema.json')
      )
        node[key] = child + '.schema.json'
      walk(child as JsonSchema)
    }
  }
  walk(schema)
}
for (const schema of Object.values(schemas)) {
  processSchema(schema)
}

// Add all entity and primitive types to the `Node` union schema.
//
// The order of the types is important as it influences the
// order of attempted de-serialization and coercion (in Rust and possibly other
// languages). Furthermore, some code may rely on the order. So test before
// changing.
const entitySchema = schemas['Entity']
const primitiveSchema = schemas['Primitive']
const nodeSchema = schemas['Node']
nodeSchema.anyOf = [
  {
    $ref: 'Entity.schema.json',
  },
  ...(entitySchema.descendants ?? []).map((descendant) => ({
    $ref: `${descendant}.schema.json`,
  })),
  ...(primitiveSchema.anyOf ?? []),
]
schemas['Node'] = nodeSchema

// Add `*Types` schemas to the map of schemas which
// are the union (`anyOf`) of any descendant types
for (const [title, schema] of Object.entries(schemas)) {
  const { descendants } = schema
  if (descendants !== undefined && descendants.length > 0) {
    const typesTitle = title + 'Types'
    schemas[typesTitle] = {
      $schema: 'http://json-schema.org/draft-07/schema#',
      $id: `${DEST_BASE_URL}/${typesTitle}.schema.json`,
      title: typesTitle,
      description: `All type schemas that are derived from ${title}`,
      anyOf: [title, ...descendants].map((descendant) => ({
        $ref: `${descendant}.schema.json`,
      })),
    }
  }
}

// Write schema
await Promise.all(
  Object.entries(schemas).map(async ([title, schema]) => {
    const file = path.join(DEST_DIR, `${title}.schema.json`)
    const json = JSON.stringify(schema, null, '  ')
    await fs.promises.writeFile(file, json)
  })
)
