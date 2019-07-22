/** `
 * Module for generating R language bindings.
 */

import fs from 'fs-extra'
import path from 'path'
import {
  read,
  types,
  props,
  Schema,
  unions
} from './bindings'

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()

/**
 * Generate `src/types.R` from schemas.
 */
async function build(): Promise<void> {
  const schemas = await read()

  const classesCode = types(schemas).map(classGenerator).join('\n')
  const unionsCode = unions(schemas).map(unionGenerator).join('\n')

  const code = `
${classesCode}

${unionsCode}
`

  await fs.writeFile(path.join(__dirname, '..', 'R', 'types.R'), code)
}

/**
 * Generate a function for a normal type.
 */
function classGenerator (schema: Schema): string {
  const { title, extends: parent, description, properties } = schema
  const { inherited, own, required, optional } = props(schema)

  let code = `${title} <- function (\n`
  code += [
    ...required.map(({ name }) => `  ${name}`),
    ...optional.map(({ name }) => `  ${name}`)
  ].join(',\n')
  code += `\n){\n`

  if (parent === undefined) {
    code += `  self <- list()\n`
  } else {
    code += `  self <- ${parent}(\n`
    code += inherited.map(({ name }) => `    ${name}=${name}`).join(',\n')
    code += '\n  )\n'
  }

  code += own
    .map(({ name, schema }) => {
      const type = schemaToType(schema)
      return `  if(!missing(${name})) setProp(self, "${name}", ${type}, ${name})`
    })
    .join('\n')

  code += `\n  class(self) <- c(class(self), "${title}")`
  code += `\n  self`

  code += `\n}\n\n`

  return code
}

/**
 * Generate a `Union` type.
 */
function unionGenerator (schema: Schema): string {
  const {title = '', description = title} = schema
  let code = `#\` ${description.replace('\n', '\n#` ')}\n`
  code += `${title} = ${schemaToType(schema)}\n\n`
  return code
}

/**
 * Convert a schema definition to a R class
 */
function schemaToType(schema: Schema): string {
  const { type, anyOf, allOf, $ref } = schema

  if ($ref !== undefined) return `"${$ref.replace('.schema.json', '')}"`
  if (anyOf !== undefined) return anyOfToType(anyOf)
  if (allOf !== undefined) return allOfToType(allOf)
  if (schema.enum !== undefined) return enumToType(schema.enum)

  if (type === 'null') return '"NULL"'
  if (type === 'boolean') return '"logical"'
  if (type === 'number') return '"numeric"'
  if (type === 'integer') return '"numeric"'
  if (type === 'string') return '"character"'
  if (type === 'array') return arrayToType(schema)
  if (type === 'object') return '"list"'

  throw new Error(`Unhandled schema: ${JSON.stringify(schema)}`)
}

/**
 * Convert a schema with the `anyOf` property to a `Union` type checker.
 */
function anyOfToType(anyOf: Schema[]): string {
  const types = anyOf
    .map(schema => schemaToType(schema))
    .reduce(
      (prev: string[], curr) => (prev.includes(curr) ? prev : [...prev, curr]),
      []
    )
  if (types.length === 0) return ''
  if (types.length === 1) return types[0]
  return `Union(${types.join(', ')})`
}

/**
 * Convert a schema with the `allOf` property to a Python type.
 */
function allOfToType(allOf: Schema[]): string {
  if (allOf.length === 1) return schemaToType(allOf[0])
  else return schemaToType(allOf[allOf.length - 1])
}

/**
 * Convert a schema with the `array` property to an `Array` type checker.
 */
function arrayToType(schema: Schema): string {
  const items = Array.isArray(schema.items)
    ? anyOfToType(schema.items)
    : schema.items !== undefined
    ? schemaToType(schema.items)
    : 'Any()'
  return `Array(${items})`
}

/**
 * Convert a schema with the `enum` property to an `Enum` type checker.
 */
function enumToType(enu: (string | number)[]): string {
  const values = enu
    .map(schema => {
      return JSON.stringify(schema)
    })
    .join(', ')
  return `"Enum"`
}
