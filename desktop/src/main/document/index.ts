import { ipcMain } from 'electron'
import { Document, documents } from 'stencila'
import { CHANNEL } from '../../preload'
import { projectWindow } from '../project/window'

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
    async (_event, filePath: string) => {
      getDocument(filePath)
      return documents.read(filePath)
    }
  )

  ipcMain.handle(
    CHANNEL.DOCUMENT_GET_PREVIEW,
    async (_event, filePath: string) => {
      getDocument(filePath)
      documents.subscribe(filePath, ['encoded:html'], (_topic, event) => {
        projectWindow?.webContents.send(CHANNEL.DOCUMENT_GET_PREVIEW, event)
      })
    }
  )

  ipcMain.handle(CHANNEL.CLOSE_DOCUMENT, async (_event, filePath: string) =>
    closeDocument(filePath)
  )
}
