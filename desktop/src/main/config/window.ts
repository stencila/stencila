import { BrowserWindow } from 'electron'
import { createWindow } from '../../app/window'
import { i18n } from '../../i18n'

let settingsWindow: BrowserWindow | null

const settingsUrl = '/settings'

export const showSettings = () => {
  settingsWindow = createWindow(settingsUrl, {
    width: 800,
    height: 800,
    maxWidth: 1000,
    minWidth: 600,
    minHeight: 600,
    show: false,
    title: i18n.t('settings.title'),
  })

  settingsWindow.on('closed', () => {
    settingsWindow = null
  })

  settingsWindow.webContents.on('did-finish-load', () => {
    settingsWindow?.show()
  })

  settingsWindow?.loadURL(settingsUrl)
}
