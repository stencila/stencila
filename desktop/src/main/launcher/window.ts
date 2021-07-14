import { BrowserWindow } from 'electron'
import { registerLauncherHandlers, removeLauncherHandlers } from '.'
import { registerConfigHandlers, removeConfigHandlers } from '../config'
import { registerProjectHandlers } from '../project'
import { createWindow } from '../window'

let launcherWindow: BrowserWindow | null

const launcherUrl = '/'

export const openLauncherWindow = () => {
  if (launcherWindow) {
    launcherWindow.show()
    return launcherWindow
  }

  launcherWindow = createWindow(launcherUrl, {
    height: 380,
    width: 760,
    maxHeight: 380,
    maxWidth: 960,
    minHeight: 310,
    minWidth: 600,
    center: true,
  })

  launcherWindow.on('closed', () => {
    removeLauncherHandlers()
    removeConfigHandlers()
    launcherWindow = null
  })

  launcherWindow.webContents.on('did-finish-load', () => {
    registerConfigHandlers()
    registerProjectHandlers()
    registerLauncherHandlers()
    launcherWindow?.show()
  })

  launcherWindow?.loadURL(launcherUrl)

  return launcherWindow
}

export const closeLauncherWindow = () => {
  launcherWindow?.close()
  launcherWindow = null
}
