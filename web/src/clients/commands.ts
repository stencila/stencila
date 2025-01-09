import { ExecutionMode, File, InstructionType, NodeType } from '@stencila/types'

import type { DocumentAccess, DocumentId, NodeId } from '../types'

import { RestClient } from './rest'

export interface DocumentCommand {
  command: string
  args: Array<null | boolean | number | string | object>
}

/**
 * The name of the `CustomEvent` for document commands emitted from the browser DOM
 */
const DOCUMENT_COMMAND_EVENT = 'stencila-document-command'

/**
 * Create a `CustomEvent` containing a `DocumentCommand`
 */
const documentCommandEvent = (command: DocumentCommand): CustomEvent =>
  new CustomEvent(DOCUMENT_COMMAND_EVENT, {
    detail: command,
    bubbles: true,
    composed: true,
  })

/**
 * Create a `patch-value` command event
 */
export const patchValue = (
  nodeType: NodeType,
  nodeId: NodeId,
  patchPath: string | number | Array<string | number>,
  value: null | boolean | number | string
) =>
  documentCommandEvent({
    command: 'patch-value',
    args: [nodeType, nodeId, patchPath, value],
  })

/**
 * Create a `patch-clone` command event
 */
export const patchClone = (
  nodeType: NodeType,
  nodeId: NodeId,
  patchPath: string | number | Array<string | number>,
  cloneId: NodeId
) =>
  documentCommandEvent({
    command: 'patch-clone',
    args: [nodeType, nodeId, patchPath, cloneId],
  })

/**
 * Create a `patch-chat-focus` command event
 */
export const patchChatFocus = (chatId: NodeId, cloneId: NodeId) =>
  documentCommandEvent({
    command: 'patch-chat-focus',
    args: ['Chat', chatId, ['suggestions'], cloneId],
  })

/**
 * Create an `invoke.insert-clones` command event
 */
export const insertClones = (nodeIds: NodeId[]) =>
  documentCommandEvent({
    command: 'invoke.insert-clones',
    args: [[nodeIds]],
  })

/**
 * Create an `invoke.insert-instruction` command event
 */
export const insertInstructions = (
  nodeIds: NodeId[],
  instructionType: InstructionType,
  executionMode: ExecutionMode
) =>
  documentCommandEvent({
    command: 'invoke.insert-instruction',
    args: [nodeIds, instructionType, executionMode],
  })

/**
 * Create a `run-node` command event
 */
export const runNode = (
  nodeType: NodeType,
  nodeId: NodeId,
  scope?: 'plus-before' | 'plus-after'
) =>
  documentCommandEvent({
    command: 'run-node',
    args: [nodeType, nodeId, scope],
  })

/**
 * Create a `run-chat` command event
 */
export const runChat = (nodeId: NodeId, text: string, files: Array<File>) =>
  documentCommandEvent({
    command: 'run-chat',
    args: [nodeId, text, files],
  })

/**
 * Create an `archive-node` command event
 */
export const archiveNode = (nodeType: NodeType, nodeId: NodeId) =>
  documentCommandEvent({
    command: 'archive-node',
    args: [nodeType, nodeId],
  })

/**
 * Create a `revise-node` command event
 */
export const reviseNode = (
  nodeType: NodeType,
  nodeId: NodeId,
  feedback?: string
) =>
  documentCommandEvent({
    command: 'revise-node',
    args: [nodeType, nodeId, feedback],
  })

export class CommandsClient extends RestClient {
  /**
   * Create a new `CommandsClient`
   *
   * @param docId The id of the document that will be sent commands
   * @param elem The HTML element from which events will be forwarded
   */
  constructor(docId: DocumentId, _access: DocumentAccess, elem: HTMLElement) {
    super()

    elem.addEventListener(
      DOCUMENT_COMMAND_EVENT,
      async (event: CustomEvent) => {
        await RestClient.documentCommand(docId, event.detail)
      }
    )
  }
}
