/**
 * Generate Python language bindings.
 */

import crypto from 'crypto'
import path from 'path'
import fs from 'fs-extra'
import { read, types, props, Schema, unions } from './bindings'

/**
 * Run `build()` when this file is run as a Node script
 */
// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (module.parent === null) build()

/**
 * A list of global definitions required for enums
 */
let globals: string[] = []

/**
 * Generate `python/types.py` from schemas.
 */
async function build(): Promise<void> {
  const schemas = await read()

  globals = []
  const classesCode = types(schemas)
    .map(classGenerator)
    .join('')
  const unionsCode = unions(schemas)
    .map(unionGenerator)
    .join('')
  const globalsCode = globals.join('\n')

  const code = `
from typing import Any, Dict, List as Array, Optional, Union
from enum import Enum

${globalsCode}

${classesCode}

${unionsCode}
`

  await fs.writeFile(path.join(__dirname, '..', 'python', 'types.py'), code)
}

/**
 * Generate a `class`.
 */
function classGenerator(schema: Schema): string {
  const { title, extends: parent, description } = schema
  const { inherited, own, required, optional } = props(schema)

  const base = parent !== undefined ? '(' + parent + ')' : ''
  const clas = `class ${title}${base}:\n    """\n    ${description}\n    """\n\n`

  const attrs = own
    .map(({ name, schema, optional }) => {
      const type = schemaToType(schema)
      const attrType = optional ? `Optional[${type}]` : type
      return `    ${name}: ${attrType}`
    })
    .join('\n')

  const initPars =
    '\n' +
    [
      '        self',
      ...required.map(
        ({ name, schema }) => `        ${name}: ${schemaToType(schema)}`
      ),
      ...optional.map(
        ({ name, schema }) =>
          `        ${name}: Optional[${schemaToType(schema)}] = None`
      )
    ].join(',\n') +
    '\n    '

  const superArgs =
    '\n' +
    inherited.map(({ name }) => `            ${name}=${name}`).join(',\n') +
    '\n        '
  const superCall = `        super().__init__(${superArgs})`

  const initSetters = own
    .map(({ name }) => `        if ${name} is not None: self.${name} = ${name}`)
    .join('\n')

  const init = `    def __init__(${initPars}) -> None:\n${superCall}\n${initSetters}\n\n`

  return clas + (attrs.length > 0 ? attrs + '\n\n' : '') + init + '\n'
}

/**
 * Generate a `Union` type.
 */
function unionGenerator(schema: Schema): string {
  const { title, description } = schema
  let code = `"""\n${description}\n"""\n`
  code += `${title} = ${schemaToType(schema)}\n\n`
  return code
}

/**
 * Convert a schema definition to a Python type
 */
function schemaToType(schema: Schema): string {
  const { type, anyOf, allOf, $ref } = schema

  if ($ref !== undefined) return `"${$ref.replace('.schema.json', '')}"`
  if (anyOf !== undefined) return anyOfToType(anyOf)
  if (allOf !== undefined) return allOfToType(allOf)
  if (schema.enum !== undefined) return enumToType(schema.enum)

  if (type === 'null') return 'None'
  if (type === 'boolean') return 'bool'
  if (type === 'number') return 'float'
  if (type === 'integer') return 'int'
  if (type === 'string') return 'str'
  if (type === 'array') return arrayToType(schema)
  if (type === 'object') return 'Dict[str, Any]'

  throw new Error(`Unhandled schema: ${JSON.stringify(schema)}`)
}

/**
 * Convert a schema with the `anyOf` property to a Python `Union` type.
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
  return `Union[${types.join(', ')}]`
}

/**
 * Convert a schema with the `allOf` property to a Python type.
 *
 * If the `allOf` is singular then just use that (this usually arises
 * because the `allOf` is used for a property with a `$ref`). Otherwise,
 * use the last schema (this is usually because one or more codecs can be
 * used on a property and the last schema is the final, expected, type of
 * the property).
 */
function allOfToType(allOf: Schema[]): string {
  if (allOf.length === 1) return schemaToType(allOf[0])
  else return schemaToType(allOf[allOf.length - 1])
}

/**
 * Convert a schema with the `array` property to a Python `Array` type.
 */
function arrayToType(schema: Schema): string {
  const items = Array.isArray(schema.items)
    ? anyOfToType(schema.items)
    : schema.items !== undefined
    ? schemaToType(schema.items)
    : 'Any'
  return `Array[${items}]`
}

/**
 * Convert a schema with the `enum` property to a Python `Enum`.
 */
function enumToType(enu: (string | number)[]): string {
  const values = enu
    .map(schema => {
      return JSON.stringify(schema)
    })
    .join(', ')
  const signature = crypto
    .createHash('md5')
    .update(values)
    .digest('hex')

  const name = `Enum${signature}`
  const defn = `${name} = Enum("${signature}", [${values}])`

  if (!globals.includes(defn)) globals.push(defn)

  return `"${name}"`
}
