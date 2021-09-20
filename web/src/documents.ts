import { Document, DocumentEvent, DomPatch, Patch } from '@stencila/stencila'
import { Client } from './client'
import { applyPatch } from './patches'

export type DocumentPath = string

export type DocumentId = string

type DocumentTopic = 'patched'

export type NodeId = string

export type NodeValue =
  | string
  | number
  | boolean
  | null
  | NodeValue[]
  | { [key: string | number]: NodeValue }

/**
 * The browser `CustomEvent` detail emitted when a node in the current
 * document is executed.
 */
export interface NodeExecute {
  nodeId: NodeId
  patch?: Patch
}

/**
 * The browser `CustomEvent` detail emitted when a node in the current
 * document is changed by the user.
 */
export interface NodePatch {
  nodeId: NodeId
  patch: Patch
}

/**
 * Open a document
 *
 * Loads the document into memory on the server and returns a document object
 * with an `id` which can be used to subscribe to topics.
 */
export async function open(
  client: Client,
  documentPath: DocumentPath
): Promise<Document> {
  return client.call('documents.open', {
    path: documentPath,
  }) as Promise<Document>
}

/**
 * Close a document
 */
export async function close(
  client: Client,
  documentId: DocumentId
): Promise<Document> {
  return client.call('documents.close', {
    documentId,
  }) as Promise<Document>
}

/**
 * Default handler for document events
 */
function defaultHandler(event: DocumentEvent): void {
  if (event.type === 'patched') {
    applyPatch(event.patch as DomPatch)
  }
}

/**
 * Subscribe to a document topic
 */
export async function subscribe(
  client: Client,
  documentId: DocumentId,
  topic: DocumentTopic,
  handler: (event: DocumentEvent) => void = defaultHandler
): Promise<Document> {
  client.on(`documents:${documentId}:${topic}`, handler)
  return client.call('documents.subscribe', {
    documentId,
    topic,
  }) as Promise<Document>
}

/**
 * Unsubscribe from a document topic
 */
export async function unsubscribe(
  client: Client,
  documentId: DocumentId,
  topic: DocumentTopic
): Promise<Document> {
  client.off(`documents:${documentId}:${topic}`)
  return client.call('documents.unsubscribe', {
    documentId,
    topic,
  }) as Promise<Document>
}

/**
 * Patch a document node
 *
 * Will generate an error if the patch could not be
 * applied e.g. no node with the id could be found or
 * the patch was inconsistent with the node.
 */
export async function patch(
  client: Client,
  documentId: DocumentId,
  nodeId: NodeId,
  patch: Patch
): Promise<void> {
  return client.call('documents.patch', {
    documentId,
    nodeId,
    patch,
  }) as Promise<void>
}

/**
 * Execute a document node
 *
 * Optionally, pass a patch to apply to the node
 * prior to executing it.
 */
export async function execute(
  client: Client,
  documentId: DocumentId,
  nodeId: NodeId,
  patch?: Patch
): Promise<void> {
  return client.call('documents.execute', {
    documentId,
    nodeId,
    patch,
  }) as Promise<void>
}
