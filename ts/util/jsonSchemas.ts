/**
 * Utility functions for runtime inspection of JSON Schemas
 * in this repository.
 *
 * As with the other utility modules it
 * avoids using non-builtin modules so that this package has
 * no production dependencies.
 */

import fs from 'fs'
import path from 'path'
import {promisify} from 'util'
import JsonSchema from '../jsonSchema'

// Lazily loaded set of JSON-Schema's
let SCHEMAS: Record<string, JsonSchema> = {}

/**
 * Get all Stencila Schema's JSON Schemas.
 */
export async function jsonSchemas(): Promise<typeof SCHEMAS> {
  if (Object.keys(SCHEMAS).length === 0) {
    const dir = path.join(
      __dirname,
      ...(__filename.endsWith('.ts') ? ['..', '..', 'public'] : [])
    )
    const files = await promisify(fs.readdir)(
      dir,
      'utf8'
    )
    const schemaFiles = files.filter(filename => filename.endsWith('.schema.json'))
    const promises = schemaFiles.map(async file => {
      const json = await promisify(fs.readFile)(path.join(dir, file), 'utf8')
      return JSON.parse(json) as JsonSchema
    })
    const schemas = await Promise.all(promises)
    SCHEMAS = schemas.reduce((prev: typeof SCHEMAS, schema) => {
      const { title } = schema
      return title === undefined ? prev : {...prev, [title]: schema}
    }, {})
  }
  return SCHEMAS
}

/**
 * Get all the names of all types in the Stencila Schema.
 */
export async function jsonSchemaTypes(): Promise<string[]> {
  const schemas = await jsonSchemas()
  return Object.keys(schemas)
}

/**
 * Get all the properties in all types in the Stencila Schema.
 */
export async function jsonSchemaProperties(): Promise<string[]> {
  const schemas = await jsonSchemas()
  const props = Object.values(schemas).reduce(
      (properties: string[], schema) => [
        ...properties, ...Object.keys(schema.properties ?? {})
      ],
      []
    )
  return Array.from(new Set(props)).sort()
}
