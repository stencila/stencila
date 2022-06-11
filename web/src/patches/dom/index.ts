/**
 * This module implements DOM-based analogues of several functions in `../rust/src/patches.rs`.
 *
 * Compared to the Rust functions which apply a `Patch` to a Stencila `Node` (e.g. an `Article`
 * or `String`), these functions apply a `Patch` to a DOM `Node` (e.g. an `Element` or `Text`).
 *
 * In the Rust `patch` module most of the action in applying operations occurs in the `Patchable`
 * implementations for `Vec` (vectors), `Option` (usually optional properties of `struct`s), and
 * `String`s. To promote consistency in the implementations we use those names in functions
 * in this module.
 */

import { Operation, Patch, ElementId } from '../../types'
import { assertNumber } from '../checks'
import { applyAdd } from './add'
import { applyMove } from './move'
import { applyRemove } from './remove'
import { applyReplace } from './replace'
import { applyTransform } from './transform'

/**
 * Apply a `Patch` to the `root` node of the current document
 */
export function applyPatch(patch: Patch): void {
  const { ops, target } = patch
  for (let op of ops) {
    op = convertOp(op)
    applyOp(op, target)
  }
}

/**
 * Convert an `Operation` before applying it
 *
 * Some path operations need to be convert into a different type of
 * operation. This is because there is not always a direct mapping between the
 * structure of a document node and it's HTML representation. For example,
 * a `Heading.depth` is not represented as a DOM element or attribute but
 * as the tag name e.g. `h1` or `h2`. Another example is `TableCell.cellType` which
 * is represented as a `td` or `th`.
 *
 * This function intercepts operations on such properties and converts them into
 * `Transform` operations.
 */
export function convertOp(op: Operation): Operation {
  const type = op.type
  const slot =
    type === 'Add' || type === 'Replace' || type === 'Remove'
      ? op.address[op.address.length - 1]
      : undefined

  // Replace `Heading.depth` (required property so will never be added or removed)
  // Note addition of one to depth, consistent with HTML encoding
  if (slot === 'depth' && type === 'Replace') {
    assertNumber(op.value)
    return {
      type: 'Transform',
      address: op.address.slice(0, -1),
      from: '',
      to: `h${op.value + 1}`,
    }
  }

  // Add, remove, replace `TableCell.cell_type`
  if (slot === 'cellType') {
    if (type === 'Add' || type === 'Replace') {
      return {
        type: 'Transform',
        address: op.address.slice(0, -1),
        from: '',
        to: op.value === 'Header' ? 'th' : 'td',
      }
    } else if (type === 'Remove') {
      return {
        type: 'Transform',
        address: op.address.slice(0, -1),
        from: '',
        to: 'td',
      }
    }
  }

  return op
}

/**
 * Apply an `Operation` to the `root` node
 *
 * Uses the `type` discriminant to dispatch to specific functions for
 * each operation variant.
 */
export function applyOp(op: Operation, target?: ElementId): void {
  switch (op.type) {
    case 'Add':
      return applyAdd(op, target)
    case 'Remove':
      return applyRemove(op, target)
    case 'Replace':
      return applyReplace(op, target)
    case 'Move':
      return applyMove(op, target)
    case 'Transform':
      return applyTransform(op, target)
  }
}
