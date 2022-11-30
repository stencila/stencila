import { EditorState, Transaction } from 'prosemirror-state'
import { Operation, Patch } from '../../types'
import { diff } from '../json'
import { prosemirrorToStencila } from './convert'

export function transactionToOps(
  transaction: Transaction,
  pre: EditorState,
  post: EditorState
): Patch | null {
  const patch = transaction.getMeta('stencila-document-patch')
  if (patch) {
    return patch
  } else if (patch === false) {
    return null
  } else if (transaction.steps.length > 0) {
    const patch = diffToPatch(pre, post)
    return patch.ops.length > 0 ? patch : null
  } else {
    return null
  }
}

export function diffToPatch(pre: EditorState, post: EditorState): Patch {
  console.info('ℹ️ Diffing states to derive patch operations')

  const preNode = prosemirrorToStencila(pre.doc.toJSON())
  const postNode = prosemirrorToStencila(post.doc.toJSON())
  //console.log(JSON.stringify(pre.doc.toJSON()))
  //console.log(JSON.stringify(post.doc.toJSON()))
  //console.log(JSON.stringify(preNode))
  //console.log(JSON.stringify(postNode))

  return diff(preNode, postNode)
}
