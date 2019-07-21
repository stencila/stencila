/**
 * Module for defining a `Schema` interface for use by other modules
 * in this repository e.g. generation of `*.json.schema` files by `./schema.ts`.
 */

import { JSONSchema7, JSONSchema7Definition } from 'json-schema'

/**
 * Interface for Stencila `Schema` instances.
 *
 * The `Schema` interface extends JSON Schema
 * with additional properties for:
 *
 *   - defining inheritance,
 *   - defining property aliases
 *   - specifying codecs used in coercion
 *   - categorizing node types
 *
 * For more details see the guidelines for authoring schemas.
 */
export default interface Schema extends JSONSchema7 {
  /**
   * The role that this schema has.
   */
  role?: 'base' | 'primary' | 'secondary' | 'tertiary'

  /**
   * The current status of this schema.
   */
  status?: 'experimental' | 'unstable' | 'stable'

  /**
   * The category of node that this schema belongs to.
   */
  category?: string

  /**
   * The schema that this schema extends.
   */
  extends?: string

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
   * Aliases for this property schema.
   * Only applies when used in a property of another schema.
   */
  aliases?: string[]

  /**
   * A map of property aliases.
   * Added during schema processing based on the `aliases`
   * of properties.
   *
   * TODO: Currently we use `aliases.json`, a separate file,
   * for storing aliases, but this could be used instead.
   */
  propertyAliases?: { [key: string]: string }

  /**
   * The name of a Encoda codec that can be used to decode
   * values for this schema.
   */
  codec?: string

  /**
   * The file in which this schema is defined.
   * Added during schema processing.
   */
  file?: string

  /**
   * The source file for this schema. A URL that can be used to
   * provide a link to view or edit the source.
   */
  source?: string

  // The following are type specializations of the
  // properties in `JSONSchema7` to match our usage
  // e.g. this `Schema` is used to define `properties`
  // of this `Schema`

  properties?: { [key: string]: Schema }
  allOf?: Schema[]
  anyOf?: Schema[]
  items?: Schema[]
  enum?: (string | number)[]
}
