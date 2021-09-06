import { Client } from './client'

export type DocumentPath = string

export type DocumentId = string

export interface Document {
  id: DocumentId
}

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
      html: String
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
 * Loads the document into memory and returns a document object
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
 * Default handler for document events
 *
 * Dispatches a `CustomEvent` with the type of the event
 * prefixed with "document:" e.g. "document:nodevalueupdated"
 */
function defaultHandler(event: DocumentEvent) {
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
    document: documentId,
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
    document: documentId,
    topic,
  }) as Promise<Document>
}

/**
 * Execute a document
 */
export async function execute(
  client: Client,
  documentId: DocumentId
): Promise<Document> {
  return client.call('documents.execute', {
    document: documentId,
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
    document: documentId,
    node: nodeId,
    value: nodeValue,
  }) as Promise<Document>
}
