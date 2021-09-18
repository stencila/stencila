import { DomOperationRemove, Slot } from '@stencila/stencila'
import {
  assert,
  assertNumber,
  assertString,
  isElement,
  isString,
  resolveParent,
  resolveSlot,
} from './utils'

/**
 * Apply a remove operation
 */
export function applyRemove(op: DomOperationRemove): void {
  const { address, items } = op
  const [parent, slot] = resolveParent(address)

  if (isElement(parent)) {
    if (isString(slot)) applyRemoveOption(parent, slot, items)
    else applyRemoveVec(parent, slot, items)
  } else applyRemoveString(parent, slot, items)
}

/**
 * Apply a remove operation to an element representing an `Option`
 */
export function applyRemoveOption(
  node: Element,
  slot: Slot,
  items: number
): void {
  assertString(slot)
  assert(
    items === 1,
    `Unexpected remove items ${items} for option slot ${slot}`
  )

  const target = resolveSlot(node, slot)
  target.remove()
}

/**
 * Apply a remove operation to an element representing a `Vec`
 */
export function applyRemoveVec(node: Element, slot: Slot, items: number): void {
  assertNumber(slot)

  const children = node.childNodes
  assert(
    slot >= 0 && slot < children.length,
    `Unexpected remove slot ${slot} for element with ${children.length} children`
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
 * Apply a remove operation to a text node representing a `String`
 */
export function applyRemoveString(node: Text, slot: Slot, items: number): void {
  assertNumber(slot)

  const text = node.textContent ?? ''
  assert(
    slot >= 0 && slot <= text.length,
    `Unexpected remove slot ${slot} for text node of length ${text.length}`
  )
  assert(
    items > 0 && slot + items <= text.length,
    `Unexpected remove items ${items} for text node of length ${text.length}`
  )

  node.textContent = text.slice(0, slot) + text.slice(slot + items)
}
