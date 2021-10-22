import { OperationTransform } from '@stencila/stencila'
import { ElementId } from '../../types'
import { assert, isElement, isText, panic } from '../checks'
import { resolveNode } from './resolve'

/**
 * Apply a transform operation
 *
 * Transform operations allow for a lightweight diff where only the type
 * of the node has changed. See the `diff_transform` function in `rust/src/patches/inlines.rs`
 * This function should be able to apply all the transforms potentially
 * generated on the server.
 *
 * Asserts that the element type is as expect in the `from` property of the operation.
 */
export function applyTransform(
  op: OperationTransform,
  target?: ElementId
): void {
  const { address, from, to } = op

  const node = resolveNode(address, target)
  if (isText(node)) applyTransformString(node, from, to)
  else if (isElement(node)) applyTransformElem(node, from, to)
  else throw panic(`Unexpected transform node`)
}

/**
 * Apply a transform operation to a `String` slot
 */
export function applyTransformString(
  text: Text,
  from: string,
  to: string
): void {
  assert(from === 'String', `Expected transform from type String, got ${from}`)

  const tag = TYPE_TAGS[to]
  if (tag === undefined) {
    throw panic(`Unexpected transform to type ${to}`)
  }

  const elem = document.createElement(tag)
  elem.textContent = text.textContent
  text.replaceWith(elem)
}

/**
 * Apply a transform operation a `Node` slot
 */
export function applyTransformElem(
  elem: Element,
  from: string,
  to: string
): void {
  const tag = elem.tagName.toLowerCase()
  const expectedFrom = TAGS_TYPE[tag]
  if (expectedFrom === undefined) throw panic(`Unhandled from tag ${tag}`)
  if (expectedFrom !== from)
    throw panic(
      `Expected transform from type ${expectedFrom} for tag ${tag}, got ${from}`
    )

  if (to === 'String') {
    const text = document.createTextNode(elem.textContent ?? '')
    elem.replaceWith(text)
  } else {
    const tag = TYPE_TAGS[to]
    if (tag === undefined) throw panic(`Unhandled to type ${to}`)
    const transformed = document.createElement(tag)
    transformed.innerHTML = elem.innerHTML
    for (let index = 0; index < elem.attributes.length; index++) {
      const attr = elem.attributes[index] as Attr
      if (attr.name === 'itemtype') {
        transformed.setAttribute(attr.name, `https://stenci.la/${to}`)
      } else {
        transformed.setAttribute(attr.name, attr.value)
      }
    }
    elem.replaceWith(transformed)
  }
}

/**
 * Tags used for various node types
 */
const TYPE_TAGS: Record<string, string> = {
  Emphasis: 'em',
  Delete: 'del',
  Strong: 'strong',
  Subscript: 'sub',
  Superscript: 'sup',
}

/**
 * Types corresponding to various element tags
 */
const TAGS_TYPE: Record<string, string> = {
  em: 'Emphasis',
  del: 'Delete',
  strong: 'Strong',
  sub: 'Subscript',
  sup: 'Superscript',
}
