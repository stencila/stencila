import export_ from './export'
import import_ from './import'

/**
 * Convert a thing from one format to another.
 *
 * @param thing The thing to convert as a string
 * @param from The current format of the thing as a MIME type e.g. `text/markdown`
 * @param to The desired format for the thing as a MIME type e.g. `text/html`
 */
export default function convert (thing: string, from: string= 'application/ld+json', to: string= 'application/ld+json'): string {
  return export_(import_(thing, from), to)
}
