// To iron out:
// - race conditions for node execution responses
// - cancelling execution
// - handling stale status for downstream node dependencies
// - what does a `documents.change` payload look like for HTML nodes?
// - How do we migrate old published documents
// - Attach Node IDs for required elements in published article HTML

import {
  Document,
  DocumentEvent,
  DomPatch,
  Operation,
  Session,
} from '@stencila/stencila'
import { Client, ClientId, connect, disconnect } from './client'
import * as documents from './documents'
import { applyPatch } from './patches'
import * as sessions from './sessions'
import { ProjectId, SnapshotId } from './types'

export const main = (
  url: string,
  clientId: ClientId,
  projectId: ProjectId,
  snapshotId: SnapshotId,
  documentPath: documents.DocumentPath
): (() => Promise<[Client, Document, Session]>) => {
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
        .subscribe(client, document.id, 'patched', patchNode)
        .catch((err) => {
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

  // Execute a node, optionally updating properties prior to execution.
  async function executeNode(
    nodeId: documents.NodeId,
    properties: Record<string, unknown>
  ): Promise<void> {
    const [client, document] = await startup()
    const patch = Object.entries(properties).map(
      ([key, value]): Operation => ({
        type: 'Replace',
        address: [key],
        value,
        items: 1,
        length: 1,
      })
    )
    return documents.execute(client, document.id, nodeId, patch)
  }

  // Attach `executeNode` to event handlers
  window.onload = () => {
    window.addEventListener('patched', async (event) => {
      const [client, document] = await startup()
      // @ts-expect-error because this is temporary
      // eslint-disable-next-line
      const patch = event.detail
      return documents.patch(client, document.id, undefined, patch)
    })

    // `onChange` for `Parameter` nodes
    window.document.querySelectorAll('input').forEach((input) => {
      input.addEventListener('change', () => {
        // Using JSON.parse here but in the future we'd have the `Parameter` component
        // be providing the correct value type, based on it's `validator`..
        const value = JSON.parse(input.value) as unknown
        executeNode(input.id, { value }).catch((err) => {
          console.warn(`Couldn't execute the parameter`, err)
        })
      })
    })

    // `executeHandler` for `CodeChunk` and `CodeExpression` nodes
    window.document
      .querySelectorAll('stencila-code-chunk,stencila-code-expression')
      .forEach((elem) => {
        /* eslint-disable @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-return */
        // @ts-expect-error because we're not importing component types
        elem.executeHandler = (node) => {
          executeNode(elem.id, {
            text: node.text,
            programming_language:
              // This is a temporary hack for testing purposes. More work on
              // normalizing language names needed.
              node.programmingLanguage === 'plain text'
                ? 'calc'
                : node.programmingLanguage,
          }).catch((err) => {
            console.warn(`Couldn't execute the node`, err)
          })
          // The WebComponent for a `CodeExpression` has a `isOutputEmpty` property
          // which is set based on the return value from this function and does not
          // change later when we actually update the output. So, here's a hack to
          // make that always true.
          return { ...node, output: '' }
        }
        /* eslint-enable @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-return */
      })
  }

  // Patch a node
  ///
  /// Handles a document 'patched' event by either sending it to the relevant WebComponent
  /// so that it can make the necessary changes to the DOM, or by calling `applyPatch` which
  /// makes changes to the DOM directly.
  function patchNode(event: DocumentEvent): void {
    let patch
    if (event.type === 'patched') {
      patch = event.patch as DomPatch
    } else {
      console.error(
        `Expected document event to be of type 'patched', got type '${event.type}'`
      )
      return
    }

    // Patches for node types with WebComponents are handled differently
    // from patches to other DOM elements.
    if (patch.target !== undefined && patch.ops[0]?.type === 'Replace') {
      const type = patch.ops[0]?.json.type as string
      if (type === 'Parameter') {
        // Nothing to do (?)
        return
      } else if (type === 'CodeChunk') {
        window.dispatchEvent(
          new CustomEvent('document:patched', {
            detail: {
              nodeId: patch.target,
              value: patch.ops[0]?.json,
            },
          })
        )
        return
      } else if (type === 'CodeExpression') {
        window.dispatchEvent(
          new CustomEvent('document:node:changed', {
            detail: {
              nodeId: patch.target,
              value: patch.ops[0]?.json,
            },
          })
        )
        return
      } else {
        console.error(`Unexpected patch JSON value type '${type}'`)
        return
      }
    }

    applyPatch(patch)
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
