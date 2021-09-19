import { Document, DocumentEvent, DomPatch } from '@stencila/stencila'
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
  value?: NodeValue
}

/**
 * The browser `CustomEvent` detail emitted when a node in the current
 * document is changed by the user.
 */
export interface NodeValueChanged {
  nodeId: NodeId
  value: NodeValue
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
 * Change a document node
 */
export async function change(
  client: Client,
  documentId: DocumentId,
  nodeId: NodeId,
  nodeValue: NodeValue
): Promise<Document> {
  return client.call('documents.change', {
    documentId,
    nodeId,
    value: nodeValue,
  }) as Promise<Document>
}

/**
 * Execute a document node
 *
 * Optionally, pass a new value for the node.
 */
export async function execute(
  client: Client,
  documentId: DocumentId,
  nodeId: NodeId,
  nodeValue?: NodeValue
): Promise<Document> {
  return client.call('documents.execute', {
    documentId,
    nodeId,
    value: nodeValue,
  }) as Promise<Document>
}
