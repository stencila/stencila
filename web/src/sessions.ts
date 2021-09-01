import { SoftwareSession } from '@stencila/schema'
import { DocumentId, NodeUpdate } from './types'

declare function start(documentId: DocumentId): Promise<SoftwareSession>

declare function stop(documentId: DocumentId): Promise<void>

declare function subscribe(
  documentId: DocumentId,
  topics: string[],
  handler: (updates: NodeUpdate[]) => void
): void

declare function unsubscribe(documentId: DocumentId, topics: string[]): void

export const sessions = {
  start,
  stop,
  subscribe,
  unsubscribe,
}
