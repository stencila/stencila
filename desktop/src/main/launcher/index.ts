import { ipcMain } from 'electron'
import { CHANNEL } from '../../preload'
import { removeChannelHandlers } from '../utils/handler'
import { LAUNCHER_CHANNEL } from './channels'
import { closeLauncherWindow, openLauncherWindow } from './window'

export const registerLauncherHandlers = () => {
  ipcMain.handle(CHANNEL.OPEN_LAUNCHER_WINDOW, async () => {
    openLauncherWindow()
  })

  ipcMain.handle(CHANNEL.CLOSE_LAUNCHER_WINDOW, async () => {
    closeLauncherWindow()
  })
}

export const removeLauncherHandlers = () => {
  removeChannelHandlers(LAUNCHER_CHANNEL)
}
