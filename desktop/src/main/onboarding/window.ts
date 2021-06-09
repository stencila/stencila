import { BrowserWindow } from 'electron'
import { createWindow } from '../../app/window'
import { i18n } from '../../i18n'

let onboardingWindow: BrowserWindow | null

const onboardingUrl = '/onboarding'

export const openOnboardingWindow = () => {
  onboardingWindow = createWindow(onboardingUrl, {
    width: 800,
    height: 600,
    maxWidth: 1000,
    minWidth: 600,
    minHeight: 600,
    show: false,
    title: i18n.t('onboarding.title'),
    center: true,
  })

  onboardingWindow.on('closed', () => {
    onboardingWindow = null
  })

  onboardingWindow.webContents.on('did-finish-load', () => {
    onboardingWindow?.show()
  })

  onboardingWindow?.loadURL(onboardingUrl)

  return onboardingWindow
}

export const closeOnboardingWindow = () => {
  onboardingWindow?.close()
}
