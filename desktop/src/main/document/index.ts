import { ipcMain } from 'electron'
import { documents } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { removeChannelHandlers } from '../utils/handler'
import { DOCUMENT_CHANNEL } from './channel'
import { rewriteHtml } from '../local-protocol'

export const registerDocumentHandlers = () => {
  try {
    ipcMain.handle(
      CHANNEL.CLOSE_DOCUMENT,
      async (_event, documentId: string) => {
        try {
          documents.close(documentId)
        } catch (e) {}
      }
    )

    ipcMain.handle(
      CHANNEL.UNSUBSCRIBE_DOCUMENT,
      async (
        _event,
        { documentId, topics }: { documentId: string; topics: string[] }
      ) => {
        documents.unsubscribe(documentId, topics)
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
        return documents.dump(documentId)
      }
    )

    ipcMain.handle(
      CHANNEL.DOCUMENT_GET_PREVIEW,
      async (ipcEvent, documentId: string) => {
        documents.subscribe(
          documentId,
          ['encoded:html'],
          (_topic, docEvent) => {
            const event = {
              ...docEvent,
              content: rewriteHtml(docEvent.content ?? '')
            }
            ipcEvent.sender.send(CHANNEL.DOCUMENT_GET_PREVIEW, event)
          }
        )

        return rewriteHtml(documents.dump(documentId, 'html'))
      }
    )

    ipcMain.handle(
      CHANNEL.SAVE_DOCUMENT,
      async (
        _event,
        { documentId, content }: { documentId: string; content: string }
      ) => {
        documents.write(documentId, content)
      }
    )
  } catch {
    // Handlers likely already registered
  }
}

export const removeDocoumentHandlers = () => {
  removeChannelHandlers(DOCUMENT_CHANNEL)
}
