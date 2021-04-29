import { app, BrowserWindow, BrowserWindowConstructorOptions } from 'electron'
import { scheme } from '../main/app-protocol'

// declare const MAIN_WINDOW_WEBPACK_ENTRY: string
declare const MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY: string

const isDevelopment = process.env.NODE_ENV === 'development'

export const createWindow = (
  url: string,
  options: Omit<BrowserWindowConstructorOptions, 'webPreferences'> = {}
): BrowserWindow => {
  const win = new BrowserWindow({
    height: 860,
    width: 1024,
    ...options,
    webPreferences: {
      // TODO: Fix sandboxing, currently prevents `preload` script access
      sandbox: false,
      nodeIntegration: false,
      contextIsolation: true, // protect against prototype pollution
      enableRemoteModule: false,
      preload: MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY,
      additionalArguments: [`storePath:${app.getPath('userData')}`],
    },
  })

  win.loadURL(
    isDevelopment ? `http://localhost:3333${url}` : `${scheme}://rse${url}`
  )

  if (process.env.NODE_ENV === 'development') {
    // Open the DevTools.
    win.webContents.openDevTools()
  }

  return win
}
