import { OperationRemove, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
  isName,
  assert,
  assertIndex,
  assertName,
  isAttr,
  isElement,
} from '../checks'
import { applyRemove as applyRemoveString } from '../string'
import { unescapeAttr, unescapeHtml } from './escape'
import { resolveParent } from './resolve'

/**
 * Apply a `Remove` operation
 */
export function applyRemove(op: OperationRemove, target?: ElementId): void {
  const { address, items } = op

  const [parent, slot] = resolveParent(address, target)

  if (isElement(parent)) {
    if (isName(slot)) applyRemoveOption(parent, slot, items)
    else applyRemoveVec(parent, slot, items)
  } else applyRemoveText(parent, slot, items)
}

/**
 * Apply a `Remove` operation to an optional property of a struct
 */
export function applyRemoveOption(
  elem: Element,
  slot: Slot,
  items: number
): void {
  assertName(slot)
  assert(
    items === 1,
    `Unexpected remove items ${items} for option slot '${slot}'`
  )

  // If represented as an attribute then remove it
  if (elem.hasAttribute(slot)) {
    elem.removeAttribute(slot)
    return
  }

  // If represented as a child element then clear it's content
  // and its attributes (other than `data-itemprop`) so that it remains
  // a placeholder if the property is added again later.
  const child = elem.querySelector(`[data-itemprop="${slot}"]`)
  if (child) {
    child.innerHTML = ''
    for (const attr of child.getAttributeNames()) {
      if (attr !== 'data-itemprop') child.removeAttribute(attr)
    }
    return
  }

  console.warn(`Unable to find existing property "${slot}" to remove"`)
}

/**
 * Apply a `Remove` operation to a `Vec` slot
 */
export function applyRemoveVec(node: Element, slot: Slot, items: number): void {
  assertIndex(slot)

  const children = node.childNodes
  assert(
    slot >= 0 && slot < children.length,
    `Unexpected remove slot '${slot}' for element with ${children.length} children`
  )
  assert(
    items > 0 && slot + items <= children.length,
    `Unexpected remove items ${items} for element with ${children.length} children`
  )

  let removed = 0
  while (removed < items) {
    children[slot]?.remove()
    removed += 1
  }
}

/**
 * Apply a `Remove` operation to a `Text` or `Attr` DOM node
 */
export function applyRemoveText(
  node: Attr | Text,
  slot: Slot,
  items: number
): void {
  const current = node.textContent ?? ''
  const unescaped = isAttr(node) ? unescapeAttr(current) : unescapeHtml(current)
  const updated = applyRemoveString(unescaped, slot, items)
  node.textContent = updated
}
