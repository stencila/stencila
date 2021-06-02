import { ipcMain } from 'electron'
import { Document, documents } from 'stencila'
import { CHANNEL } from '../../preload'

const documentRefs: Record<string, Document | undefined> = {}

const getDocument = (filePath: string): Document => {
  let doc = documentRefs[filePath]
  if (doc) {
    return doc
  }

  doc = documents.open(filePath)
  documentRefs[filePath] = doc

  return doc
}

const closeDocument = (filePath: string) => {
  documents.unsubscribe(filePath, ['encoded:html'])
  documents.close(filePath)
  delete documentRefs[filePath]
}

export const registerDocumentHandlers = () => {
  ipcMain.handle(
    CHANNEL.GET_DOCUMENT_CONTENTS,
    async (ipcEvent, filePath: string) => {
      getDocument(filePath)
      documents.subscribe(filePath, ['modified'], (_topic, docEvent) => {
        ipcEvent.sender.send(CHANNEL.GET_DOCUMENT_CONTENTS, docEvent)
      })

      return documents.read(filePath)
    }
  )

  ipcMain.handle(
    CHANNEL.DOCUMENT_GET_PREVIEW,
    async (ipcEvent, filePath: string) => {
      getDocument(filePath)
      documents.subscribe(filePath, ['encoded:html'], (_topic, docEvent) => {
        ipcEvent.sender.send(CHANNEL.DOCUMENT_GET_PREVIEW, docEvent)
      })
    }
  )

  ipcMain.handle(CHANNEL.CLOSE_DOCUMENT, async (_event, filePath: string) => {
    try {
      closeDocument(filePath)
    } catch (e) {}
  })

  ipcMain.handle(
    CHANNEL.SAVE_DOCUMENT,
    async (
      _event,
      { filePath, content }: { filePath: string; content: string }
    ) => {
      documents.write(filePath, content)
    }
  )
}
