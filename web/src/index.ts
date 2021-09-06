// To iron out:
// - race conditions for node execution responses
// - cancelling execution
// - handling stale status for downstream node dependencies
// - what does a `documents.change` payload look like for HTML nodes?
// - How do we migrate old published documents
// - Attach Node IDs for required elements in published article HTML

import { Client, connect, disconnect } from './client'
import * as documents from './documents'
import { Document } from './documents'
import * as sessions from './sessions'
import { Session } from './sessions'
import { ProjectId, SnapshotId } from './types'

export const main = (
  projectId: ProjectId,
  snapshotId: SnapshotId,
  documentPath: documents.DocumentPath
) => {
  let client: Client | undefined
  let session: Session | undefined
  let document: Document | undefined

  // Start the client and session, if necessary
  const startup = async (): Promise<[Client, Document, Session]> => {
    if (client == undefined) {
      client = await connect()
    }

    if (session == undefined) {
      session = await sessions.start(client, projectId, snapshotId)
      sessions.subscribe(client, session.id, 'updated')
      sessions.subscribe(client, session.id, 'heartbeat')
    }

    if (document == undefined) {
      document = await documents.open(client, documentPath)
      documents.subscribe(client, document.id, 'node:value')
      documents.subscribe(client, document.id, 'node:html')
    }

    return [client, document, session]
  }

  // Shutdown the session, if necessary
  const shutdown = async () => {
    if (client !== undefined && session !== undefined) {
      await sessions.stop(client, session.id)
      session = undefined
    }
  }

  // Listen for a `document:execute` custom event e.g. user presses
  // a document level "run button".
  window.addEventListener('document:execute', async () => {
    const [client, document] = await startup()
    await documents.execute(client, document.id)
  })

  // Listen for a `document:node:changed` custom event emitted from within browser window
  // e.g. user changes the code of a `CodeChunk`, or slides a numeric `Parameter`
  window.addEventListener('document:node:changed', async (e) => {
    const [client, document] = await startup()
    const event = (e as CustomEvent).detail as documents.NodeValueChanged
    await documents.change(client, document.id, event.id, event.value)
  })

  // Listen for a `session:stop` custom event e.g. user presses
  // a document level "stop button".
  window.addEventListener('session:stop', () => {
    shutdown()
  })

  // Shutdown and disconnect on page unload
  window.addEventListener('unload', () => {
    if (client !== undefined) {
      shutdown()
      disconnect(client)
    }
  })
}
