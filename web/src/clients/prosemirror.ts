import { EditorState, Transaction } from 'prosemirror-state'
import { Step } from 'prosemirror-transform'
import { EditorView } from 'prosemirror-view'

import { type DocumentAccess, type DocumentId } from '../types'

import { NodePatch, NodesClient } from './nodes'

/**
 * A write-only client that keeps a ProseMirror editor synchronized with
 * the root node of a document.
 */
export class ProseMirrorClient extends NodesClient {
  /**
   * Construct a new `ProseMirrorClient`
   *
   * @param id The id of the document
   * @param access The access level of the client
   * @param elem The element to listen to for node patches (e.g. from
   *             descendent Web Components)
   */
  constructor(id: DocumentId, access: DocumentAccess, elem: HTMLElement) {
    super(id, access, elem, 'prosemirror')
  }

  /**
   * Create a function to forward ProseMirror transactions to the server
   *
   * @returns A function that can be used for the ProseMirror
   *          `dispatchTransaction` constructor option
   */
  public sendPatches(): (transaction: Transaction) => void {
    return function (transaction: Transaction) {
      // This function is called with a ProseMirror `EditorView` as `this`
      // and must update the state with the transaction (in addition to
      // forwarding the transformed transaction to the server)
      const view = this as EditorView

      // Apply the transaction to the state to get a new state
      const newState = view.state.apply(transaction)

      // Generate a patch from the transaction and send to the server
      const patch = ProseMirrorClient.transactionToPatch(
        transaction,
        view.state,
        newState
      )
      if (patch) this.sendMessage(patch)

      // Update this view with the new state
      this.updateState(newState)
    }
  }

  /**
   * Transform a ProseMirror transaction into a `NodePatch`
   *
   * If the transaction contains no `steps` (e.g. for a change in selection)
   * will return null. Otherwise, generates a patch using different approaches
   * based on the type of steps.
   */
  private static transactionToPatch(
    transaction: Transaction,
    pre: EditorState,
    post: EditorState
  ): NodePatch | null {
    const steps = transaction.steps

    if (steps.length === 0) {
      return null
    }

    if (steps.length === 1) {
      const step = steps[0]
      const stepType = step.constructor.prototype.jsonID
      if (stepType === 'replace') {
        return ProseMirrorClient.textPatch(step)
      }
    }

    return ProseMirrorClient.diffPatch(pre, post)
  }

  /**
   * Transform a ProseMirror text editing step into a node patch
   *
   * TODO: Re-implement, this a placeholder only
   *
   * @param step The ProseMirror step to transform
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  private static textPatch(step: Step): NodePatch | null {
    return null
  }

  /**
   * Generate a node patch by diffing two editor states
   *
   * This is intended as a fallback for when more performant custom
   * approaches have not yet been implemented for a particular type
   * of ProseMirror transaction.
   *
   * TODO: Re-implement, this a placeholder only
   *
   * @see Previous implementation
   *      Convert PM nodes to Stencila nodes: https://github.com/stencila/stencila/blob/v1/web/src/patches/prosemirror/convert.ts
   *      Diff the Stencila nodes to a patch: https://github.com/stencila/stencila/blob/v1/web/src/patches/json/index.ts
   *
   * @param pre The editor state prior to the transaction
   * @param post The editor state after the transaction
   */
  private static diffPatch(
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    pre: EditorState,
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    post: EditorState
  ): NodePatch | null {
    if (process.env.NODE_ENV === 'development') {
      console.log('🔀 Diffing editor states to derive patch')
    }

    return null
  }
}
