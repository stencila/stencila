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

interface Session {
  id: string
}

interface SessionEvent {
  type: 'Updated' | 'Heartbeat'
  session: Session
}


// Development test function to check for expected events etc
export const test = async (url: string, clientId: string, projectId: string, snapshotId: string) => {
  let sleep = (seconds: number) => new Promise((resolve) => { setTimeout(resolve, seconds * 1000) });

  const client = new Client(`ws://${window.location.host}${url}?client=${clientId}`)
  client.on('open', async () => {
    // Start a session
    let session = await client.call('sessions.start', { project: projectId, snapshot: snapshotId }) as Session
    console.debug(session)

    await sleep(10);

    // Subscribe to the session
    session = await client.call('sessions.subscribe', { session: session.id }) as Session
    console.debug(session)

    // Handlers for session events that we just subscribed to
    client.on(`sessions:${session.id}:updated`, (event: SessionEvent) => console.log(event))
    client.on(`sessions:${session.id}:heartbeat`, (event: SessionEvent) => console.log(event))

    await sleep(30);

    // Unsubscribe from the session
    session = await client.call('sessions.unsubscribe', { session: session.id }) as Session
    console.debug(session)

    await sleep(10);

    // Stop the session
    session = await client.call('sessions.stop', { session: session.id }) as Session
    console.debug(session)

    // Close the client
    client.close()
  })
}
