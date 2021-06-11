import { ipcMain } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { removeChannelHandlers } from '../utils/handler'
import { LAUNCHER_CHANNEL } from './channels'
import { closeLauncherWindow, openLauncherWindow } from './window'

export const registerLauncherHandlers = () => {
  try {
    ipcMain.handle(CHANNEL.OPEN_LAUNCHER_WINDOW, async () => {
      openLauncherWindow()
    })

    ipcMain.handle(CHANNEL.CLOSE_LAUNCHER_WINDOW, async () => {
      closeLauncherWindow()
    })
  } catch {
    // Handlers likely already registered
  }
}

export const removeLauncherHandlers = () => {
  removeChannelHandlers(LAUNCHER_CHANNEL)
}
