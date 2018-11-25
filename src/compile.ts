import import_ from './import'
import Thing from './types/Thing'

/**
 * Compile a thing
 *
 * @param thing The thing to compile
 * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
 */
export default function compile (thing: string | object | Thing, format: string = 'application/ld+json'): Thing {
  thing = import_(thing, format)
  return thing as Thing
}
