import { ipcMain } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { logStore } from '../../preload/logging'
import { makeHandlers, removeChannelHandlers } from '../utils/handler'
import { valueToSuccessResult } from '../utils/ipc'
import { LOG_CHANNEL } from './channels'
import { showLogs } from './window'

const registerLogHandlers = () => {
  ipcMain.handle(CHANNEL.LOGS_WINDOW_OPEN, async () => {
    showLogs()
    return valueToSuccessResult()
  })

  ipcMain.handle(CHANNEL.LOGS_GET, async () => {
    return valueToSuccessResult(logStore)
  })
}

const removeLogHandlers = () => {
  removeChannelHandlers(LOG_CHANNEL)
}

export const logHandlers = makeHandlers(registerLogHandlers, removeLogHandlers)
