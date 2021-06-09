import { ipcMain } from 'electron'
import { CHANNEL } from '../../preload'
import { openLauncherWindow, closeLauncherWindow } from './window'

export const registerLauncherHandlers = () => {
  ipcMain.handle(CHANNEL.OPEN_LAUNCHER_WINDOW, async () => {
    openLauncherWindow()
  })

  ipcMain.handle(CHANNEL.CLOSE_LAUNCHER_WINDOW, async () => {
    closeLauncherWindow()
  })
}
