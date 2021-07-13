import { BrowserWindow } from 'electron'
import { CHANNEL } from '../../preload/channels'

export const getFocusedWindow = () => {
  return BrowserWindow.getFocusedWindow()
}

export const saveActiveDoc = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENT_WRITE_ACTIVE)
}

export const closeActiveTab = async () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENTS_CLOSE_ACTIVE)
}
