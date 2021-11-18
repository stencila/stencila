import { OperationMove, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import { assert, assertElement, assertIndex, panic } from '../checks'
import { resolveParent } from './resolve'

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

  applyMoveVec(fromParent, fromSlot, toSlot, items)
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

  const children = vec.childNodes
  assert(
    items > 0 && from + items <= children.length,
    `Unexpected move items ${items} for element with ${children.length} children`
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
    vec.insertBefore(child, toChild)
    moved += 1
  }
}
