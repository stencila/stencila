import { ipcMain } from 'electron'
import { documents } from 'stencila'
import { CHANNEL } from '../../preload'

export const registerDocumentHandlers = () => {
  ipcMain.handle(
    CHANNEL.GET_DOCUMENT_CONTENTS,
    async (ipcEvent, filePath: string) => {
      let documentId = documents.open(filePath).id
      documents.subscribe(documentId, ['modified'], (_topic, docEvent) => {
        ipcEvent.sender.send(CHANNEL.GET_DOCUMENT_CONTENTS, docEvent)
      })
      return documents.read(documentId)
    }
  )

  ipcMain.handle(
    CHANNEL.DOCUMENT_GET_PREVIEW,
    async (ipcEvent, filePath: string) => {
      let documentId = documents.open(filePath).id
      documents.subscribe(documentId, ['encoded:html'], (_topic, docEvent) => {
        ipcEvent.sender.send(CHANNEL.DOCUMENT_GET_PREVIEW, docEvent)
      })
    }
  )

  ipcMain.handle(CHANNEL.CLOSE_DOCUMENT, async (_event, documentId: string) => {
    try {
      documents.close(documentId)
    } catch (e) {}
  })

  ipcMain.handle(
    CHANNEL.SAVE_DOCUMENT,
    async (
      _event,
      { documentId, content }: { documentId: string; content: string }
    ) => {
      documents.write(documentId, content)
    }
  )
}
