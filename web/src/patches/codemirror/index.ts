/**
 * Translate CodeMirror "changes" into Stencila `Operation`s and vice verse.
 */

import { ChangeSet, ChangeSpec, EditorState } from '@codemirror/state'
import { Address, Operation } from '@stencila/stencila'
import { assertNumber } from '../checks'
import { diff } from '../string'

/**
 * Translate a CodeMirror `EditorState` into a set of Stencila `Operation`s
 *
 * A brute force approach that produces a single replace operation.
 * Prefer `diffToOps` or `changesToOps` (although this could be useful as a
 * fallback to ensure the content is synchronized).
 */
export function stateToOps(state: EditorState, address: Address): Operation[] {
  const lines = state.doc.toJSON()
  const value = lines.join('\n')
  const length = value.length
  return [
    {
      type: 'Replace',
      address,
      value,
      items: 1,
      length,
    },
  ]
}

/**
 * Translate pre- and post- CodeMirror `EditorState`s into a set of Stencila `Operation`s
 */
export function diffToOps(
  pre: EditorState,
  post: EditorState,
  address: Address
): Operation[] {
  return diff(
    pre.doc.toJSON().join('\n'),
    post.doc.toJSON().join('\n'),
    address
  ).ops
}

/**
 * Translate a CodeMirror `ChangeSet`s to a set of Stencila `Operation`s
 *
 * This will generate invalid `address`, `items` and `length` fields in operations
 * if there are Unicode graphemes in the content. That is because CodeMirror uses
 * character indices whereas Stencila uses grapheme indices.
 */
export function changesToOps(
  changes: ChangeSet,
  address: Address
): Operation[] {
  let ops: Operation[] = []
  changes.iterChanges((fromA, toA, fromB, toB, inserted) => {
    const lines = inserted.toJSON()
    const value = lines.join('\n')
    // console.log(fromA, toA, fromB, toB, value)
    ops.push(changeToOp(fromA, toA, value, address))
  })
  return ops
}

/**
 * Translate a CodeMirror change to a Stencila `Operation`
 */
export function changeToOp(
  from: number,
  to: number,
  value: string,
  address: Address
): Operation {
  if (value == '') {
    return {
      type: 'Remove',
      address: [...address, from],
      items: to - from,
    }
  }

  if (from == to) {
    return {
      type: 'Add',
      address: [...address, from],
      value,
      length: value.length,
    }
  }

  return {
    type: 'Replace',
    address: [...address, from],
    items: to - from,
    value,
    length: value.length,
  }
}

/**
 * Translate a set of Stencila `Operation`s to a set of CodeMirror `ChangeSpec`s
 *
 * The resulting set of changes can be send to a CodeMirror editor view e.g.
 *
 *  editorView.dispatch({
 *     changes: [patchToChanges(op)],
 *   })
 */
export function opsToChanges(ops: Operation[]): ChangeSpec[] {
  return ops.map(opToChange)
}

/**
 * Translate a Stencila `Operation` to a CodeMirror `ChangeSpec`
 */
function opToChange(op: Operation): ChangeSpec {
  switch (op.type) {
    case 'Add':
    case 'Remove':
    case 'Replace': {
      const from = op.address[1]
      assertNumber(from)

      switch (op.type) {
        case 'Add':
          return {
            from,
            insert: op.value,
          }
        case 'Remove':
          return {
            from,
            to: from + op.items,
          }
        case 'Replace':
          return {
            from,
            to: from + op.items,
            insert: op.value,
          }
      }
    }
    default:
      throw new Error(`Unhandled operation type '${op.type}'`)
  }
}
