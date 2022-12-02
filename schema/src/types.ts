import { JSONSchema7 } from 'json-schema'

/**
 * Interface for Stencila `JsonSchema` instances.
 *
 * The `JsonSchema` interface extends JSON Schema with additional
 * properties including for:
 *
 *   - compatibility with JSON-LD
 *   - defining inheritance
 *   - defining property aliases
 *   - categorizing node types
 */
export interface JsonSchema extends JSONSchema7 {
  /**
   * The id for the type or property schema to be used
   * when generating JSON-LD.
   */
  '@id'?: string

  /**
   * The current status of this schema.
   */
  status?: 'experimental' | 'unstable' | 'stable'

  /**
   * The schema that this schema extends.
   */
  extends?: string

  /**
   * The URL of the source file in which this schema is defined.
   * Added when the file is read in to memory.
   */
  source?: string

  /**
   * The names of the child (direct descendants) schemas of this schema.
   * Added during schema processing.
   */
  children?: string[]

  /**
   * The descendant schemas of this schema.
   * Added during schema processing.
   */
  descendants?: string[]

  /**
   * The schema from which this property schema was inherited.
   * Only applies when used in a property of another schema.
   * Added during schema processing.
   */
  from?: string

  /**
   * Is this property schema an override of a property inherited
   * from an ancestor. Examples of overrides include making an
   * optional property required, or changing the schema of the property.
   */
  isOverride?: boolean

  /**
   * Is the property an array?
   */
  isArray?: boolean

  /**
   * Is the property an array and have a pluralized name e.g. authors
   */
  isPlural?: boolean

  /**
   * Aliases for this property schema.
   * Only applies when used in a property of another schema.
   */
  aliases?: string[]

  /**
   * A map of property aliases.
   * Added during schema processing based on the `aliases`
   * of properties.
   */
  propertyAliases?: { [key: string]: string }

  // The following are type specializations of the
  // properties in `JSONSchema7` to match our usage
  // e.g. this `JsonSchema` is used to define `properties`
  // of this `JsonSchema`

  properties?: { [key: string]: JsonSchema }
  allOf?: JsonSchema[]
  anyOf?: JsonSchema[]
  items?: JsonSchema[]
  enum?: (string | number)[]
}
