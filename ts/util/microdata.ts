import { Node, Types } from '../types'
import {
  jsonLdContext,
  jsonLdTermName,
  jsonLdTermUrl,
  jsonLdUrl,
} from './jsonld'
import { nodeType } from './nodeType'
import { isPrimitive } from './guards'

/**
 * Get the URL used in Microdata attributes.
 *
 * This is used to normalize the versioned URL from the
 * JSON-LD context.
 */
export function microdataUrl(type = ''): string {
  return `http://schema.stenci.la/${type}`
}

export type Microdata = MicrodataItem & MicrodataProperty

type Role = 'array' | 'item'

/**
 * Create all Microdata attributes for a Stencila `Node`.
 *
 * @param node The node e.g. a `Person` node
 * @param property The name of the property that this node is part of e.g `authors`
 * @param role Is this an item within an array property e.g a `Person` within `authors`
 * @param id The id used to link to / from this Microdata item
 */
export function microdata(
  node: Node,
  property?: string,
  role?: Role,
  id?: string
): Microdata {
  return {
    ...(role !== 'array'
      ? microdataItem(node, property === undefined ? id : undefined)
      : {}),
    ...(property !== undefined ? microdataProperty(property, role, id) : {}),
  }
}

/**
 * Attributes for Microdata ["items"](https://www.w3.org/TR/microdata/#items)
 *
 * "The itemtype attribute must not be specified on elements that do not have
 *  an itemscope attribute specified."
 */
export interface MicrodataItem {
  itemscope?: ''
  itemtype?: string
  'data-itemtype'?: string
  itemid?: string
}

/**
 * Create `MicrodataItem` attributes for a node.
 *
 * Does not create the `itemscope` and `itemtype` attributes for nodes that
 * are primitive (and therefore which do not represent a "scope" having
 * `itemprop`s nested within it). Instead, for primitive nodes, other than `Text`
 * add the `data-itemtype` attribute, do they can be styled if so desired.
 *
 * @param node The node to create Microdata attributes for
 * @param id Id of the Microdata item. Used to link to this node using the `itemref` property.
 */
export function microdataItem(node: Node, id?: string): MicrodataItem {
  const itemtype = microdataItemtype(nodeType(node))
  const itemidAttr = id !== undefined ? { itemid: `#${id}` } : {}
  if (itemtype !== undefined && !isPrimitive(node))
    return {
      itemscope: '',
      itemtype,
      ...itemidAttr,
    }
  else if (typeof node !== 'string')
    return {
      'data-itemtype': itemtype,
      ...itemidAttr,
    }
  else return itemidAttr
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
 *
 * @param property The name of the property
 * @param isItem Is the node for which attributes are being generated
 *                    an item within an array property? e.g. a `Person` in `authors`.
 * @param id Id of another Microdata item to link to using the `itemref` property.
 */
export function microdataProperty(
  property: string,
  role?: Role,
  id?: string
): MicrodataProperty {
  const [prefix, name = ''] = microdataItemprop(property, role)
  const key = prefix === 'schema' ? 'itemprop' : 'data-itemprop'
  const itemrefAttr = id !== undefined ? { itemref: id } : {}
  return { [key]: name, ...itemrefAttr }
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
 *
 * @param property The name of the property
 * @param role Is the node for which attributes are being generated
 *               an item within an array property? e.g. a `Person` in `authors`.
 */
export function microdataItemprop(
  property: string,
  role?: Role
): [string | undefined, string | undefined] {
  if (role === 'array') return ['stencila', property]

  const context = jsonLdContext()
  const mapping = context[property]
  if (mapping === undefined) return [undefined, undefined]
  if (typeof mapping === 'string') return [undefined, mapping]

  const id = mapping['@id']
  let [prefix, name] = id.split(':')

  // If this is an item in an array property and the
  // name is a plural, then return a singular name.
  // For external vocabs the `@id` is usually a singular already e.g.
  //   schema:author
  //   codemeta:maintainer
  if (role === 'item' && prefix === 'stencila' && name.endsWith('s'))
    name = name.slice(0, -1)

  return [prefix, name]
}

/**
 * Get the 'pseudo' HTML Microdata attribute for the root element.
 *
 * This attribute name / value pair is used to scope CSS variables to
 * the root Stencila node in an HML document. It is used by Encoda when
 * encoding to HTML, it is in Thema to scope CSS variable thereby
 * avoiding variable name clashes from using the CSS `:root` pseudo-class.
 *
 * Although not directly related to Microdata, given it is used in both
 * of those projects, this appears to be the best place for it.
 */
export function microdataRoot(): { 'data-itemscope': 'root' } {
  return { 'data-itemscope': 'root' }
}
