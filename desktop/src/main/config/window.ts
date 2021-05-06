import { BrowserWindow } from 'electron'
import { createWindow } from '../../app/window'

let settingsWindow: BrowserWindow | null

const settingsUrl = '/settings'

export const showSettings = () => {
  const parent = BrowserWindow.getAllWindows()[0]

  settingsWindow = createWindow(settingsUrl, {
    width: 800,
    height: 800,
    maxWidth: 1000,
    minWidth: 600,
    minHeight: 600,
    show: false,
    parent,
  })

  settingsWindow.on('closed', () => {
    settingsWindow = null
  })

  settingsWindow.webContents.on('did-finish-load', () => {
    settingsWindow?.show()
  })

  settingsWindow?.loadURL(settingsUrl)
}
