/**
 * Decorators for storing meta data on types and properties
 * for runtime type checking, schema validation
 * and serialization / serialization.
 */

import 'reflect-metadata'

/**
 * Define a type.
 *
 * @param id The `@id` of the type e.g. `schema:Thing` for https://schema.org/Thing
 */
export function type (id: string): ClassDecorator {
  return function (target: Object) {
    Reflect.defineMetadata('type:id', id, target)
  }
}

/**
 * Define a property.
 *
 * @param id        The `@id` of the property e.g. `schema:name` for https://schema.org/name
 * @param container The `@container` type for the property. Must be `list` or `set` (default).
 *                  A `list` is a ordered collection. A `set` is an unordered collection.
 *                  See the [JSON-LD docs](https://w3c.github.io/json-ld-syntax/#sets-and-lists)
 *                  for more info.
 */
export function property (id: string, container: string = 'set'): PropertyDecorator {
  return function (target: Object, propertyKey: string | symbol) {
    Reflect.defineMetadata('property:name', propertyKey, target, propertyKey)
    Reflect.defineMetadata('property:id', id, target, propertyKey)
    if (container) Reflect.defineMetadata('property:container', container, target, propertyKey)
  }
}
