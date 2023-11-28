import { Transaction } from "prosemirror-state";
import { EditorView } from "prosemirror-view";

import { Capability } from "../capability";
import { DocumentId } from "../ids";

import { NodesClient } from "./nodes";

/**
 * A client for a ProseMirror editor
 */
export class ProseMirrorClient extends NodesClient {
  constructor(docId: DocumentId, capability: Capability, elem: HTMLElement) {
    super(docId, capability, elem);
  }

  /**
   * Create a function to forward ProseMirror transactions to the server
   *
   * @returns A function that can be used for the ProseMirror `dispatchTransaction` option
   */
  sendPatches(): (transaction: Transaction) => void {
    return function (transaction: Transaction) {
      // This function is called with a ProseMirror `EditorView` as `this`
      // and must update the state with the transaction (in addition to
      // forwarding the transformed transaction to the server)
      const view = this as EditorView;

      // Apply the transaction to the state to get a new state
      const newState = view.state.apply(transaction);

      // Generate a patch and send to the server via the window.stencilaClient
      // via a custom event
      //const patch = transactionToPatch(transaction, view.state, newState);

      // Update this view with the new state
      this.updateState(newState);
    };
  }
}
