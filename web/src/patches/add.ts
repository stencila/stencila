import { DomOperationAdd, Slot } from '@stencila/stencila'
import {
  assert,
  assertNumber,
  assertString,
  createFragment,
  isElement,
  isString,
  panic,
  resolveParent,
} from './utils'

/**
 * Apply an add operation
 */
export function applyAdd(op: DomOperationAdd): void {
  const { address, html } = op
  const [parent, slot] = resolveParent(address)

  if (isElement(parent)) {
    if (isString(slot)) applyAddOption(parent, slot, html)
    else applyAddVec(parent, slot, html)
  } else applyAddString(parent, slot, html)
}

/**
 * Apply an add operation to an element representing an `Option`
 */
export function applyAddOption(node: Element, slot: Slot, html: string): void {
  assertString(slot)

  const fragment = createFragment(html)
  node.appendChild(fragment)
}

/**
 * Apply an add operation to an element representing a `Vec`
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
        `Unexpected add slot ${slot} for element with ${children.length} children`
      )
    node.insertBefore(fragment, sibling)
  }
}

/**
 * Apply an add operation to a text node representing a `String`
 */
export function applyAddString(node: Text, slot: Slot, value: string): void {
  assertNumber(slot)

  const text = node.textContent ?? ''
  assert(
    slot >= 0 && slot <= text.length,
    `Unexpected add slot ${slot} for text node of length ${text.length}`
  )
  node.textContent = text.slice(0, slot) + value + text.slice(slot)
}
