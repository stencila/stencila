/**
 * Check `schema/*.schema.yaml` files and generate `public/*.schema.json`
 * from them.
 */

import Ajv from 'ajv'
import betterAjvErrors from 'better-ajv-errors'
import fs from 'fs-extra'
import globby from 'globby'
import yaml from 'js-yaml'
import cloneDeep from 'lodash.clonedeep'
import path from 'path'
import log from './log'
import Schema from './schema-interface'
import del from 'del'

const SCHEMA_SOURCE_DIR = path.join(__dirname, '..', 'schema')
const SCHEMA_DEST_DIR = path.join(__dirname, '..', 'public')

/**
 * Get the Schema major version for use in generated URLs
 */
const VERSION_MAJOR = fs
  .readJSONSync(path.join(__dirname, '..', 'package.json'))
  .version.split('.')[0]

/**
 * The base URL for JSON Schema `$id`s.
 */
const ID_BASE_URL = `http://schema.stenci.la/v${VERSION_MAJOR}`

/**
 * The base URL for source files.
 */
const SOURCE_BASE_URL = `https://github.com/stencila/schema/blob/master`

// Create a validation function for JSON Schema for use in `checkSchema`
const ajv = new Ajv({ jsonPointers: true })
// eslint-disable-next-line @typescript-eslint/no-var-requires
const metaSchema = require('ajv/lib/refs/json-schema-draft-07.json')
const validateSchema = ajv.compile(metaSchema)

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()

/**
 * Generate `public/*.schema.json` files from `schema/*.schema.yaml` files.
 */
export async function build(cleanup = true): Promise<void> {
  // Clean up old files
  if (cleanup) await del('*.schema.json', { cwd: SCHEMA_DEST_DIR })

  // Asynchronously read all the schema definition YAML files into a map of objects
  const files = await globby('*.schema.yaml', { cwd: SCHEMA_SOURCE_DIR })
  const schemas = new Map<string, Schema>(
    await Promise.all(
      files.map(
        async (file: string): Promise<[string, Schema]> => {
          const schema = yaml.safeLoad(
            await fs.readFile(path.join(SCHEMA_SOURCE_DIR, file), 'utf-8')
          )
          const title = schema.title
          if (title === undefined)
            throw new Error(`Schema title is required in source file: ${file}`)
          if (file.split('.')[0] !== title)
            log.warn(`Schema title differs to filename: "${title}" in ${file}`)
          return [title, { ...schema, file }]
        }
      )
    )
  )
  const schemata = Array.from(schemas.values())

  // Check each schema is valid
  const types: string[] = []
  const properties: { [key: string]: string } = {}
  const ids: { [key: string]: string } = {}
  const fails = schemata
    .map(schema => checkSchema(schemas, schema, types, properties, ids))
    .reduce((fails, ok) => (!ok ? fails + 1 : fails), 0)
  if (fails > 0) {
    log.error(`Errors in ${fails} schemas, please see messages above`)
    // Exit with code 1 so that this fails on CI or elsewhere
    process.exit(1)
  }

  // Process each of the schemas
  schemata.forEach(schema => processSchema(schemas, schema))

  // Write to destination
  await fs.ensureDir('public')
  await Promise.all(
    Array.from(schemas.entries()).map(async ([title, schema]) => {
      const destPath = path.join(SCHEMA_DEST_DIR, title + '.schema.json')
      await fs.writeJSON(destPath, schema, { spaces: 2 })
    })
  )
}

/**
 * Read a generated schema file
 */
export const readSchema = async (type: string): Promise<Schema> => {
  return fs.readJSON(
    path.join(__dirname, '..', 'public', type + '.schema.json')
  )
}

/**
 * Check that a schema is valid, including that,
 *
 * - no duplicate `title`s
 * - is valid JSON Schema v7
 * - all type schemas (those with `properties`) have a `@id` and `description`
 * - all property schemas (those that define a property) have a `@id` and `description`
 * - each property name is associated with only one `@id`
 * - each `@id` is associated with only one property name or type `title`
 * - no duplicate `stencila:` `@ids` (case insensitive)
 * - that other schemas that are referred to in `extends` or `$ref` exist
 *
 * @param schemas A map of all the schemas
 * @param schema The schema being checked
 */
