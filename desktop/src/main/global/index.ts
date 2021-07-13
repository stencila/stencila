import { ipcMain, shell } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { captureError, LogHandler } from '../../preload/errors'
import { valueToSuccessResult } from '../utils/rpc'

export const registerGlobalHandlers = () => {
  ipcMain.handle(CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER, (_event, link: string) =>
    shell.openExternal(link).then(() => valueToSuccessResult())
  )

  ipcMain.handle(CHANNEL.CAPTURE_ERROR, (_event, payload: LogHandler) => {
    captureError(payload)
  })
}
