import { BrowserWindow } from 'electron'
import { configHandlers } from '.'
import { i18n } from '../../i18n'
import { registerBaseMenu } from '../menu'
import { createWindow } from '../window'

let settingsWindow: BrowserWindow | null

const settingsUrl = '/settings'

export const showSettings = () => {
  if (settingsWindow) {
    settingsWindow.show()
    return settingsWindow
  }

  settingsWindow = createWindow(settingsUrl, {
    width: 800,
    height: 800,
    maxWidth: 1000,
    minWidth: 600,
    minHeight: 600,
    show: false,
    title: i18n.t('settings.title'),
  })

  // The ID needs to be stored separately from the window object. Otherwise an error
  // is thrown because the time remove handlers are called the window object is already destroyed.
  const windowId = settingsWindow.id

  configHandlers.register(windowId)

  settingsWindow.on('closed', () => {
    configHandlers.remove(windowId)
    settingsWindow = null
  })

  settingsWindow.webContents.on('did-finish-load', () => {
    settingsWindow?.show()
  })

  settingsWindow.on('focus', () => {
    registerBaseMenu()
  })

  settingsWindow?.loadURL(settingsUrl)
}
