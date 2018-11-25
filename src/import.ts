import Thing from './Thing'
import * as types from './types'

/**
 * Import a `Thing`.
 *
 * @param thing The thing to be imported
 * @param format The current format of the thing as a MIME type e.g. `text/markdown`
 * @returns An instance of a class derived from `Thing`
 */
export default function import_ (thing: string | object | Thing, format: string= 'application/ld+json'): Thing {
  if (thing instanceof Thing) {
    return thing
  } else if (typeof thing === 'object') {
    return importObject(thing)
  } else {
    switch (format) {
      case 'application/ld+json':
        return importJsonLd(thing)
      default:
        throw Error(`Unhandled import format: ${format}`)
    }
  }
}

/**
 * Import an `Object` to a `Thing`
 *
 * This function demarshalls a plain JavaScript object into an
 * instance of a class derived from `Thing` based on the `type`
 * property of the object.
 *
 * @param object A plain JavaScript object with a `type` property
 * @returns An instance of a class derived from `Thing`
 */
export function importObject (object: any): Thing {
  const type = object.type
  if (!type) throw new Error('Object is missing required "type" property')
  // @ts-ignore
  const Type = types[type]
  if (!Type) throw new Error(`Unknown type "${type}"`)
  return new Type(object)
}

/**
 * Import a JSON-LD document to a `Thing`
 *
 * @param jsonld A JSON-LD document with a `type` property
 * @returns An instance of a class derived from `Thing`
 */
export function importJsonLd (jsonld: string): Thing {
  const object = JSON.parse(jsonld)
  return importObject(object)
}
