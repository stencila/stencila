import { BrowserWindow } from 'electron'
import { registerOnboardingHandlers, removeOnboaringHandlers } from '.'
import { i18n } from '../../i18n'
import { registerConfigHandlers, removeConfigHandlers } from '../config'
import { registerLauncherHandlers, removeLauncherHandlers } from '../launcher'
import { createWindow } from '../window'

let onboardingWindow: BrowserWindow | null

const onboardingUrl = '/onboarding'

export const openOnboardingWindow = () => {
  onboardingWindow = createWindow(onboardingUrl, {
    width: 800,
    height: 600,
    maxWidth: 1000,
    minWidth: 600,
    maxHeight: 800,
    minHeight: 500,
    show: false,
    title: i18n.t('onboarding.title'),
    fullscreenable: false,
    center: true,
  })

  onboardingWindow.on('closed', () => {
    removeLauncherHandlers()
    removeConfigHandlers()
    removeOnboaringHandlers()
    onboardingWindow = null
  })

  onboardingWindow.webContents.on('did-finish-load', () => {
    registerLauncherHandlers()
    registerConfigHandlers()
    registerOnboardingHandlers()
    onboardingWindow?.show()
  })

  onboardingWindow?.loadURL(onboardingUrl)

  return onboardingWindow
}

export const closeOnboardingWindow = () => {
  onboardingWindow?.close()
}
