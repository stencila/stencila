import { CHANNEL } from '../../preload/channels'
import {
  OnboardingWindowClose,
  OnboardingWindowOpen,
} from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/rpc'
import { ONBOARDING_CHANNEL } from './channels'
import { closeOnboardingWindow, openOnboardingWindow } from './window'

export const registerOnboardingHandlers = () => {
  try {
    handle<OnboardingWindowOpen>(CHANNEL.OPEN_ONBOARDING_WINDOW, async () => {
      openOnboardingWindow()
      return valueToSuccessResult()
    })

    handle<OnboardingWindowClose>(CHANNEL.CLOSE_ONBOARDING_WINDOW, async () => {
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
