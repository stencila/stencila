import { OperationMove, Slot, ElementId } from '../../types'
import {
  assert,
  assertElement,
  assertIndex,
  assertName,
  panic,
} from '../checks'
import {
  isArrayElement,
  isObjectElement,
  resolveObjectKey,
  resolveParent,
} from './resolve'

/**
 * Apply a move operation
 *
 * At the time of writing, `Move` operations are only generated
 * for `Vec`s so this panics if the operation is on a string
 * or has string terminal slots.
 */
export function applyMove(op: OperationMove, target?: ElementId): void {
  const { from, items, to } = op

  const [fromParent, fromSlot] = resolveParent(from, target)
  const [toParent, toSlot] = resolveParent(to, target)

  if (fromParent === undefined) {
    return console.warn(
      `Unable to resolve address '${from.join(
        ','
      )}'; 'Move' operation will be ignored'`
    )
  }
  if (toParent === undefined) {
    return console.warn(
      `Unable to resolve address '${to.join(
        ','
      )}'; 'Move' operation will be ignored'`
    )
  }

  assert(
    toParent.isSameNode(fromParent),
    'Expected the from and to addresses to have the same parent'
  )
  assertElement(fromParent)

  if (isObjectElement(fromParent))
    applyMoveObject(fromParent, fromSlot, toSlot, items)
  else if (isArrayElement(fromParent))
    applyMoveArray(fromParent, fromSlot, toSlot, items)
  else applyMoveVec(fromParent, fromSlot, toSlot, items)
}

/**
 * Apply a `Move` operation to an element representing an `Object`
 */
export function applyMoveObject(
  object: Element,
  from: Slot,
  to: Slot,
  items: number
): void {
  assertName(from)
  assertName(to)
  assert(items === 1, `Expected move items to be 1 for object`)

  const dl = object.querySelector('dl')
  assertElement(dl)

  const fromKey = resolveObjectKey(dl, from)
  assert(fromKey !== undefined, 'Unable to find from key for object move')
  assertElement(fromKey)

  const fromValue = fromKey?.nextElementSibling
  assertElement(fromValue)

  const toKey = resolveObjectKey(dl, to)
  if (toKey !== undefined) {
    // Move to where the to key is
    toKey.nextElementSibling?.replaceWith(fromValue)
  } else {
    // Just relabel the from key
    fromKey.textContent = to
  }
}

/**
 * Apply a `Move` operation to an element representing an `Array`
 */
export function applyMoveArray(
  array: Element,
  from: Slot,
  to: Slot,
  items: number
): void {
  assertIndex(from)
  assertIndex(to)

  const ol = array.querySelector('ol')
  assertElement(ol)

  moveChildren(ol, from, to, items)
}

/**
 * Apply a `Move` operation to an element representing a `Vec`
 */
export function applyMoveVec(
  vec: Element,
  from: Slot,
  to: Slot,
  items: number
): void {
  assertIndex(from)
  assertIndex(to)

  moveChildren(vec, from, to, items)
}

/**
 * Move children in an element
 */
function moveChildren(
  elem: Element,
  from: number,
  to: number,
  items: number
): void {
  const children = elem.childNodes
  assert(
    items > 0 && from + items <= children.length,
    `Unexpected move items ${items} for ${elem.tagName} element with ${children.length} children`
  )

  const toChild =
    to === children.length
      ? // Move to end
        null
      : from < to
      ? // Move forward
        children[to + 1]
      : // Move back
        children[to]
  if (toChild === undefined) throw panic(`Unexpected move to slot ${to}`)

  let moved = 0
  while (moved < items) {
    const child = children[from]
    if (child === undefined) {
      throw panic(`Unexpected move from slot ${from}`)
    }
    elem.insertBefore(child, toChild)
    moved += 1
  }
}
