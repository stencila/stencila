import { Types } from '../types'
import { jsonLdTermUrl, jsonLdContext, jsonLdTermName } from './jsonld'

/**
 * Get the HTML Microdata `itemtype` for a Stencila Schema type
 *
 * @see {@link https://www.w3.org/TR/microdata/#dfn-itemtype}
 */
export function microdataItemtype(type: keyof Types): string | undefined {
  return jsonLdTermUrl(type)
}

/**
 * Get the Stencila Schema type from a HTML Microdata `itemtype`.
 *
 * This is the inverse of `microdataItemtype`.
 */
export function microdataType(itemtype: string): keyof Types | undefined {
  return jsonLdTermName(itemtype) as keyof Types
}

/**
 * Get the HTML Microdata `itemprop` for a Stencila Schema property.
 *
 * The `itemprop` attribute is normally just the name of the property
 * i.e. it is not prefixed by a base URL. This function returns the
 * `[prefix, name]` pair e.g. `["schema", "author"]`,
 * `["codemeta", "maintainer"]` because you may only want to encode
 * `itemprop`s for well known schemas e.g. schema.org
 *
 * @see {@link https://www.w3.org/TR/microdata/#dfn-attr-itemprop}
 */
export function microdataItemprop(
  property: string
): [string | undefined, string | undefined] {
  const context = jsonLdContext()
  const mapping = context[property]
  if (mapping === undefined) return [undefined, undefined]
  if (typeof mapping === 'string') return [undefined, mapping]

  const id = mapping['@id']
  const parts = id.split(':')
  return parts.length === 1 ? [undefined, parts[0]] : [parts[0], parts[1]]
}
