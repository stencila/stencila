// To iron out:
// - race conditions for node execution responses
// - cancelling execution
// - handling stale status for downstream node dependencies
// - what does a `documents.change` payload look like for HTML nodes?
// - How do we migrate old published documents
// - Attach Node IDs for required elements in published article HTML

import { FileFormatUtils } from '@stencila/components'
import { CodeChunk, CodeExpression } from '@stencila/schema'
import { Document, Operation, Session } from '@stencila/stencila'
import { Client, ClientId, connect, disconnect } from './client'
import * as documents from './documents'
import { available } from './kernels'
import * as sessions from './sessions'
import { ProjectId, SnapshotId } from './types'

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
  const availableKernels: FileFormatUtils.FileFormatMap = {}

  // Start the client and session, if necessary
  const startup = async (): Promise<[Client, Document, Session]> => {
    if (!client) {
      client = await connect(projectId, clientId, origin, token)
    }

    if (!session) {
      session = await sessions.start(client, projectId, snapshotId)
      sessions.subscribe(client, session.id, 'updated').catch((err) => {
        console.warn(`Couldn't subscribe to session updates`, err)
      })

      const kernels = await available(client, session.id)
      kernels.forEach((kernelName) => {
        const foundFormat = FileFormatUtils.lookupFormat(kernelName)
        availableKernels[foundFormat.name] = foundFormat
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
        .subscribe(client, document.id, 'patched', (event) =>
          documents.receivePatch(clientId, event)
        )
        .catch((err) => {
          console.warn(`Couldn't subscribe to document 'patched'`, err)
        })
    }

    documents.listen(client, clientId, document.id)

    window.addEventListener('appload', initComponents)

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
    const ops = Object.entries(properties).map(
      ([key, value]): Operation => ({
        type: 'Replace',
        address: [key],
        value,
        items: 1,
        length: 1,
      })
    )
    return documents.execute(client, document.id, nodeId, { ops })
  }

  function initComponents(): void {
    window.removeEventListener('appload', initComponents)

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
      .querySelectorAll<
        HTMLStencilaCodeChunkElement | HTMLStencilaCodeExpressionElement
      >('stencila-code-chunk,stencila-code-expression')
      .forEach((elem) => {
        elem.executeHandler = <C extends CodeChunk | CodeExpression>(
          node: C
        ): Promise<C> => {
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
          return Promise.resolve({ ...node, output: '' })
        }

        // @ts-expect-error TODO: Remove once CodeExpression components also support `executableLanguages`
        elem.executableLanguages = availableKernels
      })
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