const checkSchema = (
  schemas: Map<string, Schema>,
  schema: Schema,
  allTypes: string[],
  allProperties: { [key: string]: string },
  allIds: { [key: string]: string }
): boolean => {
  let valid = true
  const { title, extends: extends_, description, properties } = schema

  log.debug(`Checking type schema "${title}".`)
  if (title === undefined) return true

  const error = (message: string): void => {
    log.error(message)
    valid = false
  }

  // No type with same title already
  if (allTypes.includes(title)) error(`Type ${title} already exists`)

  // Is a valid schema?
  if (validateSchema(schema) !== true) {
    const message = (betterAjvErrors(
      metaSchema,
      schema,
      validateSchema.errors,
      {
        format: 'cli',
        indent: 2
      }
    ) as unknown) as string
    error(`${title} is not a valid JSON Schema:\n${message}`)
  }

  const maxDescriptionLength = 120

  // All schemas should have a description
  if (description === undefined) error(`${title} is missing description`)
  else if (description.length > maxDescriptionLength)
    error(`${title}.description is too long`)

  // Type schemas have necessary properties and extends is valid
  if (properties !== undefined) {
    const id = schema['@id']
    if (id === undefined) error(`${title} is missing @id`)
    else {
      if (allIds[id] !== undefined && allIds[id] !== title)
        error(
          `@id "${id}" is associated with more than one name "${allIds[id]}", "${title}"`
        )
      else allIds[id] = title
    }

    if (extends_ !== undefined) {
      if (!schemas.has(extends_))
        error(`${title}.extends refers to unknown type "${extends_}"`)
    }

    // Property schemas have necessary properties
    for (const [name, property] of Object.entries(properties)) {
      const id = property['@id']
      if (id === undefined) error(`${title}.${name} is missing @id`)
      else {
        if (allIds[id] !== undefined && allIds[id] !== name)
          error(
            `@id "${id}" is associated with more than one name "${allIds[id]}", "${name}"`
          )
        else allIds[id] = name
      }

      if (allProperties[name] !== undefined) {
        if (allProperties[name] !== id)
          error(
            `Property "${name}" is associated with more than one @id "${id}", "${allProperties[name]}"`
          )
      } else if (id !== undefined) {
        allProperties[name] = id
      }

      if (property.description === undefined)
        error(`${title}.${name} is missing description`)
      else if (property.description.length > maxDescriptionLength)
        error(`${title}.${name}.description is too long`)
    }
  }

  // Check $refs are valid
  const walk = (node: Schema): void => {
    if (typeof node !== 'object') return
    for (const [key, child] of Object.entries(node)) {
      if (key === '$ref' && typeof child === 'string' && !schemas.has(child)) {
        error(`${title} has a $ref to unknown type "${child}"`)
      }
      walk(child)
    }
  }
  walk(schema)

  return valid
}

/**
 * Process a schema object to implement inheritance and
 * add add derived properties.
 */
const processSchema = (schemas: Map<string, Schema>, schema: Schema): void => {
  const { $schema, $id, title, file, source, children, descendants } = schema
  log.debug(`Processing type schema "${title}".`)

  // If it's already got a children and descendants, then it's been processed.
  if (children !== undefined && descendants !== undefined) return
  schema.children = []
  schema.descendants = []

  if (title === undefined)
    throw new Error(`Schema title is required in source file: ${file}`)

  if ($schema === undefined)
    schema.$schema = `http://json-schema.org/draft-07/schema#`

  if ($id === undefined) schema.$id = `${ID_BASE_URL}/${title}.schema.json`

  if (source === undefined) schema.source = `${SOURCE_BASE_URL}/${file}`

  try {
    const parent = parentSchema(schemas, schema)
    let parentProperties: { [key: string]: Schema } = {}
    let parentPropertyAliases: { [key: string]: string } = {}
    let parentRequired: string[] = []
    if (parent !== null) {
      // Ensure that the parent schema has been processed (to collect properties)
      processSchema(schemas, parent)
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
        schema.properties[name].from = title
        // Registered declared aliases
        if (property.aliases !== undefined) {
          for (const alias of property.aliases) propertyAliases[alias] = name
        }
        // Add aliases for array properties (if not yet registered)
        if (property.type === 'array' && name.endsWith('s')) {
          const alias = name.slice(0, -1)
          if (property.aliases === undefined) property.aliases = []
          if (!property.aliases.includes(alias)) property.aliases.push(alias)
          propertyAliases[alias] = name
        }
        // Is this an override of a property schema in parent?
        if (name in parentProperties) property.override = true
      }

      if (Object.keys(propertyAliases).length > 0)
        schema.propertyAliases = propertyAliases

      if (schema.additionalProperties === undefined)
        schema.additionalProperties = false
    }

    // Apply `extends` keyword
    if (parent !== null) {
      // Extend `properties`
      schema.properties = {
        ...cloneDeep(parentProperties),
        ...(schema.properties !== undefined ? schema.properties : {})
      }

      // Flag inherited, but newly required properties, as overrides
      for (const [name, property] of Object.entries(schema.properties)) {
        if (
          property.from !== title &&
          schema.required !== undefined &&
          schema.required.includes(name)
        )
          property.override = true
      }

      // Having done that, now we can extend `required`
      schema.required = [
        ...parentRequired,
        ...(schema.required !== undefined ? schema.required : [])
      ]

      // Extend `propertyAliases`
      schema.propertyAliases = {
        ...parentPropertyAliases,
        ...(schema.propertyAliases !== undefined ? schema.propertyAliases : {})
      }

      // Initialize the `type` property
      if (schema.properties.type !== undefined) {
        schema.properties.type = {
          ...schema.properties.type,
          enum: [title],
          default: title
        }
      }

      // Add to parent's children
      parent.children =
        parent.children === undefined
          ? [title]
          : [...parent.children, title].sort()

      // Add to all ancestors' descendants and type enum
      let ancestor: Schema | null = parent
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
            ...ancestor.descendants
          ]
        }
        ancestor = parentSchema(schemas, ancestor)
      }
    }

    // Replace any `$ref`s to YAML with a ref to the JSON generated in this function
    const walk = (node: Schema): void => {
      if (typeof node !== 'object') return
      for (const [key, child] of Object.entries(node)) {
        if (
          key === '$ref' &&
          typeof child === 'string' &&
          !child.endsWith('.schema.json')
        )
          node[key] = child + '.schema.json'
        walk(child)
      }
    }
    walk(schema)
  } catch (error) {
    throw new Error(
      `Error when processing "${schema.source}": "${error.stack}"`
    )
  }
}

/**
 * Get the parent schema, if any, of a schema
 */
const parentSchema = (
  schemas: Map<string, Schema>,
  schema: Schema
): Schema | null => {
  if (schema.extends === undefined) return null

  const parent = schemas.get(schema.extends)
  if (parent === undefined)
    throw new Error(`Unknown schema used in "extends": "${schema.extends}"`)

  return parent
}
