import { CHANNEL } from '../../preload/channels'
import { LauncherWindowClose, LauncherWindowOpen } from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/rpc'
import { LAUNCHER_CHANNEL } from './channels'
import { closeLauncherWindow, openLauncherWindow } from './window'

export const registerLauncherHandlers = () => {
  try {
    handle<LauncherWindowOpen>(CHANNEL.OPEN_LAUNCHER_WINDOW, async () => {
      openLauncherWindow()
      return valueToSuccessResult()
    })

    handle<LauncherWindowClose>(CHANNEL.CLOSE_LAUNCHER_WINDOW, async () => {
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
