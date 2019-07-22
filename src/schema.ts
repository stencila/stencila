/**
 * Generate JSON Schema files.
 */

import fs from 'fs-extra'
import globby from 'globby'
import yaml from 'js-yaml'
import path from 'path'
import Schema from './schema.d'
import { isConstructorDeclaration } from 'typescript';

const SCHEMA_SOURCE_DIR = path.join(__dirname, '..', 'schema')
const SCHEMA_DEST_DIR = path.join(__dirname, '..', 'built')

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()

/**
 * Generate `built/*.schema.json` files from `schema/*.schema.yaml` files.
 */
export async function build(): Promise<void> {
  // Asynchronously read all the schema definition YAML files into a map of objects
  const files = await globby('*.schema.yaml', {
    cwd: SCHEMA_SOURCE_DIR
  })
  const schemas = new Map<string, Schema>(
    await Promise.all(
      files.map(
        async (file: string): Promise<[string, Schema]> => {
          const filePath = path.join(SCHEMA_SOURCE_DIR, file)
          const schema = yaml.safeLoad(await fs.readFile(filePath, 'utf-8'))
          return [file, { ...schema, file }]
        }
      )
    )
  )

  // Process each of the schemas
  Array.from(schemas.values()).forEach(schema => processSchema(schemas, schema))

  // Write to destination
  await fs.ensureDir('built')
  await Promise.all(
    Array.from(schemas.entries()).map(async ([file, schema]) => {
      const destPath = path.join(
        SCHEMA_DEST_DIR,
        file.replace('.yaml', '.json')
      )
      await fs.writeJSON(destPath, schema, { spaces: 2 })
    })
  )
}

/**
 * Process a schema object to implement inheritance and
 * add add derived properties.
 */
function processSchema(schemas: Map<string, Schema>, schema: Schema): void {
  const { $schema, $id, title, file, source, children, descendants } = schema

  // If it's already got a children and descendants, then it's been processed.
  if (children !== undefined && descendants !== undefined) return
  schema.children = []
  schema.descendants = []

  // We need a `title`!
  if (title === undefined)
    throw new Error(`Schema title is required in source file: ${file}`)

  if ($schema === undefined)
    schema.$schema = `http://json-schema.org/draft-07/schema#`

  if ($id === undefined)
    schema.$id = `https://stencila.github.com/schema/${title}.schema.json`

  if (source === undefined)
    schema.source = `https://github.com/stencila/schema/blob/master/schema/${file}`

  try {
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
        // Add aliases for array properties (if not ye registered)
        if (property.type === 'array' && name.endsWith('s')) {
          const alias = name.slice(0, -1)
          if (property.aliases === undefined) property.aliases = []
          if (!property.aliases.includes(alias)) property.aliases.push(alias)
          propertyAliases[alias] = name
        }
      }
      if (Object.keys(propertyAliases).length > 0) {
        schema.propertyAliases = propertyAliases
      }

      if (schema.additionalProperties === undefined) {
        schema.additionalProperties = false
      }
    }

    // Apply `extends` keyword
    const parent = parentSchema(schemas, schema)
    if (parent !== null) {
      // Ensure that the base schema has been processed (to collect properties)
      processSchema(schemas, parent)

      // Extends properties and requireds
      schema.properties = {
        ...(parent.properties !== undefined ? parent.properties : {}),
        ...(schema.properties !== undefined ? schema.properties : {})
      }
      schema.required = [
        ...(parent.required !== undefined ? parent.required : []),
        ...(schema.required !== undefined ? schema.required : [])
      ]

      // Initialise the `type` property
      if (schema.properties.type !== undefined) {
        schema.properties.type = {
          ...schema.properties.type,
          enum: [title],
          default: title
        }
      }

      // Add to parent's children
      parent.children =
        parent.children === undefined ? [title] : [...parent.children, title]

      // Add to all ancestors' descendants and type enum
      let ancestor: Schema | null = parent
      while (ancestor !== null) {
        ancestor.descendants =
          ancestor.descendants === undefined
            ? [title]
            : [...ancestor.descendants, title]
        if (
          ancestor.properties !== undefined &&
          ancestor.properties.type !== undefined &&
          ancestor.properties.type.enum !== undefined
        ) {
          ancestor.properties.type.enum = [
            ...ancestor.properties.type.enum,
            title
          ]
        }
        ancestor = parentSchema(schemas, ancestor)
      }
    }

    // Replace any `$ref`s to YAML with a ref to the JSON generated in this function
    const walk = (node: Schema): void => {
      if (typeof node !== 'object') return
      for (const [key, child] of Object.entries(node)) {
        if (key === '$ref' && typeof child === 'string')
          node[key] = path.basename(child).replace('.yaml', '.json')
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
function parentSchema(
  schemas: Map<string, Schema>,
  schema: Schema
): Schema | null {
  if (schema.extends === undefined) return null

  const parent = schemas.get(schema.extends)
  if (parent === undefined)
    throw new Error(`Unknown schema used in "extends": "${schema.extends}"`)

  return parent
}
