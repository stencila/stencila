import { BrowserWindow, WebContents } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { AnyFunction } from '../../preload/types'

export const getFocusedWindow = () => {
  return BrowserWindow.getFocusedWindow()
}

export const createNewDocument = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENTS_CREATE)
}

export const saveActiveDoc = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENT_WRITE_ACTIVE)
}

export const saveActiveDocAs = () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENT_WRITE_ACTIVE_AS)
}

export const closeActiveTab = async () => {
  getFocusedWindow()?.webContents.send(CHANNEL.DOCUMENTS_CLOSE_ACTIVE)
}

/**
 * Listen to event sent by the renderer process when the UI is loaded and hydrated
 * @see https://stenciljs.com/docs/api#the-appload-event
 */
export const onUiLoaded =
  (webContents?: WebContents) => (callback: AnyFunction) => {
    if (!webContents) return

    webContents.on('ipc-message', (_e, channel) => {
      if (channel === CHANNEL.UI_READY) {
        callback()
      }
    })
  }
