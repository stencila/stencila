import { Document, Session } from '@stencila/stencila'
import { Client, ClientId, ClientOptions, connect, disconnect } from './client'
import * as documents from './documents'
import { onDiscoverExecutableLanguages } from './events/kernels'
import { languages } from './kernels'
import * as sessions from './sessions'
import { ProjectId } from './types'

export type { Document, Patch, Session } from '@stencila/stencila'
export * as client from './client'
export type { Client } from './client'
export * as documents from './documents'
export * as patches from './patches'
export * as utils from './utils'

export const main = (
  clientId: ClientId,
  projectId: ProjectId,
  documentPath?: documents.DocumentPath,
  origin?: string | null,
  token?: string | null,
  clientOptions?: ClientOptions
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
      client = await connect(projectId, clientId, origin, token, clientOptions)
    }

    if (session === undefined) {
      session = await sessions.start(client, projectId)
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

    if (documentPath !== undefined && document === undefined) {
      document = await documents.open(client, documentPath)
      documents
        .subscribe(client, document.id, 'patched', (event) =>
          documents.receivePatch(clientId, event)
        )
        .catch((err) => {
          console.warn(`Couldn't subscribe to document 'patched'`, err)
        })
    }

    if (document !== undefined) {
      documents.listen(client, clientId, document.id)
    }

    // TODO: Remove this after refactoring the entries points for this module
    // @ts-expect-error because `document` may still be undefined
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
