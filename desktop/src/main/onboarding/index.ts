import { ipcMain } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { removeChannelHandlers } from '../utils/handler'
import { ONBOARDING_CHANNEL } from './channels'
import { closeOnboardingWindow, openOnboardingWindow } from './window'

export const registerOnboardingHandlers = () => {
  try {
    ipcMain.handle(CHANNEL.OPEN_ONBOARDING_WINDOW, async () => {
      return openOnboardingWindow()
    })

    ipcMain.handle(CHANNEL.CLOSE_ONBOARDING_WINDOW, async () => {
      return closeOnboardingWindow()
    })
  } catch {
    // Handlers likely already registered
  }
}

export const removeOnboaringHandlers = () => {
  removeChannelHandlers(ONBOARDING_CHANNEL)
}
