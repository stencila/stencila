import { Text, URL } from './dataTypes'

/**
 * https://schema.org/Thing
 */
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

  constructor (initializer = {}) {
    Object.assign(this, initializer)
  }

  description: Text = ''

  identifiers: Array<Text | URL> = []

  name: Text = ''

  urls: Array<URL> = []
}
