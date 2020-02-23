import { Node, Types } from '../types'
import {
  jsonLdContext,
  jsonLdTermName,
  jsonLdTermUrl,
  jsonLdUrl
} from './jsonld'
import { nodeType } from './nodeType'

/**
 * Get the URL used in Microdata attributes.
 *
 * This is used to normalize the versioned URL from the
 * JSON-LD context.
 */
export function microdataUrl(type = '') {
  return `http://schema.stenci.la/${type}`
}

export type Microdata = MicrodataItem & MicrodataProperty

/**
 * Create all Microdata attributes for a Stencila `Node`.
 *
 * @param node The node e.g. a `Person` node
 * @param property The name of the property that this node is part of e.g `'author'`
 */
export function microdata(
  node: Node,
  property?: string,
  id?: string
): Microdata {
  return {
    ...microdataItem(node, property === undefined ? id : undefined),
    ...(property !== undefined ? microdataProperty(property, id) : {})
  }
}

/**
 * Attributes for Microdata ["items"](https://www.w3.org/TR/microdata/#items)
 *
 * "The itemtype attribute must not be specified on elements that do not have
 *  an itemscope attribute specified."
 */
export interface MicrodataItem {
  itemscope: ''
  itemtype: string
  itemid?: string
}

export function microdataItem(node: Node, id?: string): MicrodataItem {
  const itemtype = microdataItemtype(nodeType(node)) ?? 'Thing'
  const itemid = id !== undefined ? { itemid: `#${id}` } : {}
  return {
    itemscope: '',
    itemtype,
    ...itemid
  }
}

/**
 * Get the HTML Microdata `itemtype` for a Stencila Schema type
 *
 * @see {@link https://www.w3.org/TR/microdata/#dfn-itemtype}
 */
export function microdataItemtype(type: keyof Types): string | undefined {
  return jsonLdTermUrl(type)?.replace(jsonLdUrl(), microdataUrl())
}

/**
 * Get the Stencila Schema type from a HTML Microdata `itemtype`.
 *
 * This is the inverse of `microdataItemtype`.
 */
export function microdataType(itemtype: string): keyof Types | undefined {
  return jsonLdTermName(
    itemtype.replace(microdataUrl(), jsonLdUrl())
  ) as keyof Types
}

/**
 * Attributes for Microdata ["properties"](https://www.w3.org/TR/microdata/#names:-the-itemprop-attribute)
 *
 * The `data-itemprop` attribute is not part of the Microdata standard.
 * It is used for properties that are not defined in schema.org and which validators
 * like Google Structured Data Testing Tool throw errors about.
 */
export interface MicrodataProperty {
  itemprop?: string
  'data-itemprop'?: string
  itemref?: string
}

/**
 * Create `MicrodataProperty` attributes for a node property.
 */
export function microdataProperty(
  property: string,
  id?: string
): MicrodataProperty {
  const [prefix, name = ''] = microdataItemprop(property)
  const key = prefix === 'schema' ? 'itemprop' : 'data-itemprop'
  const itemref = id !== undefined ? { itemref: id } : {}
  return { [key]: name, ...itemref }
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
