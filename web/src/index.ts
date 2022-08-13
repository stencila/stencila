import { DocumentPath } from './documents'
import { Document, Session } from './types'
import { Client, ClientId, ClientOptions, connect, disconnect } from './client'
import { onDiscoverExecutableLanguages } from './events/kernels'
import { languages } from './kernels'
import * as sessions from './sessions'

export type { Document, Patch, Session } from './types'
export type { Client } from './client'

// The form of the following re-exports is a workaround for the error
// "@parcel/transformer-typescript-types: node.exportClause.elements is not iterable"
// See https://github.com/parcel-bundler/parcel/issues/5911#issuecomment-1007642717

import * as client_ from './client'
export const client = { ...client_ }

import * as documents_ from './documents'
export const documents = { ...documents_ }

import * as patches_ from './patches'
export const patches = { ...patches_ }

import * as uid from './utils/uid'
export const utils = { ...uid }

export const main = (
  clientId: ClientId,
  documentPath?: DocumentPath,
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
      client = await connect(clientId, origin, token, clientOptions)
      window.stencilaWebClient.websocketClient = client
    }

    if (session === undefined) {
      session = await sessions.start(client)
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
      window.stencilaWebClient.documentId = document.id
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
