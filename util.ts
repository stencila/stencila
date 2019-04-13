/**
 * Utility functions for use with Stencila Schema in Typescript and Javascript
 */
import Ajv from 'ajv'
import betterAjvErrors from 'better-ajv-errors'
import fs from 'fs-extra'
import globby from 'globby'
import path from 'path'
import * as stencila from './types'

/**
 * Create a node of a type
 * @param type The name of the type
 * @param initial Initial values for properties
 * @param validation What validation should done?
 */
export function create<Key extends keyof stencila.Types>(
  type: Key,
  initial: { [key: string]: any } = {},
  validation: 'none' | 'validate' | 'mutate' = 'validate'
): stencila.Types[Key] {
  let node = { type, ...initial }
  if (validation === 'validate') return validate(node, type)
  else if (validation === 'mutate') return mutate(node, type)
  else return node
}

/**
 * Get the type of a node
 * @param node The node to get the type for
 */
export function type(node: any): string {
  if (node === null) return 'null'
  let type = typeof node
  if (type === 'object') {
    if (Array.isArray(node)) return 'array'
    if (node.type) return node.type
  }
  return type
}

/**
 * Is a node of a particular type/s
 * @param node The node to check
 * @param types The type names to check against
 */
export function is(node: any, types: string | string[]): boolean {
  if (typeof types === 'string') return type(node) === types
  else return types.includes(type(node))
}

/**
 * Assert that a node is of a particular type/s
 * @param node The node to check
 * @param types The type names to check against
 */
export function assert(node: any, types: string | string[]): boolean {
  if (is(node, types)) return true
  else {
    const list = typeof types === 'string' ? types : types.join('|')
    throw new Error(`Node type is "${type(node)}" but expected "${list}"`)
  }
}

/**
 * Cast a node to a particular type
 *
 * The node is validated against the type.
 * This means that an error will be throw if during:
 *   - up-casting the node does not have properties
 *     that are required by the schema of the new type
 *   - down-casting the node has properties that
 *     are additional to those in the schema of the new type
 * Use `mutate` if you want to ignore such errors
 * and force mutating the node to the type.
 *
 * @param node The node to cast
 * @param type The type to cast to
 */
export function cast<Key extends keyof stencila.Types>(
  node: { [key: string]: any },
  type: Key
): stencila.Types[Key] {
  const casted = { ...node, type }
  validate(casted, type)
  return casted as stencila.Types[Key]
}

// Load all schemas for use by Ajv
const schemas = globby
  .sync(path.join(__dirname, 'dist', '*.schema.json'))
  .map(file => fs.readJSONSync(file))

// Cached JSON Schema validation functions
const validators = new Ajv({
  schemas,
  jsonPointers: true
})

/**
 * Validate a node against a type's schema
 * @param node The node to validate
 * @param type The type to validate against
 */
export function validate<Key extends keyof stencila.Types>(
  node: any,
  type: Key
): stencila.Types[Key] {
  const validator = validators.getSchema(
    `https://stencila.github.com/schema/${type}.schema.json`
  )
  if (!validator) throw new Error(`No schema for type "${type}".`)
  if (!validator(node)) {
    const errors = (betterAjvErrors(validator.schema, node, validator.errors, {
      format: 'js'
    }) as unknown) as betterAjvErrors.IOutputError[]
    throw new Error(errors.map(error => `${error.error}`).join(';'))
  }
  return node
}

/**
 * Is a node valid with respect to a particular type's schema
 * @param node The node to check
 * @param type The type to check against
 */
export function valid<Key extends keyof stencila.Types>(
  node: any,
  type: Key
): boolean {
  try {
    validate(node, type)
    return true
  } catch (error) {
    return false
  }
}

// Cached JSON Schema validation/mutation functions
// These use Ajv options that mutate nodes so we
// keep them separate from pure non-mutating validators.
const mutators = new Ajv({
  schemas,
  jsonPointers: true,
  // Add values from `default` keyword when property is missing
  useDefaults: true,
  // Remove any additional properties
  removeAdditional: true,
  // Coerce type of data to match type keyword and coerce scalar
  // data to an array with one element and vice versa, as needed.
  coerceTypes: 'array'
})

/**
 * Mutate a node so it conforms to a type's schema
 * @param node The node to mutate
 * @param type The type to conform to
 */
export function mutate<Key extends keyof stencila.Types>(
  node: any,
  type: Key
): stencila.Types[Key] {
  const mutator = mutators.getSchema(
    `https://stencila.github.com/schema/${type}.schema.json`
  )
  if (!mutator) throw new Error(`No schema for type "${type}".`)
  const mutated = { ...node, type }
  if (!mutator(mutated)) {
    const errors = (betterAjvErrors(mutator.schema, node, mutator.errors, {
      format: 'js'
    }) as unknown) as betterAjvErrors.IOutputError[]
    throw new Error(errors.map(error => `${error.error}`).join(';'))
  }
  return mutated
}
