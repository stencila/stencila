import { DomOperationRemove, Slot } from '@stencila/stencila'
import { ElementId } from '../types'
import {
  assert,
  assertNumber,
  assertString,
  isAttr,
  isElement,
  isString,
  panic,
  resolveParent,
  resolveSlot,
  toGraphemes,
} from './utils'

/**
 * Apply a remove operation
 */
export function applyRemove(op: DomOperationRemove, target?: ElementId): void {
  const { address, items } = op
  const [parent, slot] = resolveParent(address, target)

  if (isElement(parent)) {
    if (isString(slot)) applyRemoveOption(parent, slot, items)
    else applyRemoveVec(parent, slot, items)
  } else applyRemoveString(parent, slot, items)
}

/**
 * Apply a remove operation to an `Option` slot
 */
export function applyRemoveOption(
  node: Element,
  slot: Slot,
  items: number
): void {
  assertString(slot)
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
 * Apply a remove operation to a `Vec` slot
 */
export function applyRemoveVec(node: Element, slot: Slot, items: number): void {
  assertNumber(slot)

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
 * Apply a remove operation to a `String` slot
 */
export function applyRemoveString(
  node: Attr | Text,
  slot: Slot,
  items: number
): void {
  assertNumber(slot)

  const graphemes = toGraphemes(node.textContent ?? '')
  assert(
    slot >= 0 && slot <= graphemes.length,
    `Unexpected remove slot '${slot}' for text node of length ${graphemes.length}`
  )
  assert(
    items > 0 && slot + items <= graphemes.length,
    `Unexpected remove items ${items} for text node of length ${graphemes.length}`
  )

  node.textContent =
    graphemes.slice(0, slot).join('') + graphemes.slice(slot + items).join('')
}
