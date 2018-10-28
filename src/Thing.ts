import 'reflect-metadata'

import { Text, URL } from './dataTypes'
import { type, property } from './decorators'

@type('schema:Thing')
export default class Thing {
  /**
   * The JSON-LD [type specifier](https://w3c.github.io/json-ld-syntax/#specifying-the-type) corresponding to
   * the `@type` keyword. This should be overriden in all derived classes.
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
   * All other values are ignored.
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
   * Marshall this instance to a JSON-LD object
   */
  toJSONLD (standalone: boolean = true) {
    const jsonld: {[key: string]: any} = {}
    if (standalone) jsonld['@context'] = 'https://stencila.github.io/schema/context.jsonld'
    jsonld['type'] = this.type

    for (let [key, value] of Object.entries(this)) {
      if (typeof value === 'string' && value.length === 0) continue
      if (Array.isArray(value) && value.length === 0) continue

      let id = Reflect.getMetadata('property:id', this, key)
      let [context, term] = id.split(':')
      if (value instanceof Thing) {
        jsonld[term] = value.toJSONLD(false)
      } else {
        jsonld[term] = value
      }
    }

    return jsonld
  }

  @property('schema:description')
  description: Text = ''

  @property('schema:identifier', 'list')
  identifiers: Array<Text | URL> = []

  @property('schema:name')
  name: Text = ''

  @property('schema:urls', 'list')
  urls: Array<URL> = []
}
