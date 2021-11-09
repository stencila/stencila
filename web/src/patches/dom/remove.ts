import { OperationRemove, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
  isName,
  assert,
  assertIndex,
  assertName,
  isAttr,
  isElement,
  panic,
} from '../checks'
import { applyRemove as applyRemoveString } from '../string'
import { unescapeAttr, unescapeHtml } from './escape'
import { resolveParent, resolveSlot } from './resolve'

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
 * Apply a `Remove` operation to an `Option` slot
 */
export function applyRemoveOption(
  node: Element,
  slot: Slot,
  items: number
): void {
  assertName(slot)
  assert(
    items === 1,
    `Unexpected remove items ${items} for option slot '${slot}'`
  )

  const child = resolveSlot(node, slot)
  if (isElement(child)) child.remove()
  else if (isAttr(child)) node.removeAttribute(child.name)
  else throw panic(`Unexpected remove child DOM node`)
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
