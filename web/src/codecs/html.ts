import { Cord, Node, NodeType } from '@stencila/types'

/**
 * Encode a Stencila node to DOM HTML
 *
 * This is a partial, browser-based implementation of the `codec-dom` codec
 * in `rust/codec-dom` which encodes Stencila nodes to a rich, mostly lossless
 * HTML for use in the browser by the Web Components in this module.
 *
 * The Rust codec mostly relies on the `DomCodec` derive macro which uses
 * the `dom` attributes set on fields of nodes e.g.
 *
 * #[dom(elem = "section")]
 *
 * In addition, some node types have a manual implementation of `DomCodec`
 * in `rust/schema/src/implem/*.rs`  e.g.
 *
 * impl DomCodec for Figure {
 *   fn to_dom(&self, context: &mut DomEncodeContext) {
 *     ...
 *   }
 * }
 */
export function encode(node: Node, ancestors: string[] = []): string {
  // Handle `Null` nodes
  if (node === null) {
    return '<stencila-null>null</stencila-null>'
  }

  // Handle `Primitive` nodes
  switch (typeof node) {
    case 'boolean':
    case 'number':
    case 'string':
      return `<stencila-${typeof node}>${node}</stencila-${typeof node}>`
    case 'bigint':
      return `<stencila-integer>${node}</stencila-integer>`
  }

  if (Array.isArray(node)) {
    // TODO
    return `<stencila-array></stencila-array>`
  }

  if (!Object.hasOwn(node, 'type')) {
    // TODO
    return `<stencila-object></stencila-object>`
  }

  const nodeType = node.type as NodeType
  const tag = tagName(nodeType)

  const SKIP = ['type', 'compilationDigest', 'executionDigest']
  const SLOT = {
    authors: 'span',
    provenance: 'span',
    content: null,
  }

  let attrsHtml = `depth=${ancestors.length} ancestors="${ancestors.join('.')}"`
  if (ancestors.length == 0) {
    attrsHtml += ' root'
  }
  let slotsHtml = ''
  for (const key of Object.keys(node)) {
    const slot = SLOT[key]
    if (slot !== undefined) {
      slotsHtml += propertySlot(slot, key, node[key], [...ancestors, nodeType])
    } else if (!SKIP.includes(key)) {
      attrsHtml += ' ' + attr(key, node[key])
    }
  }

  return `<${tag} ${attrsHtml}>${slotsHtml}</${tag}>`
}

/**
 * Generate a HTML tag name for a node type
 */
function tagName(type: NodeType): string {
  return `stencila${type.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}`
}

/**
 * Generate a HTML attribute for a node property
 */
function attr(name: string, value: Node): string {
  if (value !== null && typeof value == 'object' && !Array.isArray(value)) {
    if (value.type == undefined) {
      const cord = value as Cord
      if (cord.string) {
        return `${attrName(name)}="${attrValue(cord.string)}"`
      }
    }
  }

  return `${attrName(name)}="${attrValue(value)}"`
}

/**
 * Generate an HTML attribute value for a node property
 */
function attrName(name: string): string {
  return name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)
}

/**
 * Generate a HTML attribute value for a node
 */
function attrValue(node: Node): string {
  if (node === null) {
    return 'null'
  }

  const attr = (() => {
    switch (typeof node) {
      case 'boolean':
      case 'number':
      case 'string':
      case 'bigint':
        return node.toString()
    }

    if (Array.isArray(node) || !Object.hasOwn(node, 'type')) {
      return JSON.stringify(node)
    }

    switch (node.type) {
      case 'Timestamp':
      case 'Duration':
        return node.value.toString()
    }

    return JSON.stringify(node)
  })()

  return attr
    .replace(/&/g, '&amp;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

/**
 * Generate a HTML element for a node property
 */
function propertySlot(
  tagName: string | null,
  name: string,
  node: Node,
  ancestors: string[]
): string {
  const content = (Array.isArray(node) ? node : [node])
    .map((node: Node) => encode(node, ancestors))
    .join('')

  if (tagName === null) {
    return content
  }

  const slotName = name.replace(
    /[A-Z]/g,
    (letter) => `-${letter.toLowerCase()}`
  )

  return `<${tagName} slot="${slotName}">${content}</${tagName}>`
}
