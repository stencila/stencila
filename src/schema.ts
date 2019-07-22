/**
 *
 * Generate `built/*.schema.json` files from `schema/*.schema.yaml` files.
 */

import fs from 'fs-extra'
import globby from 'globby'
import yaml from 'js-yaml'
import path from 'path'
import Schema from './schema.d'

const SCHEMA_SOURCE_DIR = path.join(__dirname, '..', 'schema')
const SCHEMA_DEST_DIR = path.join(__dirname, '..', 'built')

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === undefined) build()

/**
 * Generate `built/*schema.json` files
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

  // Do final processing and write schema objects to file
  await fs.ensureDir('built')
  for (const [file, schema] of schemas.entries()) {
    if (schema.title === undefined)
      throw new Error(`Schema title is required in source file: ${file}`)

    // Generate the destination path from the source and then
    // rewrite source so that it can be use for a "Edit this schema" link in docs.
    const destPath = path.join(
      SCHEMA_DEST_DIR,
      path.basename(file).replace('.yaml', '.json')
    )

    // Create a final `properties.type.enum` (all `Thing`s should have this) based on `descendants`
    if (
      schema.$id !== undefined &&
      schema.$id.includes('stencila') &&
      schema.properties !== undefined &&
      schema.properties.type !== undefined
    ) {
      if (schema.descendants !== undefined) {
        schema.properties.type.enum = [
          schema.title,
          ...schema.descendants.sort()
        ]
      } else {
        schema.properties.type.enum = [schema.title]
      }
      schema.properties.type.default = schema.title
    }
    await fs.writeJSON(destPath, schema, { spaces: 2 })
  }

  // Copy the built JSON files into `dist` for publishing package
  await fs.ensureDir('dist')
  await Promise.all(
    (await globby(path.join('built', '*.schema.json'))).map(async (file: string) =>
      fs.copy(file, path.join('dist', file))
    )
  )
}

/**
 * Process a schema object to implement inheritance and
 * add add derived properties.
 */
function processSchema(
  schemas: Map<string, Schema>,
  schema: Schema
): void {
  const { $schema, $id, title, file, source, children } = schema

  // If it's already got a children property, then it's processed.
  if (children !== undefined) return

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
    schema.children = []
    schema.descendants = []

    if (schema.properties !== undefined) {
      schema.type = 'object'

      const typesAliases: { [key: string]: string } = {}
      for (const [name, property] of Object.entries(schema.properties)) {
        schema.properties[name].from = schema.title

        // Registered declared aliases
        if (property.aliases !== undefined) {
          for (const alias of property.aliases) typesAliases[alias] = name
        }
        // Add and register aliases for array properties
        if (property.type === 'array' && name.endsWith('s')) {
          const alias = name.slice(0, -1)
          if (property.aliases === undefined) property.aliases = []
          if (!property.aliases.includes(alias)) property.aliases.push(alias)
          typesAliases[alias] = name
        }
      }
      if (Object.keys(typesAliases).length > 0) {
        schema.propertyAliases = typesAliases
      }

      if (schema.additionalProperties === undefined) {
        schema.additionalProperties = false
      }
    }

    const base = parentSchema(schemas, schema)
    if (base !== null) {
      // Ensure that the base schema has been processed (to collect properties)
      processSchema(schemas, base)

      // Do extension of properties from base
      schema.properties = {
        ...(base.properties !== undefined ? base.properties : {}),
        ...(schema.properties !== undefined ? schema.properties : {})
      }
      schema.required = [
        ...(base.required !== undefined ? base.required : []),
        ...(schema.required !== undefined ? schema.required : [])
      ]

      // For all ancestors, add this schema to the descendants list
      if (base.children !== undefined) base.children.push(title)
      let ancestor: Schema | null = base
      while (ancestor !== null) {
        ancestor.descendants = ancestor.descendants === undefined ? [title] : []
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
