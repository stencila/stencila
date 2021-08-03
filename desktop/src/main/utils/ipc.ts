import { ipcMain, IpcMainInvokeEvent, WebContents, webContents } from 'electron'
import { Result } from 'stencila'
import { InvokeTypes } from '../../preload/types'

export function valueToSuccessResult<V>(
  value?: V,
  errors?: Result['errors']
): Result<V>
export function valueToSuccessResult(
  value?: undefined,
  errors?: Result['errors']
): Result<undefined> {
  return {
    ok: true,
    value,
    errors: errors ?? [],
  }
}

// Send the passed IPC message to all open windows of the configuration change.
// Each window should register a corresponding listener and react as needed to the changes.
export const sendToAllWindows = (
  ...args: Parameters<WebContents['send']>
): void => {
  webContents.getAllWebContents().forEach((wc) => {
    wc.send(...args)
  })
}

/**
 * A wrapper around Electron's `ipcMain.handle` function in order to enable type
 * type safe invocation of both the Invoke and Handle aspects.
 * For details see `/preload/types.d.ts`
 */
export function handle<F extends InvokeTypes>(
  channel: F['channel'],
  listener: (ipcEvent: IpcMainInvokeEvent, ...args: F['args']) => F['result']
): void
export function handle<F extends InvokeTypes>(
  channel: F['channel'],
  listener: (ipcEvent: IpcMainInvokeEvent, args: F['args']) => F['result']
): void {
  ipcMain.handle(channel, listener)
}
