import compile from './compile'
import Thing from './types/Thing'

/**
 * Build a `Thing`.
 *
 * The `build` function, like the `compile` function is used to prepare a thing
 * for execution. However, it usually involves the creation of build artifacts
 * (which may take some time to build) that are exernal to the thing
 * e.g. a binary executable or Docker image.
 * Like `compile`, it may add or modify properties of the thing
 * such as providing a URL to the built artifacts.
 *
 * @param thing The thing to build
 * @param format The format of the thing as a MIME type (only applicable when `thing` is a string)
 */
export default function build (thing: string | object | Thing, format: string = 'application/ld+json'): Thing {
  thing = compile(thing, format)
  return thing as Thing
}
