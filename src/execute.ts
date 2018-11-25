import build from './build'
import Thing from './types/Thing'

/**
 * Execute a thing
 *
 * @param thing The thing to execute
 * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
 */
export default function execute (thing: string | object | Thing, format: string= 'application/ld+json'): Thing {
  thing = build(thing, format)
  return thing as Thing
}
