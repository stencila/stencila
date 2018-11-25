import import_ from './import'
import Thing from './types/Thing'

/**
 * Export a `Thing`.
 *
 * @param thing The thing to be exported
 * @param format The format, as a MIME type, to export to e.g. `text/html`
 */
export default function export_ (thing: string | object | Thing, format: string= 'application/ld+json'): string {
  if (!(thing instanceof Thing)) thing = import_(thing)

  switch (format) {
    case 'application/ld+json':
      return exportJsonLd(thing as Thing)
    default:
      throw Error(`Unhandled export format: ${format}`)
  }
}

/**
 * Export a `Thing` to a JSON-LD string
 *
 * @param thing The thing to be exported
 */
export function exportJsonLd (thing: Thing): string {
  const obj = Object.assign({
    '@context': 'https://stencila.github.io/schema/context.jsonld'
  }, exportObject(thing))
  return JSON.stringify(obj)
}

/**
 * Export a `Thing` to an `Object`
 *
 * This function marshalls a `Thing` to a plain JavaScript object
 * having a `type` and other properties of the type of thing.
 *
 * @param thing The thing to be exported
 */
export function exportObject (thing: Thing): {[key: string]: any} {
  const obj: {[key: string]: any} = {}
  obj['type'] = thing.type

  for (let [key, value] of Object.entries(thing)) {
    if (typeof value === 'string' && value.length === 0) continue
    if (Array.isArray(value) && value.length === 0) continue

    let id = Reflect.getMetadata('property:id', thing, key)
    let [context, term] = id.split(':')
    if (Array.isArray(value)) {
      obj[term] = value.map(item => (item instanceof Thing) ? exportObject(item) : item)
    } else if (value instanceof Thing) {
      obj[term] = exportObject(value)
    } else {
      obj[term] = value
    }
  }

  return obj
}
