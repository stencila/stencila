import { ipcMain, IpcMainInvokeEvent } from 'electron'
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
