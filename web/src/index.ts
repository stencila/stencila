// To iron out:
// - race conditions for node execution responses
// - cancelling execution
// - handling stale status for downstream node dependencies
// - what does a `documents.change` payload look like for HTML nodes?
// - How do we migrate old published documents
// - Attach Node IDs for required elements in published article HTML

import { Document, Patch, Session } from '@stencila/stencila'
import { Client, ClientId, connect, disconnect } from './client'
import * as documents from './documents'
import * as sessions from './sessions'
import { ProjectId, SnapshotId } from './types'

export const main = (
  url: string,
  clientId: ClientId,
  projectId: ProjectId,
  snapshotId: SnapshotId,
  documentPath: documents.DocumentPath
): void => {
  let client: Client | undefined
  let session: Session | undefined
  let document: Document | undefined

  // Start the client and session, if necessary
  const startup = async (): Promise<[Client, Document, Session]> => {
    if (!client) {
      client = await connect(url, clientId)
    }

    if (!session) {
      session = await sessions.start(client, projectId, snapshotId)
      sessions.subscribe(client, session.id, 'updated').catch((err) => {
        console.warn(`Couldn't subscribe to session updates`, err)
      })

      sessions.subscribe(client, session.id, 'heartbeat').catch((err) => {
        console.warn(`Couldn't subscribe to session updates`, err)
      })
    }

    if (document === undefined) {
      document = await documents.open(client, documentPath)
      documents.subscribe(client, document.id, 'patched').catch((err) => {
        console.warn(`Couldn't subscribe to document 'patched'`, err)
      })
    }

    return [client, document, session]
  }

  // Shutdown the session, if necessary
  const shutdown = async (): Promise<void> => {
    if (client !== undefined && session !== undefined) {
      await sessions.stop(client, session.id)
      session = undefined
    }
  }

  // Listen for a `session:start` event. Currently, mainly
  // used for manual testing of events e.g. enter in the console
  // `window.dispatchEvent(new CustomEvent('session:start'))`
  window.addEventListener('session:start', () => {
    startup().catch((err) => {
      console.warn(`Couldn't start the session`, err)
    })
  })

  // Listen for a `document:execute` custom event e.g. user presses
  // a document level "run button". Note that in this case the node id
  // _is_ the document id.
  window.addEventListener('document:execute', async () => {
    const [client, document] = await startup()
    await documents.execute(client, document.id, document.id)
  })

  // Listen for a `document:node::execute` custom event e.g. user presses
  // a the "run button" on a `CodeChunk` (to execute it without changing it)
  window.addEventListener('document:node:execute', async (e) => {
    const [client, document] = await startup()
    const { nodeId, value } = (e as CustomEvent<documents.NodeExecute>).detail
    await documents.execute(client, document.id, nodeId, value)
  })

  // Listen for a `document:patched` custom event emitted from within browser window
  // e.g. user changes the code of a `CodeChunk`, or slides a numeric `Parameter`
  //
  // Creates a `Patch` (not a `DomPatch`) that is sent to the
  // server to apply the change. The reason we construct and send a `Patch`
  // is so that in the future we are able to send `Operation`s with an `address`
  // and `value` that refer to a specific property of the node.
  window.addEventListener('document:patch', async (e) => {
    const [client, document] = await startup()
    const { nodeId, value } = (e as CustomEvent<documents.NodePatched>).detail
    const patch: Patch = [
      {
        type: 'Replace',
        address: [],
        items: 1,
        length: 1,
        value,
      },
    ]
    await documents.patch(client, document.id, nodeId, patch)
  })

  // Listen for a `session:stop` custom event e.g. user presses
  // a document level "stop button".
  window.addEventListener('session:stop', () => {
    shutdown().catch((err) => {
      console.warn(`Couldn't shut down the session`, err)
    })
  })

  // Shutdown and disconnect on page unload
  window.addEventListener('unload', () => {
    if (client !== undefined) {
      shutdown().catch((err) => {
        console.warn(`Couldn't shut down the session`, err)
      })

      disconnect(client)
    }
  })
}
