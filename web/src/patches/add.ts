import { DomOperationAdd, Slot } from '@stencila/stencila'
import { ElementId } from '../types'
import {
  assert,
  assertNumber,
  assertString,
  createFragment,
  isElement,
  isString,
  panic,
  resolveParent,
  toGraphemes,
} from './utils'

/**
 * Apply an add operation
 */
export function applyAdd(op: DomOperationAdd, target?: ElementId): void {
  const { address, html } = op
  const [parent, slot] = resolveParent(address, target)

  if (isElement(parent)) {
    if (isString(slot)) applyAddOption(parent, slot, html)
    else applyAddVec(parent, slot, html)
  } else {
    applyAddString(parent, slot, html)
  }
}

/**
 * The HTML element attributes that may be added if the slot name is matching.
 *
 * These are [HTML attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes)
 * that are also Stencila Schema property names.
 */
const ADD_ATTRIBUTES = ['id', 'value', 'rowspan', 'colspan']

/**
 * Apply an add operation to an `Option` slot
 *
 * If the provided HTML does not start with an opening angle bracket `<` then the value
 * being added must be a string (the only value type that does not get wrapped in an element)
 * so wrap it.
 */
export function applyAddOption(node: Element, slot: Slot, html: string): void {
  assertString(slot)

  if (!html.startsWith('<')) {
    html = `<span slot="${slot}">${html}</span>`
  }

  const fragment = createFragment(html)
  if (ADD_ATTRIBUTES.includes(slot)) {
    node.setAttribute(slot, fragment.textContent ?? '')
  } else {
    node.appendChild(fragment)
  }
}

/**
 * Apply an add operation to a `Vec` slot
 */
export function applyAddVec(node: Element, slot: Slot, html: string): void {
  assertNumber(slot)

  const fragment = createFragment(html)
  const children = node.childNodes
  if (slot === children.length) {
    node.appendChild(fragment)
  } else {
    const sibling = node.childNodes[slot]
    if (sibling === undefined)
      throw panic(
        `Unexpected add slot '${slot}' for element with ${children.length} children`
      )
    node.insertBefore(fragment, sibling)
  }
}

/**
 * Apply an add operation to a `String` slot
 */
export function applyAddString(
  node: Attr | Text,
  slot: Slot,
  value: string
): void {
  assertNumber(slot)

  const graphemes = toGraphemes(node.textContent ?? '')
  assert(
    slot >= 0 && slot <= graphemes.length,
    `Unexpected add slot '${slot}' for text node of length ${graphemes.length}`
  )
  node.textContent =
    graphemes.slice(0, slot).join('') + value + graphemes.slice(slot).join('')
}
