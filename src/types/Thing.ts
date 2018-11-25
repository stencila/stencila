import 'reflect-metadata'

import { Text, URL } from './dataTypes'
import { type, property } from './decorators'

/**
 * The most generic type of item.
 *
 * This is base class for all other classes in this schema.
 * As well as definining the properties of a `schema:Thing` it
 * implements methods such as `toJSONLD` for marshalling to JSON-LD.
 *
 * @see {@link https://schema.org/Thing}
 */
@type('schema:Thing')
export default class Thing {
  /**
   * The JSON-LD [type specifier](https://w3c.github.io/json-ld-syntax/#specifying-the-type) corresponding to
   * the `@type` keyword.
   */
  get type (): string {
    return this.constructor.name
  }

  /**
   * The JSON-LD [node identifier](https://w3c.github.io/json-ld-syntax/#node-identifiers) corresponding to
   * the `@id` keyword.
   */
  id: string = ''

  /**
   * Constructor
   *
   * Uses the values of properties in the initializer.
   * Only registered properties (i.e. those with the @property decorator) are initialized.
   * All other values are ignored without warning.
   *
   * @param initializer An object with initial property values
   */
  constructor (initializer = {}) {
    for (let [key, value] of Object.entries(initializer)) {
      if (Reflect.hasMetadata('property:id', this, key)) {
        // @ts-ignore
        this[key] = value
      }
    }
  }

  /**
   * A description of the item.
   *
   * @see {@link https://schema.org/description}
   */
  @property('schema:description')
  description: Text = ''

  /**
   * The identifier property represents any kind of identifier for any kind of Thing,
   * such as ISBNs, GTIN codes, UUIDs etc. Schema.org provides dedicated properties
   * for representing many of these, either as textual strings or as URL (URI) links.
   *
   * @see {@link https://schema.org/identifier}
   */
  @property('schema:identifier')
  identifiers: Array<Text | URL> = []

  /**
   * The name of the item.
   *
   * @see {@link https://schema.org/name}
   */
  @property('schema:name')
  name: Text = ''

  /**
   * URL of the item.
   *
   * @see {@link https://schema.org/url}
   */
  @property('schema:url')
  urls: Array<URL> = []
}
