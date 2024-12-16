import { NodeType } from '@stencila/types'

import type { DocumentAccess, DocumentId, NodeId } from '../types'

import { RestClient } from './rest'

/**
 * A command to send to a document
 *
 * Note that this must be consistent with the Rust enum
 * named `Command` in `rust/document/src/lib.rs`.
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
    | 'patch-node'
    | 'patch-node-format'
    | 'accept-node'
    | 'reject-node'
    | 'revise-node'
    | 'archive-node'

  /**
   * The arguments of the command
   *
   * If present, takes precedence over the other properties below.
   */
  args?: (string | number | boolean)[]

  /**
   * The type of the node that the command is being executed on.
   *
   * This is not of the Rust `Command` enum but is required for
   * compatibility with the LSP which uses the convention of prefixing
   * most command args with the document URI and the node type.
   */
  nodeType?: NodeType

  /**
   * Node ids, where applicable
   */
  nodeIds?: NodeId[]

  /**
   * Node property name and value (for `patch-node`)
   */
  nodeProperty?: [string, unknown]

  /**
   * The scope for the command
   */
  scope?: 'only' | 'plus-before' | 'plus-after' | 'plus-upstream-downstream'
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
