import type { DocumentAccess, DocumentId, NodeId } from '../types'

import { RestClient } from './rest'

/**
 * A command to send to a document
 */
export interface DocumentCommand {
  /**
   * The name of the command
   */
  command:
    | 'save-document'
    | 'execute-document'
    | 'execute-nodes'
    | 'interrupt-document'
    | 'interrupt-nodes'

  /**
   * Node ids, where applicable
   */
  nodeIds?: NodeId[]

  /**
   * The scope for the command
   */
  scope?: 'only' | 'plus-after' | 'plus-upstream-downstream'
}

/**
 * The name of the `CustomEvent` for document commands emitted from the browser DOM
 */
const DOCUMENT_COMMAND_EVENT = 'stencila-document-command'

/**
 * Create a `CustomEvent` containing a `DocumentCommand`
 */
export function documentCommandEvent(command: DocumentCommand): CustomEvent {
  return new CustomEvent(DOCUMENT_COMMAND_EVENT, {
    detail: command,
    bubbles: true,
    composed: true,
  })
}

export class CommandsClient extends RestClient {
  /**
   * Create a new `CommandsClient`
   *
   * @param docId The id of the document that will be sent commands
   * @param elem The HTML element from which events will be forwarded
   */
  // @ts-expect-error because access is not used yet
  constructor(docId: DocumentId, access: DocumentAccess, elem: HTMLElement) {
    super()

    elem.addEventListener(
      DOCUMENT_COMMAND_EVENT,
      async (event: CustomEvent) => {
        await RestClient.documentCommand(docId, event.detail)
      }
    )
  }
}
