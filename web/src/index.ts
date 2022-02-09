import { Document, Session } from '@stencila/stencila'
import { Client, ClientId, connect, disconnect } from './client'
import * as documents from './documents'
import { onDiscoverExecutableLanguages } from './events/kernels'
import { languages } from './kernels'
import * as sessions from './sessions'
import { ProjectId, SnapshotId } from './types'

export type { Document, Session } from '@stencila/stencila'
export * as documents from './documents'

export const main = (
  clientId: ClientId,
  projectId: ProjectId,
  snapshotId: SnapshotId,
  documentPath: documents.DocumentPath,
  origin?: string | null,
  token?: string | null
): (() => Promise<[Client, Document, Session]>) => {
  let client: Client | undefined
  let session: Session | undefined
  let document: Document | undefined

  // Start the client and session, if necessary
  // Returns early if already started up
  const startup = async (): Promise<[Client, Document, Session]> => {
    if (
      client !== undefined &&
      session !== undefined &&
      document !== undefined
    ) {
      return [client, document, session]
    }

    if (client === undefined) {
      client = await connect(projectId, clientId, origin, token)
    }

    if (session === undefined) {
      session = await sessions.start(client, projectId, snapshotId)
      sessions.subscribe(client, session.id, 'updated').catch((err) => {
        console.warn(`Couldn't subscribe to session updates`, err)
      })

      const kernels = await languages(client, session.id)
      // Inform components of the available kernels in this session
      // by emitting a custom event
      onDiscoverExecutableLanguages(kernels)

      // Don't subscribe to heartbeats during development because it generates
      // distracting WebSocket messages
      // if (process.env.NODE_ENV !== 'development') {
      //  sessions.subscribe(client, session.id, 'heartbeat').catch((err) => {
      //    console.warn(`Couldn't subscribe to session updates`, err)
      //  })
      // }
    }

    if (document === undefined) {
      document = await documents.open(client, documentPath)
      documents
        .subscribe(client, document.id, 'patched', (event) =>
          documents.receivePatch(clientId, event)
        )
        .catch((err) => {
          console.warn(`Couldn't subscribe to document 'patched'`, err)
        })
    }

    documents.listen(client, clientId, document.id)

    return [client, document, session]
  }

  // Shutdown the session, if necessary
  const shutdown = async (): Promise<void> => {
    if (client !== undefined && session !== undefined) {
      await sessions.stop(client, session.id)
      session = undefined
    }
  }

  // Shutdown and disconnect on page unload
  window.addEventListener('unload', () => {
    if (client !== undefined) {
      shutdown().catch((err) => {
        console.warn(`Couldn't shut down the session`, err)
      })

      disconnect(client)
    }
  })

  return startup
}
