import {client} from './client'
import { DocumentId } from "./types"

// To iron out:
// - race conditions for node execution responses
// - cancelling execution
// - handling stale status for downstream node dependencies
// - what does a `documents.change` payload look like for HTML nodes?
// - How do we migrate old published documents

// Start of session, and connection to session, should be done over a single
// WebSocket connection.
const main = (projectId: string) => (documentPath: DocumentId) => {
  // WebSocket connection instance?
  // const client = new Client()

  // client.sessions.start(projectId) -> sessionId

  // client.sessions.subscribe(sessionId)

  // client.documents.open(documentPath) -> documentId
  // client.documents.subscribe(documentId, ['update/patch/?'], handlerFn)
  // client.documents.change/patch(documentId, nodeInDocument, UpdatableNodeJSON)
    // receive subscribe event once done


  // client.documents.open(documentPath)
  // client.sessions.start(documentPath)
}
