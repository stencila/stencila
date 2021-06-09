import { BrowserWindow } from 'electron'
import { createWindow } from '../../app/window'

let launcherWindow: BrowserWindow | null

const launcherUrl = '/'

export const openLauncherWindow = () => {
  if (launcherWindow) {
    launcherWindow.show()
    return launcherWindow
  }

  launcherWindow = createWindow(launcherUrl, {
    height: 430,
    width: 860,
    maxHeight: 860,
    maxWidth: 1200,
    minHeight: 350,
    minWidth: 600,
    center: true,
  })

  launcherWindow.on('closed', () => {
    launcherWindow = null
  })

  launcherWindow.webContents.on('did-finish-load', () => {
    launcherWindow?.show()
  })

  launcherWindow?.loadURL(launcherUrl)

  return launcherWindow
}

export const closeLauncherWindow = () => {
  launcherWindow?.close()
  launcherWindow = null
}
