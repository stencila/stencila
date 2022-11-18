import { EditorState, Transaction } from 'prosemirror-state'
import { Operation } from '../../types'
import { diff } from '../json'
import { prosemirrorToStencila } from './convert'

export function transactionToOps(
  transaction: Transaction,
  pre: EditorState,
  post: EditorState
): Operation[] {
  if (transaction.steps.length > 0) {
    return diffToOps(pre, post)
  } else {
    return []
  }
}

export function diffToOps(pre: EditorState, post: EditorState): Operation[] {
  const preNode = prosemirrorToStencila(pre.doc.toJSON())
  const postNode = prosemirrorToStencila(post.doc.toJSON())
  console.log(JSON.stringify(pre.doc.toJSON()))
  console.log(JSON.stringify(post.doc.toJSON()))
  console.log(JSON.stringify(preNode))
  console.log(JSON.stringify(postNode))
  const patch = diff(preNode, postNode)
  return patch.ops
}
