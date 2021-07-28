import { BrowserWindow, dialog, SaveDialogReturnValue } from 'electron'
import { dispatch, documents } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import {
  DocumentsAlter,
  DocumentsClose,
  DocumentsCreate,
  DocumentsCreateFilePath,
  DocumentsDump,
  DocumentsGet,
  DocumentsLoad,
  DocumentsOpen,
  DocumentsPreview,
  DocumentsUnsubscribe,
  DocumentsWrite,
} from '../../preload/types'
import { rewriteHtml } from '../local-protocol'
import { makeHandlers, removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { DOCUMENT_CHANNEL } from './channel'

const registerDocumentHandlers = () => {
  handle<DocumentsOpen>(CHANNEL.DOCUMENTS_OPEN, async (_event, filePath) =>
    dispatch.documents.open(filePath)
  )

  handle<DocumentsCreate>(CHANNEL.DOCUMENTS_CREATE, async (_event, format) =>
    dispatch.documents.create(format)
  )

  handle<DocumentsClose>(
    CHANNEL.DOCUMENTS_CLOSE,
    async (_event, documentId) => {
      return dispatch.documents.close(documentId)
    }
  )

  handle<DocumentsUnsubscribe>(
    CHANNEL.DOCUMENTS_UNSUBSCRIBE,
    async (_event, documentId, topics) => {
      return dispatch.documents.unsubscribe(documentId, topics)
    }
  )

  handle<DocumentsDump>(
    CHANNEL.DOCUMENTS_DUMP,
    async (ipcEvent, documentId) => {
      documents.subscribe(documentId, ['modified'], (_topic, docEvent) => {
        ipcEvent.sender.send(CHANNEL.DOCUMENTS_DUMP, docEvent)
      })

      // Use `dump` to get document content, rather than `read`, to avoid
      // (a) a re-read of the file (that is done on open) (b) re-encoding for
      // each subscriber.
      return dispatch.documents.dump(documentId)
    }
  )

  handle<DocumentsLoad>(
    CHANNEL.DOCUMENTS_LOAD,
    async (_ipcEvent, documentId, contents) => {
      return dispatch.documents.load(documentId, contents)
    }
  )

  handle<DocumentsGet>(CHANNEL.DOCUMENTS_GET, async (_event, documentId) =>
    dispatch.documents.get(documentId)
  )

  handle<DocumentsPreview>(
    CHANNEL.DOCUMENTS_PREVIEW,
    async (ipcEvent, documentId) => {
      documents.subscribe(documentId, ['encoded:html'], (_topic, docEvent) => {
        const event = {
          ...docEvent,
          content: rewriteHtml(docEvent.content ?? ''),
        }

        ipcEvent.sender.send(CHANNEL.DOCUMENTS_PREVIEW, event)
      })

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
    CHANNEL.DOCUMENTS_WRITE,
    async (_event, documentId, content) => {
      return dispatch.documents.write(documentId, content)
    }
  )

  handle<DocumentsAlter>(
    CHANNEL.DOCUMENTS_ALTER,
    async (_event, documentId, path, format) => {
      return dispatch.documents.alter(documentId, path, format)
    }
  )

  handle<DocumentsCreateFilePath>(
    CHANNEL.DOCUMENTS_CREATE_FILE_PATH,
    async (event) => {
      const win = BrowserWindow.fromWebContents(event.sender) ?? undefined
      let file: SaveDialogReturnValue

      // Conditional check required to satisfy TypeScript function overload
      if (win) {
        file = await dialog.showSaveDialog(win, {
          securityScopedBookmarks: true,
          showsTagField: false,
        })
      } else {
        file = await dialog.showSaveDialog({
          securityScopedBookmarks: true,
          showsTagField: false,
        })
      }

      return valueToSuccessResult(file.filePath)
    }
  )
}

const removeDocoumentHandlers = () => {
  removeChannelHandlers(DOCUMENT_CHANNEL)
}

export const documentHandlers = makeHandlers(
  registerDocumentHandlers,
  removeDocoumentHandlers
)
