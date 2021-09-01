import { DocumentPath, DocumentId, JSONValue } from './types'

// NOTE: path is relative to the project root
declare function open(path: DocumentPath, format?: string): Promise<void>

declare function patch(
  path: DocumentPath,
  nodeId: string,
  payload: JSONValue
): Promise<void>

// declare function load(docId: DocumentId, contents: string, format?: string): Promise<void>

// declare function get(docId: DocumentId): Promise<void>

// declare function preview(docId: DocumentId): Promise<void>

declare function subscribe({
  documentId,
  topics,
}: {
  documentId: DocumentId
  topics: string[]
}): Promise<void>

declare function unsubscribe({
  documentId,
  topics,
}: {
  documentId: DocumentId
  topics: string[]
}): Promise<void>

export const documents = {
  open,
  // load,
  // get,
  // preview,
  subscribe,
  unsubscribe,
}
