import { CONFIG_CHANNEL } from '../main/config/channels'
import { DOCUMENT_CHANNEL } from '../main/document/channel'
import { GLOBAL_CHANNEL } from '../main/global/channels'
import { LAUNCHER_CHANNEL } from '../main/launcher/channels'
import { LOG_CHANNEL } from '../main/logging/channels'
import { ONBOARDING_CHANNEL } from '../main/onboarding/channels'
import { PROJECT_CHANNEL } from '../main/project/channels'
import { UNPROTECTED_STORE_CHANNEL } from '../main/store/channels'

export const CHANNEL = {
  ...GLOBAL_CHANNEL,
  ...UNPROTECTED_STORE_CHANNEL,
  ...LAUNCHER_CHANNEL,
  ...CONFIG_CHANNEL,
  ...LOG_CHANNEL,
  ...PROJECT_CHANNEL,
  ...DOCUMENT_CHANNEL,
  ...ONBOARDING_CHANNEL,
} as const

export type Channel = keyof typeof CHANNEL

export type Handler = (...args: unknown[]) => void

export const isChannel = (maybeChannel: string): maybeChannel is Channel => {
  return Object.keys(CHANNEL).includes(maybeChannel)
}
