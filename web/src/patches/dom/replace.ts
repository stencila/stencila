import { OperationReplace, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
  assert,
  assertIndex,
  assertName,
  assertString,
  isAttr,
  isElement,
  isName,
  panic,
} from '../checks'
import { createFragment, resolveParent, resolveSlot } from './resolve'
import { applyReplace as applyReplaceString } from '../string'

/**
 * Apply a replace operation
 */
export function applyReplace(op: OperationReplace, target?: ElementId): void {
  const { address, items, html } = op
  assertString(html)

  const [parent, slot] = resolveParent(address, target)

  if (isElement(parent)) {
    if (isName(slot)) applyReplaceOption(parent, slot, items, html)
    else applyReplaceVec(parent, slot, items, html)
  } else applyReplaceText(parent, slot, items, html)
}

/**
 * Apply a replace operation to an `Option` slot
 */
export function applyReplaceOption(
  node: Element,
  slot: Slot,
  items: number,
  html: string
): void {
  assertName(slot)
  assert(
    items === 1,
    `Unexpected replace items ${items} for option slot '${slot}'`
  )

  const child = resolveSlot(node, slot)
  if (isElement(child)) child.outerHTML = html
  else if (isAttr(child)) {
    const fragment = createFragment(html)
    node.setAttribute(child.name, fragment.textContent ?? '')
  } else child.textContent = html
}

/**
 * Apply a replace operation to a `Vec` slot
 */
export function applyReplaceVec(
  node: Element,
  slot: Slot,
  items: number,
  html: string
): void {
  assertIndex(slot)

  const fragment = createFragment(html)
  const children = node.childNodes
  if (children.length === 0) {
    node.appendChild(fragment)
  } else {
    const child = children[slot]
    if (child === undefined) {
      throw panic(
        `Unexpected replace slot '${slot}' for element with ${children.length} children`
      )
    }
    node.insertBefore(fragment, child)

    let removed = 0
    while (removed < items) {
      children[slot + 1]?.remove()
      removed += 1
    }
  }
}

/**
 * Apply a `Replace` operation to a `Text` or `Attr` DOM node
 */
export function applyReplaceText(
  node: Attr | Text,
  slot: Slot,
  items: number,
  html: string
): void {
  node.textContent = applyReplaceString(
    node.textContent ?? '',
    slot,
    items,
    html
  )
}
