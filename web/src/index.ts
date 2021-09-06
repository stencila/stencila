// To iron out:
// - race conditions for node execution responses
// - cancelling execution
// - handling stale status for downstream node dependencies
// - what does a `documents.change` payload look like for HTML nodes?
// - How do we migrate old published documents
// - Attach Node IDs for required elements in published article HTML

import { sessions } from "./sessions"
import {Client, connect, disconnect} from "./client"
import { DocumentId, ProjectId, Session, SnapshotId } from "./types"

export const main = (projectId: ProjectId, snapshotId: SnapshotId, documentPath: DocumentId) => {
  let client: Client | undefined
  let session: Session | undefined

  window.addEventListener('session:start', async () => {
    client = client ?? await connect()
    session = await sessions.start(client, projectId, snapshotId)
    sessions.subscribe(client, session.id, 'updated')
    sessions.subscribe(client, session.id, 'heartbeat')
  })

  window.addEventListener('session:stop', () => {
    if (client !== undefined && session !== undefined) {
      sessions.stop(client, session.id)
    }
  })
  
  window.addEventListener('unload', () => {
    if (client !== undefined) {
      if (session !== undefined) {
        sessions.stop(client, session.id)
      }
      disconnect(client)
    }
  })
}
