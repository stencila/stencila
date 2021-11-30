import { BrowserWindow, BrowserWindowConstructorOptions } from 'electron'
import { i18n } from '../../i18n'
import { captureError } from '../../preload/errors'
import { isDevelopment } from '../../preload/utils/env'
import { scheme } from '../app-protocol'
import { hardenWindow } from './security'

// declare const MAIN_WINDOW_WEBPACK_ENTRY: string
declare const MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY: string

export const createWindow = (
  url: string,
  options: Omit<BrowserWindowConstructorOptions, 'webPreferences'> = {}
): BrowserWindow => {
  const win = new BrowserWindow({
    height: 860,
    width: 1024,
    title: i18n.t('core.title'),
    ...options,
    webPreferences: {
      // TODO: Fix sandboxing, currently prevents `preload` script access
      sandbox: false,
      nodeIntegration: false,
      contextIsolation: true, // protect against prototype pollution
      preload: MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY,
    },
  })

  win
    .loadURL(
      isDevelopment ? `http://localhost:3333${url}` : `${scheme}://rse${url}`
    )
    .catch((err) => captureError(err))

  if (isDevelopment) {
    // Open the DevTools.
    win.webContents.openDevTools()
  }

  hardenWindow(win)

  return win
}
