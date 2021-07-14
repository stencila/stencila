import { CHANNEL } from '../../preload/channels'
import { LauncherWindowClose, LauncherWindowOpen } from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { LAUNCHER_CHANNEL } from './channels'
import { closeLauncherWindow, openLauncherWindow } from './window'

export const registerLauncherHandlers = () => {
  try {
    handle<LauncherWindowOpen>(CHANNEL.LAUNCHER_WINDOW_OPEN, async () => {
      openLauncherWindow()
      return valueToSuccessResult()
    })

    handle<LauncherWindowClose>(CHANNEL.LAUNCHER_WINDOW_CLOSE, async () => {
      closeLauncherWindow()
      return valueToSuccessResult()
    })
  } catch {
    // Handlers likely already registered
  }
}

export const removeLauncherHandlers = () => {
  removeChannelHandlers(LAUNCHER_CHANNEL)
}
