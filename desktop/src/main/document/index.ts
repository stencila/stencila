import { ipcMain } from 'electron'
import { dispatch, documents } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { rewriteHtml } from '../local-protocol'
import { removeChannelHandlers } from '../utils/handler'
import { DOCUMENT_CHANNEL } from './channel'

export const registerDocumentHandlers = () => {
  try {
    ipcMain.handle(CHANNEL.DOCUMENTS_OPEN, async (_event, filePath: string) =>
      dispatch.documents.open(filePath)
    )

    ipcMain.handle(
      CHANNEL.CLOSE_DOCUMENT,
      async (_event, documentId: string) => {
        return dispatch.documents.close(documentId)
      }
    )

    ipcMain.handle(
      CHANNEL.UNSUBSCRIBE_DOCUMENT,
      async (
        _event,
        { documentId, topics }: { documentId: string; topics: string[] }
      ) => {
        return dispatch.documents.unsubscribe(documentId, topics)
      }
    )

    ipcMain.handle(
      CHANNEL.GET_DOCUMENT_CONTENTS,
      async (ipcEvent, documentId: string) => {
        documents.subscribe(documentId, ['modified'], (_topic, docEvent) => {
          ipcEvent.sender.send(CHANNEL.GET_DOCUMENT_CONTENTS, docEvent)
        })

        // Use `dump` to get document content, rather than `read`, to avoid
        // (a) a re-read of the file (that is done on open) (b) re-encoding for
        // each subscriber.
        return dispatch.documents.dump(documentId)
      }
    )

    ipcMain.handle(
      CHANNEL.GET_DOCUMENT_PREVIEW,
      async (ipcEvent, documentId: string) => {
        documents.subscribe(
          documentId,
          ['encoded:html'],
          (_topic, docEvent) => {
            const event = {
              ...docEvent,
              content: rewriteHtml(docEvent.content ?? ''),
            }

            ipcEvent.sender.send(CHANNEL.GET_DOCUMENT_PREVIEW, event)
          }
        )

        const results = dispatch.documents.dump(documentId, 'html')

        if (results.ok) {
          return {
            ok: results.ok,
            value: rewriteHtml(results.value),
            errors: results.errors,
          }
        } else {
          return results
        }
      }
    )

    ipcMain.handle(
      CHANNEL.SAVE_DOCUMENT,
      async (
        _event,
        { documentId, content }: { documentId: string; content: string }
      ) => {
        return dispatch.documents.write(documentId, content)
      }
    )
  } catch {
    // Handlers likely already registered
  }
}

export const removeDocoumentHandlers = () => {
  removeChannelHandlers(DOCUMENT_CHANNEL)
}
