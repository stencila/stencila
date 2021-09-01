import { BrowserWindow, WebContents } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { AnyFunction } from '../../preload/types'

export const getFocusedWindow = () => {
  return BrowserWindow.getFocusedWindow()
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
