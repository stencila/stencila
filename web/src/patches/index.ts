/**
 * This module implements DOM-based analogues of several functions in `../rust/src/patches.rs`.
 *
 * Compared to the Rust functions which apply a `Patch` to a Stencila `Node` (e.g. an `Article`
 * or `String`), these functions apply a `DomPatch` to a DOM `Node` (e.g. an `Element` or `Text`).
 *
 * In the Rust `patch` module most of the action in applying operations occurs in the `Patchable`
 * implementations for `Vec` (vectors), `Option` (usually optional properties of `struct`s), and
 * `String`s. To promote consistency in the implementations we use those names in functions
 * in this module.
 */

import { DomOperation, DomPatch } from '@stencila/stencila'
import { ElementId } from '../types'
import { applyAdd } from './add'
import { applyMove } from './move'
import { applyRemove } from './remove'
import { applyReplace } from './replace'
import { applyTransform } from './transform'

/**
 * Apply a `DomPatch` to the `root` node of the current document
 */
export function applyPatch(patch: DomPatch): void {
  const { ops, target } = patch
  for (const op of ops) {
    applyOp(op, target)
  }
}

/**
 * Apply a `DomOperation` to the `root` node
 *
 * Uses the `type` discriminant to dispatch to specific functions for
 * each operation variant.
 */
export function applyOp(op: DomOperation, target?: ElementId): void {
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
