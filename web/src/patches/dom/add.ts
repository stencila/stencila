import { OperationAdd, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
  assertIndex,
  assertName,
  assertString,
  isElement,
  isName,
  panic,
} from '../checks'
import { applyAdd as applyAddString } from '../string'
import { createFragment, resolveParent } from './resolve'

/**
 * Apply an add operation
 */
export function applyAdd(op: OperationAdd, target?: ElementId): void {
  const { address, html } = op
  assertString(html)

  const [parent, slot] = resolveParent(address, target)

  if (isElement(parent)) {
    if (isName(slot)) applyAddOption(parent, slot, html)
    else applyAddVec(parent, slot, html)
  } else {
    applyAddText(parent, slot, html)
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
 * Apply an `Add` operation to an element representing an `Option`.
 *
 * If the provided HTML does not start with an opening angle bracket `<` then the value
 * being added must be a string (the only value type that does not get wrapped in an element)
 * so wrap it.
 */
export function applyAddOption(node: Element, slot: Slot, html: string): void {
  assertName(slot)

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
 * Apply an `Add` operation to an element representing a `Vec`.
 */
export function applyAddVec(node: Element, slot: Slot, html: string): void {
  assertIndex(slot)

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
 * Apply an `Add` operation to a `Text` or `Attr` DOM node
 */
export function applyAddText(
  node: Text | Attr,
  slot: Slot,
  html: string
): void {
  node.textContent = applyAddString(node.textContent ?? '', slot, html)
}
