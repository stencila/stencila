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
import JsonSchema from '../jsonSchema'

// Lazily loaded set of JSON Schemas
let SCHEMAS: Record<string, JsonSchema> = {}

interface Property {
  /**
   * Name of the property
   */
  name: string

  /**
   * Id of the property
   */
  id: string

  /**
   * Types that this property exists on.
   *
   * @see https://schema.org/domainIncludes
   */
  domainIncludes: string[]

  /**
   * Is the property an array?
   */
  isArray: boolean

  /**
   * Is the property name pluralized (ends in `s`).
   */
  isPlural: boolean
}

// Lazily populated set properties across all schemas
let PROPERTIES: Record<string, Property> = {}

/**
 * Get all Stencila Schema's JSON Schemas.
 */
export async function jsonSchemas(): Promise<typeof SCHEMAS> {
  if (Object.keys(SCHEMAS).length === 0) {
    const dir = path.join(
      __dirname,
      ...(__filename.endsWith('.ts') ? ['..', '..', 'public'] : [])
    )
    const files = await new Promise<string[]>((resolve, reject) =>
      fs.readdir(dir, 'utf8', (error, files) =>
        error !== null ? reject(error) : resolve(files)
      )
    )
    const schemaFiles = files.filter((filename) =>
      filename.endsWith('.schema.json')
    )
    const promises = schemaFiles.map(async (file) => {
      const json = await new Promise<string>((resolve, reject) =>
        fs.readFile(path.join(dir, file), 'utf8', (error, content) =>
          error !== null ? reject(error) : resolve(content)
        )
      )
      return JSON.parse(json) as JsonSchema
    })
    const schemas = await Promise.all(promises)
    SCHEMAS = schemas.reduce((prev: typeof SCHEMAS, schema) => {
      const { title } = schema
      return title === undefined ? prev : { ...prev, [title]: schema }
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
 *
 * Returns a alphabetically sorted `Record` with information on
 * each property, including the types that it occurs on,
 * and whether or not it is an array property.
 */
export async function jsonSchemaProperties(): Promise<
  Record<string, Property>
> {
  if (Object.keys(PROPERTIES).length === 0) {
    const schemas = await jsonSchemas()
    // Accumulate properties across types
    const props = Object.values(schemas).reduce(
      (prev: typeof PROPERTIES, type) => {
        const { title = '', properties = {} } = type
        Object.entries(properties).forEach(([name, prop]) => {
          const { '@id': id = '', isArray = false, isPlural = false } = prop
          const existing = prev[name]
          if (existing === undefined) {
            prev[name] = {
              name,
              id,
              domainIncludes: [title],
              isArray,
              isPlural,
            }
          } else {
            // Check that there is consistency in the property
            // Most of these checks are done in `../schema.ts` when the
            // JSON Schema files are generated. These checks may be moved
            // to elsewhere.
            const message = `${title}.${name} differs to that on ${existing.domainIncludes}`
            if (id !== existing.id) throw new Error(`${message}: @id ${id}`)
            if (isArray !== existing.isArray)
              throw new Error(`${message}: isArray ${isArray}`)
            if (isPlural !== existing.isPlural)
              throw new Error(`${message}: isPlural ${isPlural}`)
            // Add the type...
            existing.domainIncludes.push(title)
          }
        })
        return prev
      },
      {}
    )
    // Sort by key
    PROPERTIES = Object.keys(props)
      .sort()
      .reduce((prev, key) => ({ ...prev, [key]: props[key] }), {})
  }
  return PROPERTIES
}
