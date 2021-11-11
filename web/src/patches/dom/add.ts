import { OperationAdd, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
  assertIndex,
  assertName,
  assertString,
  isAttr,
  isElement,
  isName,
  panic,
} from '../checks'
import { applyAdd as applyAddString } from '../string'
import { escapeAttr, unescapeAttr, unescapeHtml } from './escape'
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
    assertString(op.value)
    applyAddText(parent, slot, op.value)
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
 * Apply an `Add` operation to an element representing an optional property of a `struct`.
 */
export function applyAddOption(elem: Element, slot: Slot, html: string): void {
  assertName(slot)

  // If the provided HTML does not start with an opening angle bracket `<` then the value
  // being added must be a string (the only value type that does not get wrapped in an element)
  // so wrap it.
  if (!html.startsWith('<')) {
    html = `<span slot="${slot}">${html}</span>`
  }
  const fragment = createFragment(html)

  // Is the property designated to be added as an element attribute?
  if (ADD_ATTRIBUTES.includes(slot)) {
    elem.setAttribute(slot, fragment.textContent ?? '')
    return
  }

  // Is there a placeholder element? If so insert into it
  const placeholder = elem.querySelector(`[data-itemprop="${slot}"]`)
  if (placeholder) {
    placeholder.innerHTML = ''
    placeholder.appendChild(fragment)
    return
  }

  // Otherwise add the element and warn
  elem.appendChild(fragment)
  console.warn(
    `Unable to find attribute or placeholder element for property "${slot}"; was appended`
  )
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
  value: string
): void {
  const current = node.textContent ?? ''
  const unescaped = isAttr(node) ? unescapeAttr(current) : unescapeHtml(current)
  const updated = applyAddString(unescaped, slot, value)
  // It seems that, because setting textContent (?), it is not necessary to escape innerHTML
  const escaped = isAttr(node) ? escapeAttr(updated) : updated
  node.textContent = escaped
}
