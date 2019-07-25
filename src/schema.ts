/**
 * Generate JSON Schema files.
 */

import fs from 'fs-extra'
import globby from 'globby'
import yaml from 'js-yaml'
import path from 'path'
import cloneDeep from 'lodash.clonedeep'
import Schema from './schema.d'

const SCHEMA_SOURCE_DIR = path.join(__dirname, '..', 'schema')
const SCHEMA_DEST_DIR = path.join(__dirname, '..', 'built')

/**
 * Generate `built/*.schema.json` files from `schema/*.schema.yaml` files.
 */
export const build = async (): Promise<void> => {
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
          return [title, { ...schema, file }]
        }
      )
    )
  )

  // Process each of the schemas
  Array.from(schemas.values()).forEach(schema => processSchema(schemas, schema))

  // Write to destination
  await fs.ensureDir('built')
  await Promise.all(
    Array.from(schemas.entries()).map(async ([title, schema]) => {
      const destPath = path.join(SCHEMA_DEST_DIR, title + '.schema.json')
      await fs.writeJSON(destPath, schema, { spaces: 2 })
    })
  )
}

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()

/**
 * Process a schema object to implement inheritance and
 * add add derived properties.
 */
const processSchema = (schemas: Map<string, Schema>, schema: Schema): void => {
  const { $schema, $id, title, file, source, children, descendants } = schema

  // If it's already got a children and descendants, then it's been processed.
  if (children !== undefined && descendants !== undefined) return
  schema.children = []
  schema.descendants = []

  if (title === undefined)
    throw new Error(`Schema title is required in source file: ${file}`)

  if ($schema === undefined)
    schema.$schema = `http://json-schema.org/draft-07/schema#`

  if ($id === undefined)
    schema.$id = `https://stencila.github.com/schema/${title}.schema.json`

  if (source === undefined)
    schema.source = `https://github.com/stencila/schema/blob/master/schema/${file}`

  try {
    const parent = parentSchema(schemas, schema)
    let parentProperties: { [key: string]: Schema } = {}
    let parentRequired: string[] = []
    if (parent !== null) {
      // Ensure that the parent schema has been processed (to collect properties)
      processSchema(schemas, parent)
      if (parent.properties !== undefined) parentProperties = parent.properties
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
        parent.children === undefined ? [title] : [...parent.children, title].sort()

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
          ancestor.properties.type.enum = [ancestor.title, ...ancestor.descendants]
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
