/**
 * A module providing functions to be used in languages bindings.
 */

import fs from 'fs-extra'
import globby from 'globby'
import path from 'path'
import toposort from 'toposort'
import * as schema from './schema'
import Schema from './schema.d'

export type Schema = Schema

/**
 * Read the schemas from `built/*.schema.json`.
 */
export async function read(
  glob: string = path.join(__dirname, '..', 'built', '*.schema.json')
): Promise<Schema[]> {
  // Ensure `*.schema.json` files are up to date
  await schema.build()

  // Read in the schemas
  const files = await globby(glob)
  return Promise.all(
    files.map(async (file: string): Promise<Schema> => fs.readJSON(file))
  )
}

/**
 * Generate code for 'normal' types (i.e. not union types) which are
 * usually translated into classes or similar for the language.
 *
 * Types are sorted topologically so that schemas come before
 * any of their descendants.
 */
export function types(schemas: Schema[]): Schema[] {
  const types = schemas.filter(schema => schema.anyOf === undefined)
  const map = new Map(schemas.map(schema => [schema.title, schema]))

  const edges = types.map(
    (schema): [string, string] => [
      schema.extends !== undefined ? schema.extends : '',
      schema.title !== undefined ? schema.title : ''
    ]
  )
  const ordered = toposort(edges).filter(title => title !== '')

  return ordered.map(title => {
    const schema = map.get(title)
    if (schema === undefined)
      throw new Error(`Holy smokes, "${title}" aint in da map @#!&??!`)
    return schema
  })
}

/**
 * Interface for properties giving a little
 * more information on each property to be used in code generation
 */
interface Property {
  name: string
  schema: Schema
  inherited: boolean
  optional: boolean
}

/**
 * Get properties for a schema.
 *
 * Properties are arranged in groups according to required (or not)
 * and inherited (or not).
 */
export function props(
  schema: Schema
): {
  all: Property[]
  inherited: Property[]
  own: Property[]
  required: Property[]
  optional: Property[]
} {
  const { title, properties = {}, required = [] } = schema

  const props = Object.entries(properties)
    .filter(([name, h]) => name !== 'type')
    .map(
      ([name, schema]): Property => {
        const { from } = schema
        const inherited = from !== title
        const optional = required === undefined || !required.includes(name)
        return { name, schema, inherited, optional }
      }
    )
    .sort((a, b) => {
      if (a.optional === b.optional) {
        if (a.name === b.name) return 0
        if (a.name < b.name) return -1
        return 1
      }
      if (a.optional) return 1
      return -1
    })

  return {
    all: props,
    inherited: props.filter(prop => prop.inherited),
    own: props.filter(prop => !prop.inherited || !prop.optional),
    required: props.filter(prop => !prop.optional),
    optional: props.filter(prop => prop.optional)
  }
}

export function unions(schemas: Schema[]): Schema[] {
  return schemas.filter(schema => schema.anyOf !== undefined)
}
