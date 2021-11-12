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
    if (isName(slot)) applyRemoveStruct(parent, slot, items)
    else applyRemoveVec(parent, slot, items)
  } else applyRemoveText(parent, slot, items)
}

/**
 * Apply a `Remove` operation to an element representing a `struct`
 */
export function applyRemoveStruct(
  struct: Element,
  name: Slot,
  items: number
): void {
  assertName(name)
  assert(
    items === 1,
    `Unexpected remove items ${items} for option slot '${name}'`
  )

  // If the property is represented as an attribute then remove it
  if (struct.hasAttribute(name)) {
    struct.removeAttribute(name)
    return
  }

  // If the property is represented as a child element then clear it's content
  // and its attributes, other than `data-itemprop` (so that it remains
  // a placeholder if the property is added again later).
  const child = struct.querySelector(`[data-itemprop="${name}"]`)
  if (child) {
    child.innerHTML = ''
    for (const attr of child.getAttributeNames()) {
      if (attr !== 'data-itemprop') child.removeAttribute(attr)
    }
    return
  }

  console.warn(`Unable to find existing property "${name}" to remove"`)
}

/**
 * Apply a `Remove` operation to an element representing a `Vec`.
 */
export function applyRemoveVec(vec: Element, index: Slot, items: number): void {
  assertIndex(index)

  const children = vec.childNodes
  assert(
    index >= 0 && index < children.length,
    `Unexpected remove slot '${index}' for element with ${children.length} children`
  )
  assert(
    items > 0 && index + items <= children.length,
    `Unexpected remove items ${items} for element with ${children.length} children`
  )

  let removed = 0
  while (removed < items) {
    children[index]?.remove()
    removed += 1
  }
}

/**
 * Apply a `Remove` operation to a `Text` or `Attr` DOM node representing a `String`
 */
export function applyRemoveText(
  text: Attr | Text,
  index: Slot,
  items: number
): void {
  assertIndex(index)

  const current = text.textContent ?? ''
  const unescaped = isAttr(text) ? unescapeAttr(current) : unescapeHtml(current)
  const updated = applyRemoveString(unescaped, index, items)
  text.textContent = updated
}
