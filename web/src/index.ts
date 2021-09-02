import { Client } from 'rpc-websockets'

// To iron out:
// - race conditions for node execution responses
// - cancelling execution
// - handling stale status for downstream node dependencies
// - what does a `documents.change` payload look like for HTML nodes?
// - How do we migrate old published documents
// - Attach Node IDs for required elements in published article HTML

// Start of session, and connection to session, should be done over a single
// WebSocket connection.
export const main = (url: string) => {
  const ws = new Client(url)

  ws.on('open', async () => {
    // TODO: investigate logging general WS messages
    // ws.subscribe('message')
    // ws.on('message', (msg) => console.log('msg received: ', msg))
  })

  return ws
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
