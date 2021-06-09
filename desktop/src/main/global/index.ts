import { ipcMain, shell } from 'electron'
import { captureError, LogHandler } from '../../debug'
import { CHANNEL } from '../../preload'

export const registerGlobalHandlers = () => {
  ipcMain.handle(
    CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER,
    (_event, link: string) => {
      shell.openExternal(link)
    }
  )

  ipcMain.handle(CHANNEL.CAPTURE_ERROR, (_event, payload: LogHandler) => {
    captureError(payload)
  })
}
