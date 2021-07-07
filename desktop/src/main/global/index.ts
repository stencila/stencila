import { ipcMain, shell } from 'electron'
import { dispatch, Call } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { captureError, LogHandler } from '../../preload/errors'

export const registerGlobalHandlers = () => {
  ipcMain.handle(
    CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER,
    (_event, link: string) => {
      shell.openExternal(link)
    }
  )

  ipcMain.handle(
    CHANNEL.RPC_CALL,
    async (_event, call: Call) => {
      return dispatch(call)
    }
  )

  ipcMain.handle(CHANNEL.CAPTURE_ERROR, (_event, payload: LogHandler) => {
    captureError(payload)
  })
}
