import { BrowserWindow } from 'electron'
import { logHandlers } from '.'
import { i18n } from '../../i18n'
import { streamLogsToWindow } from '../../preload/logging'
import { registerBaseMenu } from '../menu'
import { createWindow } from '../window'
import { onUiLoaded } from '../window/windowUtils'

let logsWindow: BrowserWindow | null

const logsUrl = '/logs'

export const showLogs = () => {
  if (logsWindow) {
    logsWindow.show()
    return logsWindow
  }

  logsWindow = createWindow(logsUrl, {
    width: 600,
    height: 800,
    minWidth: 200,
    minHeight: 600,
    show: false,
    title: i18n.t('logs.title'),
  })

  // The ID needs to be stored separately from the window object. Otherwise an error
  // is thrown because the time remove handlers are called the window object is already destroyed.
  const windowId = logsWindow.id

  logHandlers.register(windowId)

  logsWindow.on('closed', () => {
    logHandlers.remove(windowId)
    logsWindow = null
  })

  streamLogsToWindow(logsWindow.webContents)

  onUiLoaded(logsWindow.webContents)(() => {
    logsWindow?.show()
  })

  logsWindow.on('focus', () => {
    registerBaseMenu()
  })

  logsWindow?.loadURL(logsUrl)
}
