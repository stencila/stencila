// Node.js bindings for ../../rust/src/config.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON, toJSON } from './prelude'
import { Config } from './types'

const addon = require('../index.node')

/**
 * Get the JSON schema for the configuration object.
 *
 * @returns A JSON Schema v7 object describing the properties of
 *          the configuration object
 */
export function schema(): JSONSchema7 {
  return fromJSON<JSONSchema7>(addon.configSchema())
}

/**
 * Get the global configuration object.
 *
 * @returns The configuration object
 */
export function get(): Config {
  return fromJSON<Config>(addon.configGet())
}

/**
 * Set the entire global configuration object.
 * 
 * The returned object may be different because defaults are populated.
 *
 * @returns The configuration object
 */
 export function set(config: Config): Config {
  return fromJSON<Config>(addon.configSet(toJSON(config)))
}

/**
 * Test that a configuration object is valid.
 *
 * @param config
 * @returns true (or throws an error)
 */
export function validate(config: Config): true {
  return addon.configValidate(toJSON(config))
}

/**
 * Set a property of the global configuration object.
 *
 * Performs validation on the value before writing to disk. 
 * Will throw errors for invalid pointer or error.
 *
 * @param pointer The pointer to the property to be set e.g. `upgrade.auto`
 * @param value The value to set the property to
 * @returns The updated configuration object
 */
export function setProperty(pointer: string, value: string): Config {
  return fromJSON<Config>(addon.configSetProperty(pointer, value))
}

/**
 * Reset all or part of the global configuration object to defaults.
 *
 * @param config The configuration object
 * @param property The property to reset. Use `all` to reset the entire object.
 * @returns The updated configuration object
 */
export function resetProperty(
  property: 'all' | 'logging' | 'serve' | 'plugins' | 'upgrade'
): Config {
  return fromJSON<Config>(addon.configResetProperty(property))
}
