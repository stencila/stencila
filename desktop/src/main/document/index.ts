import { dispatch, documents } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import {
  DocumentsClose,
  DocumentsDump,
  DocumentsOpen,
  DocumentsPreview,
  DocumentsUnsubscribe,
  DocumentsWrite,
} from '../../preload/types'
import { rewriteHtml } from '../local-protocol'
import { removeChannelHandlers } from '../utils/handler'
import { handle } from '../utils/rpc'
import { DOCUMENT_CHANNEL } from './channel'

export const registerDocumentHandlers = () => {
  try {
    handle<DocumentsOpen>(CHANNEL.DOCUMENTS_OPEN, async (_event, filePath) =>
      dispatch.documents.open(filePath)
    )

    handle<DocumentsClose>(
      CHANNEL.CLOSE_DOCUMENT,
      async (_event, documentId) => {
        return dispatch.documents.close(documentId)
      }
    )

    handle<DocumentsUnsubscribe>(
      CHANNEL.UNSUBSCRIBE_DOCUMENT,
      async (_event, documentId, topics) => {
        return dispatch.documents.unsubscribe(documentId, topics)
      }
    )

    handle<DocumentsDump>(
      CHANNEL.GET_DOCUMENT_CONTENTS,
      async (ipcEvent, documentId) => {
        documents.subscribe(documentId, ['modified'], (_topic, docEvent) => {
          ipcEvent.sender.send(CHANNEL.GET_DOCUMENT_CONTENTS, docEvent)
        })

        // Use `dump` to get document content, rather than `read`, to avoid
        // (a) a re-read of the file (that is done on open) (b) re-encoding for
        // each subscriber.
        return dispatch.documents.dump(documentId)
      }
    )

    handle<DocumentsPreview>(
      CHANNEL.GET_DOCUMENT_PREVIEW,
      async (ipcEvent, documentId) => {
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

    handle<DocumentsWrite>(
      CHANNEL.SAVE_DOCUMENT,
      async (_event, documentId, content) => {
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
