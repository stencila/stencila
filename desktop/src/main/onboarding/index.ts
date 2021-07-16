import { CHANNEL } from '../../preload/channels'
import {
  OnboardingWindowClose,
  OnboardingWindowOpen,
} from '../../preload/types'
import { makeHandlers, removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { ONBOARDING_CHANNEL } from './channels'
import { closeOnboardingWindow, openOnboardingWindow } from './window'

const registerOnboardingHandlers = () => {
  handle<OnboardingWindowOpen>(CHANNEL.ONBOARDING_WINDOW_OPEN, async () => {
    openOnboardingWindow()
    return valueToSuccessResult()
  })

  handle<OnboardingWindowClose>(CHANNEL.ONBOARDING_WINDOW_CLOSE, async () => {
    closeOnboardingWindow()
    return valueToSuccessResult()
  })
}

const removeOnboaringHandlers = () => {
  removeChannelHandlers(ONBOARDING_CHANNEL)
}

export const onboardingHandlers = makeHandlers(
  registerOnboardingHandlers,
  removeOnboaringHandlers
)
