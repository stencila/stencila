import { Document } from '@stencila/stencila'
import { Client } from './client'

export type DocumentPath = string

export type DocumentId = string

type DocumentTopic = 'node:value' | 'node:html'

export type NodeId = string

export type NodeValue =
  | string
  | number
  | boolean
  | null
  | NodeValue[]
  | { [key: string | number]: NodeValue }

/**
 * A document event published by the server indicating
 * that a node has been updated
 */
export type DocumentEvent =
  | {
      type: 'NodeValueUpdated'
      documentId: string
      nodeId: string
      value: NodeValue
    }
  | {
      type: 'NodeHtmlUpdated'
      documentId: string
      nodeId: string
      html: string
    }

/**
 * The browser `CustomEvent` detail emitted when a node in the current
 * document is executed.
 */
export interface NodeExecute {
  id: NodeId
  value?: NodeValue
}

/**
 * The browser `CustomEvent` detail emitted when a node in the current
 * document is changed by the user.
 */
export interface NodeValueChanged {
  id: NodeId
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
    id: documentId,
  }) as Promise<Document>
}

/**
 * Default handler for document events
 *
 * Dispatches a `CustomEvent` with the type of the event
 * prefixed with "document:" e.g. "document:nodevalueupdated"
 */
function defaultHandler(event: DocumentEvent): void {
  window.dispatchEvent(
    new CustomEvent(`document:${event.type}`.toLowerCase(), { detail: event })
  )
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
  client.on(`document:${documentId}:${topic}`, handler)
  return client.call('documents.subscribe', {
    id: documentId,
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
  client.off(`document:${documentId}:${topic}`)
  return client.call('documents.unsubscribe', {
    id: documentId,
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
    id: documentId,
    node: nodeId,
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
    id: documentId,
    node: nodeId,
    value: nodeValue,
  }) as Promise<Document>
}
