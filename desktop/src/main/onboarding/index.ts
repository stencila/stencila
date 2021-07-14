import { CHANNEL } from '../../preload/channels'
import {
  OnboardingWindowClose,
  OnboardingWindowOpen,
} from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { ONBOARDING_CHANNEL } from './channels'
import { closeOnboardingWindow, openOnboardingWindow } from './window'

export const registerOnboardingHandlers = () => {
  try {
    handle<OnboardingWindowOpen>(CHANNEL.ONBOARDING_WINDOW_OPEN, async () => {
      openOnboardingWindow()
      return valueToSuccessResult()
    })

    handle<OnboardingWindowClose>(CHANNEL.ONBOARDING_WINDOW_CLOSE, async () => {
      closeOnboardingWindow()
      return valueToSuccessResult()
    })
  } catch {
    // Handlers likely already registered
  }
}

export const removeOnboaringHandlers = () => {
  removeChannelHandlers(ONBOARDING_CHANNEL)
}
