import { DOCUMENT_CHANNEL } from '../main/document/channel'
import { CONFIG_CHANNEL } from '../main/config/channels'
import { PROJECT_CHANNEL } from '../main/project/channels'

export const CHANNEL = {
  ...CONFIG_CHANNEL,
  ...PROJECT_CHANNEL,
  ...DOCUMENT_CHANNEL,
} as const

export type Channel = keyof typeof CHANNEL

export type Handler = (...args: unknown[]) => void

export const isChannel = (maybeChannel: string): maybeChannel is Channel => {
  return Object.keys(CHANNEL).includes(maybeChannel)
}
