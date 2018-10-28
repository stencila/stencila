/**
 * Decorators for storing meta data on types and properties
 * for runtime type checking, schema validation
 * and serialization / serialization.
 */

import 'reflect-metadata'

/**
 * Define a type
 *
 * @param id The @id of the type e.g. `schema:Thing` for https://schema.org/Thing
 */
export function type (id: string): ClassDecorator {
  return function (target: Object) {
    Reflect.defineMetadata('type:id', id, target)
  }
}

/**
 * Define a property
 *
 * See https://www.w3.org/2018/jsonld-cg-reports/json-ld/#syntax-tokens-and-keywords
 *
 * @param id The @id of the property e.g. `schema:name` for https://schema.org/name
 * @param container The @container type for the property e.g. set, list
 */
export function property (id: string, container?: string): PropertyDecorator {
  return function (target: Object, propertyKey: string | symbol) {
    Reflect.defineMetadata('property:id', id, target, propertyKey)
    if (container) Reflect.defineMetadata('property:container', container, target, propertyKey)
  }
}
